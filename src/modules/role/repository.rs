use sqlx::MySqlPool;

use super::models::Role;
use crate::error::{AppError, Result};

pub async fn find_role_by_name(pool: &MySqlPool, name: &str) -> Result<Option<Role>> {
    sqlx::query_as::<_, Role>(
        "SELECT id, name, description, permissions, created_at, updated_at FROM roles WHERE name = ?",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_role(
    pool: &MySqlPool,
    name: &str,
    description: Option<&str>,
    perms: &sqlx::types::Json<Vec<super::models::Perm>>,
) -> Result<()> {
    sqlx::query("INSERT INTO roles (name, description, permissions) VALUES (?, ?, ?)")
        .bind(name)
        .bind(description)
        .bind(perms)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn list_roles(pool: &MySqlPool) -> Result<Vec<Role>> {
    sqlx::query_as::<_, Role>(
        "SELECT id, name, description, permissions, created_at, updated_at FROM roles ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete_role_by_id(pool: &MySqlPool, id: u64) -> Result<()> {
    sqlx::query("DELETE FROM roles WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}

pub async fn find_roles_by_user(pool: &MySqlPool, user_id: u64) -> Result<Vec<Role>> {
    sqlx::query_as::<_, Role>(
        "SELECT r.id, r.name, r.description, r.permissions, r.created_at, r.updated_at
         FROM user_roles ur
         JOIN roles r ON ur.role_id = r.id
         WHERE ur.user_id = ?
         ORDER BY r.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_user_role(pool: &MySqlPool, user_id: u64, role_id: u64) -> Result<()> {
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::from)
}
