use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    common::{extractor::AppJson, response::ApiResponse},
    error::{AppError, ErrorKind},
    modules::{role::models::DefaultRole, user::models::UserResp},
    state::AppState,
};

use super::{
    extractor::{JwtCtx, SessionCtx, remove_session_cookie, set_session_cookie},
    models::{
        AuthStateResp, JwtEchoResp, LoginReq, LoginResp, MessageResp, RefreshReq, RefreshResp,
        RegisterReq,
    },
};

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterReq,
    responses(
        (status = 200, description = "注册的用户", body = ApiResponse<UserResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 409, description = "用户名或邮箱已存在"),
    ),
)]
pub async fn register(
    State(state): State<AppState>,
    AppJson(payload): AppJson<RegisterReq>,
) -> Result<impl IntoResponse, AppError> {
    let user = state
        .srv()
        .user
        .create(payload.username, payload.email, payload.password)
        .await?;

    let role = state
        .srv()
        .role
        .find_by_name(DefaultRole::User.name())
        .await?
        .ok_or_else(|| ErrorKind::Internal.msg("默认角色未初始化,请先运行 CLI 的 init 命令"))?;
    state.srv().role.assign_to_user(user.id, role.id).await?;

    Ok(Json(ApiResponse::success(UserResp::from(user))))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginReq,
    responses(
        (status = 200, description = "登录成功,设置会话 Cookie", body = ApiResponse<AuthStateResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 403, description = "用户名或密码错误"),
    ),
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    AppJson(payload): AppJson<LoginReq>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user = state
        .srv()
        .auth
        .authenticate(&payload.username, &payload.password)
        .await?
        .ok_or_else(|| ErrorKind::InvalidCredentials.msg("用户名或密码错误"))?;

    let session_id = state.srv().session.create(auth_user.user.id).await?;
    let jar = set_session_cookie(jar, &state, &session_id);

    Ok((
        jar,
        Json(ApiResponse::success(AuthStateResp {
            user: UserResp::from(auth_user.user),
            permissions: auth_user.permissions,
        })),
    ))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "当前用户", body = ApiResponse<AuthStateResp>),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn me(
    State(state): State<AppState>,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, AppError> {
    let auth_user = state.srv().auth.get_auth_user(ctx.user_id).await?;

    Ok(Json(ApiResponse::success(AuthStateResp {
        user: UserResp::from(auth_user.user),
        permissions: auth_user.permissions,
    })))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "已退出", body = ApiResponse<MessageResp>),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, AppError> {
    state.srv().session.delete_by_user_id(ctx.user_id).await?;

    // 清除客户端 Cookie,避免浏览器此后每次请求携带死会话
    let jar = remove_session_cookie(jar, &state);

    Ok((
        jar,
        Json(ApiResponse::success(MessageResp {
            message: "已退出登录".to_string(),
        })),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/jwt/login",
    tag = "auth",
    request_body = LoginReq,
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<LoginResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 403, description = "用户名或密码错误"),
    ),
)]
pub async fn jwt_login(
    State(state): State<AppState>,
    AppJson(payload): AppJson<LoginReq>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user = state
        .srv()
        .auth
        .authenticate(&payload.username, &payload.password)
        .await?
        .ok_or_else(|| ErrorKind::InvalidCredentials.msg("用户名或密码错误"))?;

    let access_token = state.srv().token.encode_access_token(&auth_user.user)?;
    let refresh_token = state
        .srv()
        .token
        .generate_refresh_token(auth_user.user.id)
        .await?;

    Ok(Json(ApiResponse::success(LoginResp {
        access_token,
        refresh_token,
        state: AuthStateResp {
            user: UserResp::from(auth_user.user),
            permissions: auth_user.permissions,
        },
    })))
}

#[utoipa::path(
    post,
    path = "/auth/jwt/refresh",
    tag = "auth",
    request_body = RefreshReq,
    responses(
        (status = 200, description = "新令牌对", body = ApiResponse<RefreshResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "刷新令牌无效或已过期"),
    ),
)]
pub async fn jwt_refresh(
    State(state): State<AppState>,
    AppJson(payload): AppJson<RefreshReq>,
) -> Result<impl IntoResponse, AppError> {
    let rotated = state
        .srv()
        .token
        .rotate_refresh_token(&payload.refresh_token)
        .await?;

    Ok(Json(ApiResponse::success(RefreshResp {
        access_token: rotated.access_token,
        refresh_token: rotated.refresh_token,
    })))
}

#[utoipa::path(
    post,
    path = "/auth/jwt/logout",
    tag = "auth",
    responses(
        (status = 200, description = "已撤销全部刷新令牌", body = ApiResponse<MessageResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_logout(
    State(state): State<AppState>,
    ctx: JwtCtx,
) -> Result<impl IntoResponse, AppError> {
    state
        .srv()
        .token
        .delete_all_refresh_tokens(ctx.user_id)
        .await?;

    Ok(Json(ApiResponse::success(MessageResp {
        message: "已退出登录".to_string(),
    })))
}

#[utoipa::path(
    get,
    path = "/auth/jwt/me",
    tag = "auth",
    responses(
        (status = 200, description = "当前用户", body = ApiResponse<AuthStateResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_me(
    State(state): State<AppState>,
    ctx: JwtCtx,
) -> Result<impl IntoResponse, AppError> {
    let auth_user = state.srv().auth.get_auth_user(ctx.user_id).await?;

    Ok(Json(ApiResponse::success(AuthStateResp {
        user: UserResp::from(auth_user.user),
        permissions: auth_user.permissions,
    })))
}

#[utoipa::path(
    get,
    path = "/auth/jwt/echo",
    tag = "auth",
    responses(
        (status = 200, description = "认证检查示例", body = ApiResponse<JwtEchoResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_echo(ctx: JwtCtx) -> Result<impl IntoResponse, AppError> {
    Ok(Json(ApiResponse::success(JwtEchoResp {
        user_id: ctx.user_id,
        username: ctx.username().to_string(),
    })))
}
