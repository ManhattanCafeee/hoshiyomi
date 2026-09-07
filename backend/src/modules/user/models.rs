use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// users 表行模型(password 不对外序列化,响应一律走 UserResp)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
