use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration;

use crate::{
    bail,
    error::{AppError, ErrorKind, OptionAppExt, ResultExt},
    modules::user::models::User,
    state::{AppState, Services},
};

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

    pub async fn user(&self, services: &Services) -> Result<User, AppError> {
        services.user.get_by_id(self.user_id).await
    }
}

impl FromRequestParts<AppState> for JwtCtx {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ErrorKind::Unauthorized)?;

        let claims = state.srv().token.decode_access_token(token)?;

        Ok(JwtCtx {
            user_id: claims.sub,
            username: claims.username,
        })
    }
}

pub fn set_session_cookie(jar: CookieJar, state: &AppState, session_id: &str) -> CookieJar {
    let name = state.cfg().auth.session.cookie_name.clone();
    let ttl_secs = state.cfg().auth.session.ttl_hours * 3600;
    let mut cookie = Cookie::new(name, session_id.to_owned());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(Some(Duration::seconds(ttl_secs as i64)));
    jar.add(cookie)
}

pub fn remove_session_cookie(jar: CookieJar, state: &AppState) -> CookieJar {
    let name = state.cfg().auth.session.cookie_name.clone();
    let mut cookie = Cookie::new(name, "");
    cookie.set_path("/");
    jar.remove(cookie)
}

/// 从会话 Cookie 提取的认证上下文
#[derive(Debug)]
pub struct SessionCtx {
    pub user_id: u64,
}

impl SessionCtx {
    pub async fn user(&self, services: &Services) -> Result<User, AppError> {
        services.user.get_by_id(self.user_id).await
    }
}

impl FromRequestParts<AppState> for SessionCtx {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .err_kind(ErrorKind::Unauthorized)?;

        let session_id = jar
            .get(&state.cfg().auth.session.cookie_name)
            .ok_or_err_msg(ErrorKind::Unauthorized, "缺少会话 Cookie")?
            .value()
            .to_owned();

        let session = state
            .srv()
            .session
            .find(&session_id)
            .await?
            .ok_or(ErrorKind::Unauthorized)?;

        if session.expires_at < chrono::Utc::now() {
            state.srv().session.delete(&session_id).await?;
            bail!(ErrorKind::Unauthorized, "会话已过期");
        }

        // 滑动续期由 middleware::session::refresh_session_cookie 在响应阶段完成
        Ok(SessionCtx {
            user_id: session.user_id,
        })
    }
}

impl FromRequestParts<AppState> for Option<SessionCtx> {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(SessionCtx::from_request_parts(parts, state).await.ok())
    }
}
