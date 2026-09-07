pub mod user;

use axum::{routing::get, Json, Router};

use crate::{common::response::ApiResponse, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(user::router())
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}
