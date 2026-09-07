use sqlx::MySqlPool;

use crate::error::{ErrorKind, Result};

use super::{
    models::{Perm, Role},
    repository,
};

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
        if repository::find_role_by_name(&self.pool, &name)
            .await?
            .is_some()
        {
            return Err(ErrorKind::AlreadyExists.msg("角色已存在"));
        }
        repository::insert_role(
            &self.pool,
            &name,
            description.as_deref(),
            &sqlx::types::Json(perms.to_vec()),
        )
        .await
        .map_err(|e| crate::error::map_duplicate_key(e, "角色已存在"))?;
        repository::find_role_by_name(&self.pool, &name)
            .await?
            .ok_or_else(|| ErrorKind::Internal.msg("角色创建后查询失败"))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>> {
        repository::find_role_by_name(&self.pool, name).await
    }

    pub async fn list_all(&self) -> Result<Vec<Role>> {
        repository::list_roles(&self.pool).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        repository::delete_role_by_id(&self.pool, id).await
    }

    pub async fn assign_to_user(&self, user_id: u64, role_id: u64) -> Result<()> {
        let existing = repository::find_roles_by_user(&self.pool, user_id).await?;
        if existing.iter().any(|r| r.id == role_id) {
            return Err(ErrorKind::AlreadyExists.msg("角色已分配给该用户"));
        }
        repository::insert_user_role(&self.pool, user_id, role_id)
            .await
            .map_err(|e| crate::error::map_duplicate_key(e, "角色已分配给该用户"))
    }

    pub async fn get_user_permissions(&self, user_id: u64) -> Result<Vec<Perm>> {
        let roles = repository::find_roles_by_user(&self.pool, user_id).await?;
        let mut perms: Vec<Perm> = roles.iter().flat_map(|r| r.parse_perms()).collect();
        perms.sort();
        perms.dedup();
        Ok(perms)
    }
}
