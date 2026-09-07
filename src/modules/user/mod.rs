pub mod handlers;
pub mod models;
pub mod repository;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/users/{id}", get(handlers::get_user))
}
