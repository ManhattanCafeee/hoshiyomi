use axum::{Json, extract::State, http::header, response::IntoResponse};

use crate::{
    common::response::ApiResponse,
    modules::{role::models::DefaultRole, user::models::UserResp},
    state::AppState,
};
use vivarium_rs::{ApiError, ErrorKind, Varser};

use super::{
    extractor::{JwtCtx, SessionCtx},
    models::{
        AuthStateResp, HsClaims, JwtEchoResp, LoginReq, LoginResp, MessageResp, RefreshReq,
        RefreshResp, RegisterReq,
    },
};

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    operation_id = "Auth__register",
    request_body = RegisterReq,
    responses(
        (status = 200, description = "注册的用户", body = ApiResponse<UserResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 409, description = "用户名或邮箱已存在"),
    ),
)]
pub async fn register(
    State(state): State<AppState>,
    Varser(payload): Varser<RegisterReq>,
) -> Result<impl IntoResponse, ApiError> {
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
        .ok_or_else(|| {
            ApiError::new(
                ErrorKind::Internal,
                "默认角色未初始化,请先运行 CLI 的 init 命令",
            )
        })?;
    state.srv().role.assign_to_user(user.id, role.id).await?;

    Ok(Json(ApiResponse::ok(UserResp::from(user))))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    operation_id = "Auth__login",
    request_body = LoginReq,
    responses(
        (status = 200, description = "登录成功,设置会话 Cookie", body = ApiResponse<AuthStateResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 403, description = "用户名或密码错误"),
    ),
)]
pub async fn login(
    State(state): State<AppState>,
    Varser(payload): Varser<LoginReq>,
) -> Result<impl IntoResponse, ApiError> {
    let auth_user = state
        .srv()
        .auth
        .authenticate(&payload.username, &payload.password)
        .await?
        .ok_or_else(|| ApiError::forbidden("用户名或密码错误"))?;

    let session = state.srv().session();
    let session_id = session.start(auth_user.user.id).await?;

    let mut response = Json(ApiResponse::ok(AuthStateResp {
        user: UserResp::from(auth_user.user),
        permissions: auth_user.permissions,
    }))
    .into_response();
    response
        .headers_mut()
        .append(header::SET_COOKIE, session.set_cookie_value(&session_id));
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    operation_id = "Auth__me",
    responses(
        (status = 200, description = "当前用户", body = ApiResponse<AuthStateResp>),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn me(
    State(state): State<AppState>,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, ApiError> {
    let auth_user = state.srv().auth.get_auth_user(ctx.user_id).await?;

    Ok(Json(ApiResponse::ok(AuthStateResp {
        user: UserResp::from(auth_user.user),
        permissions: auth_user.permissions,
    })))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    operation_id = "Auth__logout",
    responses(
        (status = 200, description = "已退出", body = ApiResponse<MessageResp>),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn logout(
    State(state): State<AppState>,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, ApiError> {
    let session = state.srv().session();
    session.end_for_user(ctx.user_id).await?;

    let mut response = Json(ApiResponse::ok(MessageResp {
        message: "已退出登录".to_string(),
    }))
    .into_response();
    // 清除客户端 Cookie,避免浏览器此后每次请求携带死会话
    response
        .headers_mut()
        .append(header::SET_COOKIE, session.clear_cookie_value());
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/auth/jwt/login",
    tag = "auth",
    operation_id = "Auth__jwtLogin",
    request_body = LoginReq,
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<LoginResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 403, description = "用户名或密码错误"),
    ),
)]
pub async fn jwt_login(
    State(state): State<AppState>,
    Varser(payload): Varser<LoginReq>,
) -> Result<impl IntoResponse, ApiError> {
    let auth_user = state
        .srv()
        .auth
        .authenticate(&payload.username, &payload.password)
        .await?
        .ok_or_else(|| ApiError::forbidden("用户名或密码错误"))?;

    let rt = state.srv().runtime();
    let mut claims = HsClaims::new(auth_user.user.id, auth_user.user.username.clone());
    let pair = rt.tokens.issue(&mut claims, auth_user.user.id).await?;

    Ok(Json(ApiResponse::ok(LoginResp {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
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
    operation_id = "Auth__jwtRefresh",
    request_body = RefreshReq,
    responses(
        (status = 200, description = "新令牌对", body = ApiResponse<RefreshResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "刷新令牌无效或已过期"),
    ),
)]
pub async fn jwt_refresh(
    State(state): State<AppState>,
    Varser(payload): Varser<RefreshReq>,
) -> Result<impl IntoResponse, ApiError> {
    let rt = state.srv().runtime();
    let users = state.srv().user.clone();
    let pair = rt
        .tokens
        .rotate::<HsClaims, _, _>(&payload.refresh_token, move |user_id| async move {
            let user = users.get_by_id(user_id).await?;
            Ok(HsClaims::new(user.id, user.username))
        })
        .await?;

    Ok(Json(ApiResponse::ok(RefreshResp {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
    })))
}

#[utoipa::path(
    post,
    path = "/auth/jwt/logout",
    tag = "auth",
    operation_id = "Auth__jwtLogout",
    responses(
        (status = 200, description = "已撤销全部刷新令牌", body = ApiResponse<MessageResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_logout(
    State(state): State<AppState>,
    ctx: JwtCtx,
) -> Result<impl IntoResponse, ApiError> {
    state.srv().runtime().tokens.revoke_all(ctx.user_id).await?;

    Ok(Json(ApiResponse::ok(MessageResp {
        message: "已退出登录".to_string(),
    })))
}

#[utoipa::path(
    get,
    path = "/auth/jwt/me",
    tag = "auth",
    operation_id = "Auth__jwtMe",
    responses(
        (status = 200, description = "当前用户", body = ApiResponse<AuthStateResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_me(
    State(state): State<AppState>,
    ctx: JwtCtx,
) -> Result<impl IntoResponse, ApiError> {
    let auth_user = state.srv().auth.get_auth_user(ctx.user_id).await?;

    Ok(Json(ApiResponse::ok(AuthStateResp {
        user: UserResp::from(auth_user.user),
        permissions: auth_user.permissions,
    })))
}

#[utoipa::path(
    get,
    path = "/auth/jwt/echo",
    tag = "auth",
    operation_id = "Auth__jwtEcho",
    responses(
        (status = 200, description = "认证检查示例", body = ApiResponse<JwtEchoResp>),
        (status = 401, description = "令牌无效"),
    ),
)]
pub async fn jwt_echo(ctx: JwtCtx) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(ApiResponse::ok(JwtEchoResp {
        user_id: ctx.user_id,
        username: ctx.username().to_string(),
    })))
}
