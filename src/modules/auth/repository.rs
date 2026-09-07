use chrono::{DateTime, Utc};
use sqlx::MySqlPool;

use crate::error::{AppError, Result};

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub user_id: u64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RefreshTokenRow {
    pub id: u64,
    pub user_id: u64,
    pub expires_at: DateTime<Utc>,
}

pub async fn insert_session(
    pool: &MySqlPool,
    user_id: u64,
    session_id: &str,
    expires_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query("INSERT INTO sessions (user_id, session_id, expires_at) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(session_id)
        .bind(expires_at)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn find_session_by_token(
    pool: &MySqlPool,
    session_id: &str,
) -> Result<Option<SessionRow>> {
    sqlx::query_as::<_, SessionRow>(
        "SELECT user_id, expires_at, created_at FROM sessions WHERE session_id = ?",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update_session(
    pool: &MySqlPool,
    session_id: &str,
    expires_at: DateTime<Utc>,
    activity_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query("UPDATE sessions SET expires_at = ?, created_at = ? WHERE session_id = ?")
        .bind(expires_at)
        .bind(activity_at)
        .bind(session_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn delete_session_by_token(pool: &MySqlPool, session_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn delete_sessions_by_user(pool: &MySqlPool, user_id: u64) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn insert_refresh_token(
    pool: &MySqlPool,
    user_id: u64,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn find_refresh_token_by_token(
    pool: &MySqlPool,
    token: &str,
) -> Result<Option<RefreshTokenRow>> {
    sqlx::query_as::<_, RefreshTokenRow>(
        "SELECT id, user_id, expires_at FROM refresh_tokens WHERE token = ?",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete_refresh_token_by_id(pool: &MySqlPool, id: u64) -> Result<u64> {
    let result = sqlx::query("DELETE FROM refresh_tokens WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(result.rows_affected())
}

pub async fn delete_refresh_tokens_by_user(pool: &MySqlPool, user_id: u64) -> Result<()> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}
