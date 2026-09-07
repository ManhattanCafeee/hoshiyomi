use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::common::response::ApiResponse;

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    NotFound(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Db(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        };
        let body = ApiResponse::<()>::error(status.as_u16() as i32, message);
        (status, Json(body)).into_response()
    }
}
