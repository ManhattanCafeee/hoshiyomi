use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::modules::{role::models::Perm, user::models::UserResp};

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

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginReq {
    #[validate(length(min = 1, max = 64, message = "用户名不能为空"))]
    pub username: String,
    #[validate(length(min = 1, max = 128, message = "密码不能为空"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshReq {
    #[validate(length(min = 1, message = "refresh_token 不能为空"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResp {
    pub access_token: String,
    pub refresh_token: String,
    pub state: AuthStateResp,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResp {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthStateResp {
    pub user: UserResp,
    pub permissions: Vec<Perm>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResp {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JwtEchoResp {
    pub user_id: u64,
    pub username: String,
}
