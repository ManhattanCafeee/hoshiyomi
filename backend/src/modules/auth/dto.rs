//! auth 领域的对外 DTO:请求体与响应体。
//!
//! JWT 载荷(`HsClaims`)属于内部模型,见 `models.rs`。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::modules::{role::models::Perm, user::dto::UserResp};

/// 注册请求体
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterReq {
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

/// 会话登录请求体
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginReq {
    #[validate(length(min = 1, max = 64, message = "用户名不能为空"))]
    pub username: String,
    #[validate(length(min = 1, max = 128, message = "密码不能为空"))]
    pub password: String,
}

/// 刷新令牌请求体
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshReq {
    #[validate(length(min = 1, message = "refresh_token 不能为空"))]
    pub refresh_token: String,
}

/// JWT 登录响应:令牌对与当前认证状态
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResp {
    pub access_token: String,
    pub refresh_token: String,
    pub state: AuthStateResp,
}

/// 刷新响应:轮换后的新令牌对
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResp {
    pub access_token: String,
    pub refresh_token: String,
}

/// 当前认证状态:用户信息与权限码
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthStateResp {
    pub user: UserResp,
    pub permissions: Vec<Perm>,
}

/// 通用消息响应体
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResp {
    pub message: String,
}

/// JWT 认证检查的响应体
#[derive(Debug, Serialize, ToSchema)]
pub struct JwtEchoResp {
    pub user_id: u64,
    pub username: String,
}

// 库的校验提取器(Varser/PathVarser/QueryVarser)要求显式的初始化钩子
impl vivarium_rs::Initializer for RegisterReq {}
impl vivarium_rs::Initializer for LoginReq {}
impl vivarium_rs::Initializer for RefreshReq {}
