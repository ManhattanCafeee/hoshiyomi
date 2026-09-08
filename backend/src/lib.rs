pub mod cli;
pub mod common;
pub mod config;
pub mod db;
pub mod error;
pub mod infra;
pub mod middleware;
pub mod modules;
pub mod serve;
pub mod state;
pub mod util;

pub use error::{AppError, ErrorKind, Result};

use axum::{Json, Router, http::StatusCode};
use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use common::response::ApiResponse;
use state::AppState;

pub fn build_app(state: AppState) -> Router {
    let (router, api) = OpenApiRouter::new()
        .nest("/api/v1", modules::router())
        .split_for_parts();

    router
        .merge(Scalar::with_url("/api-docs/scalar", api.clone()))
        .merge(SwaggerUi::new("/api-docs/swagger-ui").url("/api-docs/openapi.json", api))
        // fallback 必须先于 layer 注册,否则 404 响应不经过 CORS/Trace 中间件
        .fallback(not_found)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::session::refresh_session_cookie,
        ))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO))
                        .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
                )
                .layer(middleware::cors::cors()),
        )
        .with_state(state)
}

async fn not_found() -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::error(404, "未找到")),
    )
}
