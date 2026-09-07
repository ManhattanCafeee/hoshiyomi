use axum::{
    extract::{Path, State},
    Json,
};

use crate::{common::response::ApiResponse, error::AppError, state::AppState};

use super::{models::User, repository};

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let user = repository::find_user_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("用户 id={id}")))?;
    Ok(Json(ApiResponse::success(user)))
}
