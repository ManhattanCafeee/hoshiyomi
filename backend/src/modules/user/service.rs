use sqlx::MySqlPool;

use vivarium_rs::{
    ApiError, ErrorKind, Expr, Order, Page, Pagination, Query, Result, Sorter, Update, create,
    delete, find_by_id, hash, verify,
};

use super::models::{User, UserCol};

#[derive(Debug, Clone)]
pub struct UserService {
    pool: MySqlPool,
}

impl UserService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: String, email: String, password: String) -> Result<User> {
        if self.find_by_username(&username).await?.is_some() {
            return Err(ApiError::conflict("用户名已存在"));
        }
        if self.find_by_email(&email).await?.is_some() {
            return Err(ApiError::conflict("邮箱已存在"));
        }
        let hashed = hash(&password)?;
        let user = User {
            id: 0,
            username,
            email,
            password: hashed,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        // 先查后插的并发竞态:唯一键冲突由库的 From<sqlx::Error> 识别,这里换成接口文案
        let id = create(&self.pool, user)
            .await
            .map_err(|e| ApiError::conflict_from_db(e, "用户名或邮箱已存在"))?;
        find_by_id::<User, _>(&self.pool, id)
            .await?
            .ok_or_else(|| ApiError::new(ErrorKind::Internal, "用户创建后查询失败"))
    }

    pub async fn find_by_id(&self, id: u64) -> Result<Option<User>> {
        find_by_id::<User, _>(&self.pool, id)
            .await
            .map_err(ApiError::from)
    }

    pub async fn get_by_id(&self, id: u64) -> Result<User> {
        self.find_by_id(id)
            .await?
            .ok_or_else(|| ApiError::not_found("用户不存在"))
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        Query::<_, User>::new()
            .where_eq(UserCol::Username, username)
            .first(&self.pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        Query::<_, User>::new()
            .where_eq(UserCol::Email, email)
            .first(&self.pool)
            .await
            .map_err(ApiError::from)
    }

    /// 一页用户 + 总数。`paginate` 需要 `Copy` 的执行器,故传 `&pool`。
    pub async fn list_page(&self, page: u64, per_page: u64) -> Result<Page<User>> {
        let pagination = Pagination::new(
            page.min(u64::from(Pagination::MAX_PAGE)) as u32,
            per_page as u32,
        );
        Query::<_, User>::new()
            .order_by(Sorter::new(UserCol::Id, Order::Asc))
            .paginate(pagination, &self.pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn update_username(&self, id: u64, new_username: String) -> Result<User> {
        let user = self.get_by_id(id).await?;
        if new_username != user.username && self.find_by_username(&new_username).await?.is_some() {
            return Err(ApiError::conflict("用户名已存在"));
        }
        // 部分列更新:库的 update_by_id 会写全部非 id 列,故用 Update
        Update::<User, UserCol>::new(id)
            .set(UserCol::Username, new_username)
            .set_expr(UserCol::UpdatedAt, Expr::Now)
            .execute(&self.pool)
            .await
            .map_err(ApiError::from)?;
        self.get_by_id(id).await
    }

    pub async fn change_password(
        &self,
        id: u64,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let user = self.get_by_id(id).await?;
        if !verify(old_password, &user.password)? {
            return Err(ApiError::forbidden("旧密码错误"));
        }
        let hashed = hash(new_password)?;
        Update::<User, UserCol>::new(id)
            .set(UserCol::Password, hashed)
            .set_expr(UserCol::UpdatedAt, Expr::Now)
            .execute(&self.pool)
            .await
            .map_err(ApiError::from)?;
        Ok(())
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        delete::<User, _>(&self.pool, id)
            .await
            .map(|_| ())
            .map_err(ApiError::from)
    }
}
