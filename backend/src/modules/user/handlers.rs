use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    modules::{
        auth::{dto::MessageResp, extractor::SessionCtx},
        role::models::Perm,
    },
    pagination::PageData,
    state::AppState,
};
use vivarium_rs::{ApiError, ApiResponse, PathVarser, QueryVarser, Varser};

use super::dto::{
    ChangePasswordReq, CreateUserReq, IdPath, PaginationReq, UpdateUsernameReq, UserResp,
};

#[utoipa::path(
    get,
    path = "/users",
    tag = "user",
    operation_id = "User__list",
    params(
        ("page" = Option<u64>, Query, description = "页码,从 1 开始"),
        ("per_page" = Option<u64>, Query, description = "每页条数,1-100"),
    ),
    responses(
        (status = 200, description = "用户分页列表", body = ApiResponse<PageData<UserResp>>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "权限不足"),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    ctx: SessionCtx,
    QueryVarser(pagination): QueryVarser<PaginationReq>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .srv()
        .auth
        .require_permission(ctx.user_id, Perm::UserRead)
        .await?;

    let page = pagination.page;
    let per_page = pagination.per_page;

    let page_data = state.srv().user.list_page(page, per_page).await?;

    let items = page_data.items.into_iter().map(UserResp::from).collect();

    Ok(Json(ApiResponse::ok(PageData {
        items,
        total: page_data.total,
        // 回显请求值:库的 Pagination 会把 page 归一化到 1..=1_000_000,回显归一化值会让
        // 超范围请求的响应与请求不自洽(HEAD 的行为是逐字回显请求值)
        page,
        per_page,
    })))
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "user",
    operation_id = "User__create",
    responses(
        (status = 200, description = "创建的用户", body = ApiResponse<UserResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "权限不足"),
        (status = 409, description = "用户名或邮箱已存在"),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    ctx: SessionCtx,
    Varser(payload): Varser<CreateUserReq>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .srv()
        .auth
        .require_permission(ctx.user_id, Perm::UserWrite)
        .await?;

    let user = state
        .srv()
        .user
        .create(payload.username, payload.email, payload.password)
        .await?;

    Ok(Json(ApiResponse::ok(UserResp::from(user))))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "user",
    operation_id = "User__get",
    params(
        ("id" = u64, Path, description = "用户 ID"),
    ),
    responses(
        (status = 200, description = "用户信息", body = ApiResponse<UserResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "权限不足"),
        (status = 404, description = "用户不存在"),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    PathVarser(IdPath { id }): PathVarser<IdPath>,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, ApiError> {
    state
        .srv()
        .auth
        .require_permission(ctx.user_id, Perm::UserRead)
        .await?;

    let user = state.srv().user.get_by_id(id).await?;

    Ok(Json(ApiResponse::ok(UserResp::from(user))))
}

#[utoipa::path(
    put,
    path = "/users/{id}/username",
    tag = "user",
    operation_id = "User__updateUsername",
    params(
        ("id" = u64, Path, description = "用户 ID"),
    ),
    request_body = UpdateUsernameReq,
    responses(
        (status = 200, description = "更新后的用户", body = ApiResponse<UserResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "权限不足"),
        (status = 404, description = "用户不存在"),
        (status = 409, description = "用户名已存在"),
    ),
)]
pub async fn update_username(
    State(state): State<AppState>,
    PathVarser(IdPath { id }): PathVarser<IdPath>,
    ctx: SessionCtx,
    Varser(payload): Varser<UpdateUsernameReq>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .srv()
        .auth
        .require_permission(ctx.user_id, Perm::UserWrite)
        .await?;

    let user = state
        .srv()
        .user
        .update_username(id, payload.username)
        .await?;

    Ok(Json(ApiResponse::ok(UserResp::from(user))))
}

#[utoipa::path(
    put,
    path = "/users/{id}/password",
    tag = "user",
    operation_id = "User__changePassword",
    params(
        ("id" = u64, Path, description = "用户 ID"),
    ),
    request_body = ChangePasswordReq,
    responses(
        (status = 200, description = "修改结果", body = ApiResponse<MessageResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "无权修改他人密码或旧密码错误"),
        (status = 404, description = "用户不存在"),
    ),
)]
pub async fn change_password(
    State(state): State<AppState>,
    PathVarser(IdPath { id }): PathVarser<IdPath>,
    ctx: SessionCtx,
    Varser(payload): Varser<ChangePasswordReq>,
) -> Result<impl IntoResponse, ApiError> {
    if ctx.user_id != id {
        return Err(ApiError::forbidden("不能修改他人密码"));
    }

    state
        .srv()
        .user
        .change_password(id, &payload.old_password, &payload.new_password)
        .await?;

    Ok(Json(ApiResponse::ok(MessageResp {
        message: "密码修改成功".to_string(),
    })))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "user",
    operation_id = "User__delete",
    params(
        ("id" = u64, Path, description = "用户 ID"),
    ),
    responses(
        (status = 200, description = "删除结果", body = ApiResponse<MessageResp>),
        (status = 400, description = "参数或校验错误"),
        (status = 401, description = "未登录"),
        (status = 403, description = "权限不足"),
        (status = 404, description = "用户不存在"),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    PathVarser(IdPath { id }): PathVarser<IdPath>,
    ctx: SessionCtx,
) -> Result<impl IntoResponse, ApiError> {
    state
        .srv()
        .auth
        .require_permission(ctx.user_id, Perm::UserDelete)
        .await?;

    state.srv().user.delete(id).await?;

    Ok(Json(ApiResponse::ok(MessageResp {
        message: "用户已删除".to_string(),
    })))
}
