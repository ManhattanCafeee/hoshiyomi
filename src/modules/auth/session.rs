use chrono::{DateTime, Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::Result;

use super::repository::{self, SessionRow};

#[derive(Debug, Clone)]
pub struct SessionService {
    pool: MySqlPool,
    ttl_hours: u64,
}

impl SessionService {
    pub fn new(pool: MySqlPool, ttl_hours: u64) -> Self {
        Self { pool, ttl_hours }
    }

    fn expires_at(&self) -> DateTime<Utc> {
        Utc::now() + Duration::hours(self.ttl_hours as i64)
    }

    /// 会话已过半 TTL 时自动延长(created_at 记录最近活动时间,随续期重置)
    pub fn should_extend(&self, session: &SessionRow) -> bool {
        let elapsed = Utc::now() - session.created_at;
        elapsed >= Duration::hours((self.ttl_hours / 2) as i64)
    }

    pub async fn create(&self, user_id: u64) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let expires_at = self.expires_at();
        repository::insert_session(&self.pool, user_id, &session_id, expires_at).await?;
        Ok(session_id)
    }

    pub async fn find(&self, session_id: &str) -> Result<Option<SessionRow>> {
        repository::find_session_by_token(&self.pool, session_id).await
    }

    /// 滑动续期:同步重置 expires_at 与活动时间,返回新过期时刻
    pub async fn extend(&self, session_id: &str) -> Result<DateTime<Utc>> {
        let expires_at = self.expires_at();
        let activity_at = Utc::now();
        repository::update_session(&self.pool, session_id, expires_at, activity_at).await?;
        Ok(expires_at)
    }

    pub async fn delete(&self, session_id: &str) -> Result<()> {
        repository::delete_session_by_token(&self.pool, session_id).await
    }

    pub async fn delete_by_user_id(&self, user_id: u64) -> Result<()> {
        repository::delete_sessions_by_user(&self.pool, user_id).await
    }
}
