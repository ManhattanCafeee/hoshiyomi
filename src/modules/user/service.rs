use sqlx::MySqlPool;

use crate::{
    error::{ErrorKind, OptionAppExt, Result},
    util::password,
};

use super::{models::User, repository};

#[derive(Debug, Clone)]
pub struct UserService {
    pool: MySqlPool,
}

impl UserService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: String, email: String, password: String) -> Result<User> {
        if repository::find_user_by_username(&self.pool, &username)
            .await?
            .is_some()
        {
            return Err(ErrorKind::AlreadyExists.msg("用户名已存在"));
        }
        if repository::find_user_by_email(&self.pool, &email)
            .await?
            .is_some()
        {
            return Err(ErrorKind::AlreadyExists.msg("邮箱已存在"));
        }
        let hashed = password::hash(&password)?;
        repository::insert_user(&self.pool, &username, &email, &hashed)
            .await
            .map_err(|e| crate::error::map_duplicate_key(e, "用户名或邮箱已存在"))?;
        repository::find_user_by_username(&self.pool, &username)
            .await?
            .ok_or_else(|| ErrorKind::Internal.msg("用户创建后查询失败"))
    }

    pub async fn find_by_id(&self, id: u64) -> Result<Option<User>> {
        repository::find_user_by_id(&self.pool, id).await
    }

    pub async fn get_by_id(&self, id: u64) -> Result<User> {
        repository::find_user_by_id(&self.pool, id)
            .await?
            .ok_or_err_msg(ErrorKind::NotFound, "用户不存在")
    }

    pub async fn list(&self, page: u64, per_page: u64) -> Result<Vec<User>> {
        let offset = page.saturating_sub(1).saturating_mul(per_page);
        repository::list_users(&self.pool, per_page, offset).await
    }

    pub async fn count(&self) -> Result<u64> {
        repository::count_users(&self.pool).await
    }

    pub async fn update_username(&self, id: u64, new_username: String) -> Result<User> {
        let user = self.get_by_id(id).await?;
        if new_username != user.username
            && repository::find_user_by_username(&self.pool, &new_username)
                .await?
                .is_some()
        {
            return Err(ErrorKind::AlreadyExists.msg("用户名已存在"));
        }
        repository::update_user_username(&self.pool, id, &new_username).await?;
        self.get_by_id(id).await
    }

    pub async fn change_password(
        &self,
        id: u64,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let user = self.get_by_id(id).await?;
        if !password::verify(old_password, &user.password)? {
            return Err(ErrorKind::InvalidCredentials.msg("旧密码错误"));
        }
        let hashed = password::hash(new_password)?;
        repository::update_user_password(&self.pool, id, &hashed).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        repository::delete_user(&self.pool, id).await
    }
}
