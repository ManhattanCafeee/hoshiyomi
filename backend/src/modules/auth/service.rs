use sqlx::MySqlPool;

use crate::modules::{
    role::{models::Perm, service::RoleService},
    user::{models::User, service::UserService},
};
use vivarium_rs::{ApiError, PermissionSet, Result, verify_login};

#[derive(Debug)]
pub struct AuthUser {
    pub user: User,
    pub permissions: Vec<Perm>,
}

impl AuthUser {
    pub fn new(user: User, permissions: Vec<Perm>) -> Self {
        Self { user, permissions }
    }
}

#[derive(Debug, Clone)]
pub struct AuthService {
    pool: MySqlPool,
}

impl AuthService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password_str: &str,
    ) -> Result<Option<AuthUser>> {
        let Some(user) = UserService::new(self.pool.clone())
            .find_by_username(username)
            .await?
        else {
            // 用户不存在时也跑一次校验(库的 verify_login(None) 走内置 dummy 哈希),
            // 与「用户存在但密码错误」路径等时,防止按响应时间枚举用户名
            let _ = verify_login(password_str, None);
            return Ok(None);
        };

        if !verify_login(password_str, Some(user.password.as_str()))? {
            return Ok(None);
        }

        let permissions = self.get_user_permissions(user.id).await?;
        Ok(Some(AuthUser::new(user, permissions)))
    }

    pub async fn get_auth_user(&self, user_id: u64) -> Result<AuthUser> {
        let user = UserService::new(self.pool.clone())
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ApiError::not_found("用户不存在"))?;
        let permissions = self.get_user_permissions(user_id).await?;
        Ok(AuthUser::new(user, permissions))
    }

    pub async fn get_user_permissions(&self, user_id: u64) -> Result<Vec<Perm>> {
        RoleService::new(self.pool.clone())
            .get_user_permissions(user_id)
            .await
    }

    /// 权限不足时由库的 `PermissionSet::require` 产出 403(文案取 catalog `forbidden`)
    pub async fn require_permission(&self, user_id: u64, perm: Perm) -> Result<()> {
        let perms = self.get_user_permissions(user_id).await?;
        PermissionSet::new(perms.iter().map(Perm::code)).require(perm.code())
    }
}
