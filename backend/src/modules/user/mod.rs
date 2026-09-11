pub mod handlers;
pub mod models;
pub mod service;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes![handlers::list])
        .routes(routes![handlers::create])
        .routes(routes![handlers::get])
        .routes(routes![handlers::update_username])
        .routes(routes![handlers::change_password])
        .routes(routes![handlers::delete])
}
