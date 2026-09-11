pub mod auth;
pub mod role;
pub mod user;

use axum::Json;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{common::response::ApiResponse, state::AppState};

/// /api/v1 下的所有路由(OpenAPI 路径前缀由 build_app 嵌套补全)
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes![health])
        .merge(user::router())
        .merge(auth::router())
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "Health__check",
    responses(
        (status = 200, description = "健康检查", body = ApiResponse<String>),
    ),
)]
async fn health() -> Json<ApiResponse<String>> {
    Json(ApiResponse::ok("ok".to_string()))
}
