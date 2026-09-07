use std::sync::LazyLock;

use sqlx::MySqlPool;

use crate::{
    bail,
    error::{ErrorKind, Result},
    modules::{
        role::{models::Perm, service::RoleService},
        user::{models::User, repository},
    },
    util::password,
};

/// 用户名不存在时用于等时校验的固定哈希,消除用户枚举计时侧信道
static DUMMY_PASSWORD_HASH: LazyLock<String> =
    LazyLock::new(|| password::hash("hoshiyomi-timing-dummy").expect("dummy 密码哈希初始化失败"));

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
        let Some(user) = repository::find_user_by_username(&self.pool, username).await? else {
            // 与「用户存在但密码错误」路径等时,防止按响应时间枚举用户名
            let _ = password::verify(password_str, &DUMMY_PASSWORD_HASH);
            return Ok(None);
        };

        if !password::verify(password_str, &user.password)? {
            return Ok(None);
        }

        let permissions = self.get_user_permissions(user.id).await?;
        Ok(Some(AuthUser::new(user, permissions)))
    }

    pub async fn get_auth_user(&self, user_id: u64) -> Result<AuthUser> {
        let user = repository::find_user_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| ErrorKind::NotFound.msg("用户不存在"))?;
        let permissions = self.get_user_permissions(user_id).await?;
        Ok(AuthUser::new(user, permissions))
    }

    pub async fn get_user_permissions(&self, user_id: u64) -> Result<Vec<Perm>> {
        RoleService::new(self.pool.clone())
            .get_user_permissions(user_id)
            .await
    }

    pub async fn check_permission(&self, user_id: u64, perm: Perm) -> Result<bool> {
        let perms = self.get_user_permissions(user_id).await?;
        Ok(perms.iter().any(|p| p.matches(perm.code())))
    }

    pub async fn require_permission(&self, user_id: u64, perm: Perm) -> Result<()> {
        if !self.check_permission(user_id, perm).await? {
            bail!(ErrorKind::PermissionDenied, "权限不足");
        }
        Ok(())
    }
}
