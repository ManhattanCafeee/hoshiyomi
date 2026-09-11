//! 两个存储 trait 的 MySQL 实现。
//!
//! 库的会话中间件与刷新令牌管理器传进来的 `id`/`token_hash` **已经是** 64 位
//! 十六进制的 SHA-256 摘要（`hash_token(原始值)`），这里直接当键用，不要再哈希。

use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use vivarium_rs::{ApiError, RefreshTokenRecord, RefreshTokenStore, SessionRecord, SessionStore};

#[derive(Clone)]
pub struct MySqlSessionStore {
    pool: MySqlPool,
}

/// `sessions` 的四列投影(库的 `SessionRecord` 不含 `session_id`,故单独取)
#[derive(sqlx::FromRow)]
struct SessionColumns {
    user_id: u64,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

impl MySqlSessionStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl SessionStore for MySqlSessionStore {
    type UserId = u64;

    async fn create(
        &self,
        id: &str,
        user: u64,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        last_activity: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        sqlx::query(
            "INSERT INTO sessions (session_id, user_id, created_at, expires_at, last_activity)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(user)
        .bind(created_at)
        .bind(expires_at)
        .bind(last_activity)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(ApiError::database)
    }

    async fn find(&self, id: &str) -> Result<Option<SessionRecord<u64>>, ApiError> {
        let row: Option<SessionColumns> = sqlx::query_as(
            "SELECT user_id, created_at, expires_at, last_activity FROM sessions WHERE session_id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(ApiError::database)?;

        Ok(row.map(|row| SessionRecord {
            user_id: row.user_id,
            created_at: row.created_at,
            expires_at: row.expires_at,
            last_activity: row.last_activity,
        }))
    }

    async fn touch(
        &self,
        id: &str,
        expires_at: DateTime<Utc>,
        last_activity: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        sqlx::query("UPDATE sessions SET expires_at = ?, last_activity = ? WHERE session_id = ?")
            .bind(expires_at)
            .bind(last_activity)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(ApiError::database)
    }

    async fn remove(&self, id: &str) -> Result<bool, ApiError> {
        let result = sqlx::query("DELETE FROM sessions WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ApiError::database)?;
        Ok(result.rows_affected() == 1)
    }

    async fn remove_by_user(&self, user: u64) -> Result<u64, ApiError> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user)
            .execute(&self.pool)
            .await
            .map_err(ApiError::database)?;
        Ok(result.rows_affected())
    }
}

#[derive(Clone)]
pub struct MySqlRefreshTokenStore {
    pool: MySqlPool,
}

/// `refresh_tokens` 的两列投影
#[derive(sqlx::FromRow)]
struct RefreshTokenColumns {
    user_id: u64,
    expires_at: DateTime<Utc>,
}

impl MySqlRefreshTokenStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl RefreshTokenStore for MySqlRefreshTokenStore {
    type UserId = u64;

    async fn insert(
        &self,
        token_hash: &str,
        user: u64,
        expires_at: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        sqlx::query("INSERT INTO refresh_tokens (token_hash, user_id, expires_at) VALUES (?, ?, ?)")
            .bind(token_hash)
            .bind(user)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(ApiError::database)
    }

    async fn lookup(&self, token_hash: &str) -> Result<Option<RefreshTokenRecord<u64>>, ApiError> {
        let row: Option<RefreshTokenColumns> =
            sqlx::query_as("SELECT user_id, expires_at FROM refresh_tokens WHERE token_hash = ?")
                .bind(token_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(ApiError::database)?;

        Ok(row.map(|row| RefreshTokenRecord {
            user_id: row.user_id,
            expires_at: row.expires_at,
        }))
    }

    async fn remove(&self, token_hash: &str) -> Result<bool, ApiError> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(ApiError::database)?;
        // rows_affected 是单次使用的唯一保证:并发重放时后到者拿到 0 行
        Ok(result.rows_affected() == 1)
    }

    async fn remove_by_user(&self, user: u64) -> Result<u64, ApiError> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
            .bind(user)
            .execute(&self.pool)
            .await
            .map_err(ApiError::database)?;
        Ok(result.rows_affected())
    }
}
