pub mod common;
pub mod config;
pub mod error;
pub mod modules;
pub mod state;

use axum::Router;
use state::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", modules::router())
        .with_state(state)
}
