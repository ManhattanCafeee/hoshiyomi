//! user 领域的对外 DTO:请求体、响应体与提取器载荷。
//!
//! 命名约定:`*Req` 请求体、`*Resp` 响应体;行模型与列枚举见 `models.rs`。

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use super::models::User;

/// 用户响应体(不含口令散列)
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

/// 创建用户请求体
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

/// 修改用户名请求体
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUsernameReq {
    #[validate(length(min = 3, max = 20, message = "用户名长度需在 3-20 之间"))]
    pub username: String,
}

/// 修改密码请求体
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordReq {
    #[validate(length(min = 1, max = 128, message = "旧密码长度需在 1-128 之间"))]
    pub old_password: String,
    #[validate(length(min = 8, max = 32, message = "新密码长度需在 8-32 之间"))]
    pub new_password: String,
}

/// 分页查询参数(缺省 page=1、per_page=10)
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
