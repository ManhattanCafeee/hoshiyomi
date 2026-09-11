use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::{modules::auth::models::HsClaims, state::AppState};
use vivarium_rs::{ApiError, ErrorKind};

/// 从请求提取的 JWT 认证上下文(Bearer token)
#[derive(Debug)]
pub struct JwtCtx {
    pub user_id: u64,
    username: String,
}

impl JwtCtx {
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl FromRequestParts<AppState> for JwtCtx {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        // 方案名按 RFC 7235 大小写不敏感(库的 JWT 中间件同样用 eq_ignore_ascii_case 解析)
        let token = vivarium_rs::get_authorization(&parts.headers)
            .and_then(|raw| raw.split_once(' '))
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
            .map(|(_, token)| token)
            .ok_or_else(|| ApiError::unauthorized("未授权"))?;

        let claims = state
            .srv()
            .runtime()
            .jwt
            .decode::<HsClaims>(token)
            .map_err(|e| ApiError::new(ErrorKind::Unauthorized, "无效的令牌").with_source(e))?;

        Ok(JwtCtx {
            user_id: claims.sub,
            username: claims.username,
        })
    }
}

/// 从会话 Cookie 提取的认证上下文。
///
/// 会话的解析、查库、续期由库的 `session_layer` 完成,并把 `SessionCtx<u64>` 注入请求扩展;
/// 这里只读扩展,按「请求是否带了该 cookie」区分两条 401 文案。
#[derive(Debug)]
pub struct SessionCtx {
    pub user_id: u64,
}

impl FromRequestParts<AppState> for SessionCtx {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        if let Some(ctx) = parts.extensions.get::<vivarium_rs::SessionCtx<u64>>() {
            return Ok(SessionCtx {
                user_id: ctx.user_id,
            });
        }

        let name = state.srv().session().cookie().name.as_str();
        let had_cookie = parts
            .headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|raw| {
                raw.split(';')
                    .any(|pair| pair.trim_start().starts_with(&format!("{name}=")))
            });

        Err(ApiError::unauthorized(if had_cookie {
            "未授权"
        } else {
            "缺少会话 Cookie"
        }))
    }
}
