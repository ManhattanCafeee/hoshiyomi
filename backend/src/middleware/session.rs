use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use cookie::time::Duration;

use crate::state::AppState;

/// 会话滑动续期:请求携带会话 Cookie 且会话已过半 TTL 时,重置服务端 TTL
/// 并在响应中刷新 Cookie Max-Age,使客户端与滑动 TTL 同步。
/// 认证与过期判定由 SessionCtx 提取器负责;本层只做续期与 Cookie 同步。
pub async fn refresh_session_cookie(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let cookie_name = state.cfg().auth.session.cookie_name.clone();
    let session_id = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').map(str::trim).find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                (k == cookie_name).then(|| v.to_string())
            })
        });

    let mut response = next.run(request).await;

    let Some(session_id) = session_id else {
        return response;
    };
    // 会话不存在(未登录/已过期被提取器删除)时直接放行
    let session = match state.srv().session.find(&session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return response,
        Err(e) => {
            tracing::error!(error = %e, "会话查询失败,跳过滑动续期");
            return response;
        }
    };
    if !state.srv().session.should_extend(&session) {
        return response;
    }
    match state.srv().session.extend(&session_id).await {
        Ok(expires_at) => {
            let remaining = (expires_at - chrono::Utc::now()).num_seconds().max(0);
            let mut cookie = Cookie::new(cookie_name, session_id);
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Lax);
            cookie.set_max_age(Some(Duration::seconds(remaining)));
            if let Ok(value) = axum::http::HeaderValue::from_str(&cookie.encoded().to_string()) {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }
        Err(e) => tracing::error!(error = %e, "会话续期失败"),
    }

    response
}
