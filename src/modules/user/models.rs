use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
