pub mod extractor;
pub mod handlers;
pub mod models;
pub mod service;
pub mod stores;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes![handlers::register])
        .routes(routes![handlers::login])
        .routes(routes![handlers::me])
        .routes(routes![handlers::logout])
        .routes(routes![handlers::jwt_login])
        .routes(routes![handlers::jwt_refresh])
        .routes(routes![handlers::jwt_logout])
        .routes(routes![handlers::jwt_me])
        .routes(routes![handlers::jwt_echo])
}
