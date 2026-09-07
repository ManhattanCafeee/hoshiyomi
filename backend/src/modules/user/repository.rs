use sqlx::MySqlPool;

use super::models::User;
use crate::error::{AppError, Result};

pub async fn find_user_by_id(pool: &MySqlPool, id: u64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, created_at, updated_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn find_user_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, created_at, updated_at FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, created_at, updated_at FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_user(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO users (username, email, password) VALUES (?, ?, ?)")
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn list_users(pool: &MySqlPool, limit: u64, offset: u64) -> Result<Vec<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, created_at, updated_at FROM users ORDER BY id ASC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn count_users(pool: &MySqlPool) -> Result<u64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(count as u64)
}

pub async fn update_user_username(pool: &MySqlPool, id: u64, username: &str) -> Result<()> {
    sqlx::query("UPDATE users SET username = ?, updated_at = UTC_TIMESTAMP() WHERE id = ?")
        .bind(username)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn update_user_password(pool: &MySqlPool, id: u64, hash: &str) -> Result<()> {
    sqlx::query("UPDATE users SET password = ?, updated_at = UTC_TIMESTAMP() WHERE id = ?")
        .bind(hash)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn delete_user(pool: &MySqlPool, id: u64) -> Result<()> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}
