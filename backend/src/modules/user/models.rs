//! users 表的行模型与列枚举(对外 DTO 见 `dto.rs`)。

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
