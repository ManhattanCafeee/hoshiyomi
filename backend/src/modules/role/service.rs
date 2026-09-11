use sqlx::MySqlPool;

use vivarium_rs::{ApiError, ErrorKind, Order, Query, Result, Sorter, create, delete, find_by_id};

use super::models::{Perm, Role, RoleCol};
use super::repository;

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
        let existing = repository::find_roles_by_user(&self.pool, user_id).await?;
        if existing.iter().any(|r| r.id == role_id) {
            return Err(ApiError::conflict("角色已分配给该用户"));
        }
        repository::insert_user_role(&self.pool, user_id, role_id)
            .await
            .map_err(|e| ApiError::conflict_from_db(e, "角色已分配给该用户"))
    }

    pub async fn get_user_permissions(&self, user_id: u64) -> Result<Vec<Perm>> {
        let roles = repository::find_roles_by_user(&self.pool, user_id).await?;
        let mut perms: Vec<Perm> = roles.iter().flat_map(Role::parse_perms).collect();
        perms.sort();
        perms.dedup();
        Ok(perms)
    }
}
