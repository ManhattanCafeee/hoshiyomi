//! role 领域的数据访问:两条不必走库的原生 SQL。
//!
//! - `find_roles_by_user`:库的 `Query<T>` 只能 `FROM` 单表,无 JOIN 表达能力;
//! - `insert_user_role`:`user_roles` 全库仅此一处写入,不值得为它新建实体与 `Column` 枚举。
//!
//! 其余 CRUD 一律走库的 `create`/`Query`/`delete`。

use sqlx::MySqlPool;

use vivarium_rs::{ApiError, Result};

use super::models::Role;

/// `user_roles × roles` 的 JOIN:该用户的全部角色,按 id 升序(零角色返回空 vec)。
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
    .map_err(ApiError::from)
}

/// 写入一条用户-角色关联。
///
/// `user_roles` 全库仅此一处写入,不值得为它新建实体与 `Column` 枚举,故保留原生 sqlx。
/// 返回原始 `sqlx::Error`:调用方要据此识别唯一键冲突(1062)并换成本接口文案。
pub async fn insert_user_role(
    pool: &MySqlPool,
    user_id: u64,
    role_id: u64,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .map(|_| ())
}
