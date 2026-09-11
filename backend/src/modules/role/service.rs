use sqlx::MySqlPool;

use vivarium_rs::{ApiError, ErrorKind, Order, Query, Result, Sorter, create, delete, find_by_id};

use super::models::{Perm, Role, RoleCol};

#[derive(Debug, Clone)]
pub struct RoleService {
    pool: MySqlPool,
}

impl RoleService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        perms: &[Perm],
    ) -> Result<Role> {
        if self.find_by_name(&name).await?.is_some() {
            return Err(ApiError::conflict("角色已存在"));
        }
        let role = Role {
            id: 0,
            name,
            description,
            permissions: sqlx::types::Json(perms.to_vec()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        // 先查后插的并发竞态:唯一键冲突由库的 From<sqlx::Error> 识别,这里换成接口文案
        let id = create(&self.pool, role)
            .await
            .map_err(|e| ApiError::conflict_from_db(e, "角色已存在"))?;
        self.get_by_id(id)
            .await?
            .ok_or_else(|| ApiError::new(ErrorKind::Internal, "角色创建后查询失败"))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>> {
        Query::<_, Role>::new()
            .where_eq(RoleCol::Name, name)
            .first(&self.pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn get_by_id(&self, id: u64) -> Result<Option<Role>> {
        find_by_id::<Role, _>(&self.pool, id)
            .await
            .map_err(ApiError::from)
    }

    pub async fn list_all(&self) -> Result<Vec<Role>> {
        Query::<_, Role>::new()
            .order_by(Sorter::new(RoleCol::Id, Order::Asc))
            .find(&self.pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        delete::<Role, _>(&self.pool, id)
            .await
            .map(|_| ())
            .map_err(ApiError::from)
    }

    pub async fn assign_to_user(&self, user_id: u64, role_id: u64) -> Result<()> {
        let existing = self.find_roles_by_user(user_id).await?;
        if existing.iter().any(|r| r.id == role_id) {
            return Err(ApiError::conflict("角色已分配给该用户"));
        }
        insert_user_role(&self.pool, user_id, role_id)
            .await
            .map_err(|e| ApiError::conflict_from_db(e, "角色已分配给该用户"))
    }

    pub async fn get_user_permissions(&self, user_id: u64) -> Result<Vec<Perm>> {
        let roles = self.find_roles_by_user(user_id).await?;
        let mut perms: Vec<Perm> = roles.iter().flat_map(|r| r.parse_perms()).collect();
        perms.sort();
        perms.dedup();
        Ok(perms)
    }

    /// user_roles × roles 的 JOIN:库的 Query 只能 `FROM` 单表,无 JOIN 表达能力,故保留原生 sqlx。
    async fn find_roles_by_user(&self, user_id: u64) -> Result<Vec<Role>> {
        sqlx::query_as::<_, Role>(
            "SELECT r.id, r.name, r.description, r.permissions, r.created_at, r.updated_at
             FROM user_roles ur
             JOIN roles r ON ur.role_id = r.id
             WHERE ur.user_id = ?
             ORDER BY r.id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(ApiError::from)
    }
}

/// user_roles 全库仅此一条写入,不值得为它新建实体与 Column 枚举,保留原生 sqlx。
///
/// 返回原始 `sqlx::Error`:调用方要据此识别唯一键冲突(1062)并换成本接口文案。
async fn insert_user_role(
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
