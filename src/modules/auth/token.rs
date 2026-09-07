use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    error::{ErrorKind, Result, ResultExt},
    modules::user::{models::User, service::UserService},
};

use super::repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct RotatedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Clone)]
pub struct TokenService {
    pool: MySqlPool,
    encoding: EncodingKey,
    decoding: DecodingKey,
    expires_in_seconds: u64,
}

impl std::fmt::Debug for TokenService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenService")
            .field("pool", &self.pool)
            .field("expires_in_seconds", &self.expires_in_seconds)
            .finish()
    }
}

impl TokenService {
    pub fn new(pool: MySqlPool, jwt_secret: &str, expires_in_seconds: u64) -> Self {
        Self {
            pool,
            encoding: EncodingKey::from_secret(jwt_secret.as_ref()),
            decoding: DecodingKey::from_secret(jwt_secret.as_ref()),
            expires_in_seconds,
        }
    }

    pub fn encode_access_token(&self, user: &User) -> Result<String> {
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            exp: now + self.expires_in_seconds as usize,
            iat: now,
        };
        encode(&Header::default(), &claims, &self.encoding)
            .err_kind_msg(ErrorKind::Internal, "令牌生成失败")
    }

    pub fn decode_access_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding, &Validation::default())
            .map(|d| d.claims)
            .err_kind_msg(ErrorKind::Unauthorized, "无效的令牌")
    }

    pub async fn generate_refresh_token(&self, user_id: u64) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(30);
        repository::insert_refresh_token(&self.pool, user_id, &token, expires_at).await?;
        Ok(token)
    }

    pub async fn delete_all_refresh_tokens(&self, user_id: u64) -> Result<()> {
        repository::delete_refresh_tokens_by_user(&self.pool, user_id).await
    }

    pub async fn rotate_refresh_token(&self, refresh_token_str: &str) -> Result<RotatedTokens> {
        let stored = repository::find_refresh_token_by_token(&self.pool, refresh_token_str)
            .await?
            .ok_or_else(|| ErrorKind::Unauthorized.msg("无效的刷新令牌"))?;

        // 单次使用:先删后发。并发重放时后到的 DELETE 影响 0 行,判定为无效令牌。
        let deleted = repository::delete_refresh_token_by_id(&self.pool, stored.id).await?;
        if deleted != 1 {
            return Err(ErrorKind::Unauthorized.msg("无效的刷新令牌"));
        }

        if stored.expires_at < Utc::now() {
            return Err(ErrorKind::Unauthorized.msg("刷新令牌已过期"));
        }

        let user = UserService::new(self.pool.clone())
            .get_by_id(stored.user_id)
            .await?;
        let access_token = self.encode_access_token(&user)?;
        let refresh_token = self.generate_refresh_token(user.id).await?;

        Ok(RotatedTokens {
            access_token,
            refresh_token,
            user,
        })
    }
}
