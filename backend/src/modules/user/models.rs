use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// users 表行模型(password 不对外序列化,响应一律走 UserResp)
///
/// 两个时间列标 `#[entity(skip)]`:它们由 DDL 的 `DEFAULT (UTC_TIMESTAMP())` 生成,
/// 而库的 `create` 会写全部非 id、非 skip 列,不 skip 会用占位值覆盖默认值。
#[derive(Clone, sqlx::FromRow, vivarium_rs::Entity)]
#[entity(table = "users")]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub password: String,
    #[entity(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[entity(skip)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // password 是口令散列,不得经 `{:?}` 进日志(库对 VerifyOutcome 也做同样的脱敏)
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password", &"<redacted>")
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// users 表可寻址列(库的 Query/Update 需要显式列名)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCol {
    Id,
    Username,
    Email,
    Password,
    UpdatedAt,
}

impl vivarium_rs::Column for UserCol {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Username => "username",
            Self::Email => "email",
            Self::Password => "password",
            Self::UpdatedAt => "updated_at",
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct UserResp {
    pub id: u64,
    pub username: String,
    pub email: String,
    #[schema(value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResp {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserReq {
    #[validate(length(min = 3, max = 20, message = "用户名长度需在 3-20 之间"))]
    pub username: String,
    #[validate(
        email(message = "邮箱格式无效"),
        length(max = 255, message = "邮箱过长")
    )]
    pub email: String,
    #[validate(length(min = 8, max = 32, message = "密码长度需在 8-32 之间"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUsernameReq {
    #[validate(length(min = 3, max = 20, message = "用户名长度需在 3-20 之间"))]
    pub username: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordReq {
    #[validate(length(min = 1, max = 128, message = "旧密码长度需在 1-128 之间"))]
    pub old_password: String,
    #[validate(length(min = 8, max = 32, message = "新密码长度需在 8-32 之间"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(default)]
pub struct PaginationReq {
    #[validate(range(min = 1, message = "page 需 >= 1"))]
    pub page: u64,
    #[validate(range(min = 1, max = 100, message = "per_page 需在 1-100 之间"))]
    pub per_page: u64,
}

impl Default for PaginationReq {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 10,
        }
    }
}

/// 路径 ID 提取:反序列化失败带中文报错,保证在认证提取器之前返回 400
#[derive(Debug, Deserialize, Validate)]
pub struct IdPath {
    #[serde(deserialize_with = "deserialize_u64_id")]
    pub id: u64,
}

fn deserialize_u64_id<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let raw = String::deserialize(d)?;
    raw.parse()
        .map_err(|_| serde::de::Error::custom(format!("用户 id 无效: {raw}")))
}
// 库的校验提取器(Varser/PathVarser/QueryVarser)要求显式的初始化钩子
impl vivarium_rs::Initializer for IdPath {}
impl vivarium_rs::Initializer for CreateUserReq {}
impl vivarium_rs::Initializer for UpdateUsernameReq {}
impl vivarium_rs::Initializer for ChangePasswordReq {}
impl vivarium_rs::Initializer for PaginationReq {}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Debug` 不得打印口令散列。
    #[test]
    fn debug_redacts_password_hash() {
        let user = User {
            id: 1,
            username: "alice".to_owned(),
            email: "alice@example.com".to_owned(),
            password: "$argon2id$v=19$m=19456,t=2,p=1$hash".to_owned(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let printed = format!("{user:?}");

        assert!(printed.contains("alice"), "用户名应可打印: {printed}");
        assert!(
            !printed.contains("$argon2id$"),
            "口令散列不应出现在 Debug 输出: {printed}"
        );
    }
}
