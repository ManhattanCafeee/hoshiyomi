use sqlx::MySqlPool;

use super::models::User;

pub async fn find_user_by_id(pool: &MySqlPool, id: u64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, username, email, created_at FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
