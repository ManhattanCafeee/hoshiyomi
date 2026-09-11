//! auth 领域的内部模型:JWT 载荷(对外 DTO 见 `dto.rs`)。

use serde::{Deserialize, Serialize};

/// access token 的载荷:字段与原实现逐字一致(uniapp 用 `exp` 计算刷新时机)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsClaims {
    pub sub: u64,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

impl HsClaims {
    /// 只填主体与用户名;`exp`/`iat` 由 `RefreshTokenManager` 按 TTL 覆写
    pub fn new(user_id: u64, username: String) -> Self {
        Self {
            sub: user_id,
            username,
            exp: 0,
            iat: 0,
        }
    }
}

impl vivarium_rs::AccessClaims<u64> for HsClaims {
    fn subject(&self) -> u64 {
        self.sub
    }

    fn set_subject(&mut self, sub: u64) {
        self.sub = sub;
    }

    fn set_expiry(&mut self, exp: i64) {
        self.exp = exp;
    }

    fn set_issued_at(&mut self, iat: i64) {
        self.iat = iat;
    }
}
