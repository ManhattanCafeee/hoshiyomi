use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

pub const DEFAULT_ROLE_PERMISSIONS: &[(DefaultRole, &[Perm])] = &[
    (DefaultRole::Superuser, &[Perm::All]),
    (DefaultRole::Admin, &[Perm::UserAll, Perm::RoleAll]),
    (DefaultRole::User, &[Perm::UserRead]),
];

#[derive(Debug, Clone, Copy, EnumString, EnumIter, IntoStaticStr, Display)]
#[strum(serialize_all = "snake_case")]
pub enum DefaultRole {
    Superuser,
    Admin,
    User,
}

impl DefaultRole {
    pub fn name(&self) -> &'static str {
        self.into()
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Superuser => "超级用户",
            Self::Admin => "管理员",
            Self::User => "普通用户",
        }
    }

    pub fn all() -> Vec<DefaultRole> {
        Self::iter().collect()
    }

    pub fn default_permissions(&self) -> &[Perm] {
        DEFAULT_ROLE_PERMISSIONS
            .iter()
            .find(|(r, _)| r.name() == self.name())
            .map(|(_, perms)| *perms)
            .unwrap_or(&[])
    }
}

/// roles 表行:permissions 为 JSON 数组
///
/// 两个时间列标 `#[entity(skip)]`,理由同 `User`。
#[derive(Debug, sqlx::FromRow, vivarium_rs::Entity)]
#[entity(table = "roles")]
pub struct Role {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    #[entity(json)]
    pub permissions: sqlx::types::Json<Vec<Perm>>,
    #[entity(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[entity(skip)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// roles 表可寻址列
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleCol {
    Id,
    Name,
}

impl vivarium_rs::Column for RoleCol {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}

impl Role {
    pub fn perm_codes(&self) -> Vec<String> {
        self.permissions
            .iter()
            .map(|p| p.code().to_string())
            .collect()
    }

    pub fn parse_perms(&self) -> Vec<Perm> {
        self.permissions.clone().0
    }
}

/// 权限点:serde 与 strum 序列化码必须一致(见 tests::test_permissions_consistency)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumString,
    EnumIter,
    IntoStaticStr,
    Serialize,
    Deserialize,
    Display,
    utoipa::ToSchema,
)]
pub enum Perm {
    #[serde(rename = "*")]
    #[strum(serialize = "*")]
    All,

    #[serde(rename = "user:read")]
    #[strum(serialize = "user:read")]
    UserRead,
    #[serde(rename = "user:write")]
    #[strum(serialize = "user:write")]
    UserWrite,
    #[serde(rename = "user:delete")]
    #[strum(serialize = "user:delete")]
    UserDelete,
    #[serde(rename = "user:*")]
    #[strum(serialize = "user:*")]
    UserAll,

    #[serde(rename = "role:read")]
    #[strum(serialize = "role:read")]
    RoleRead,
    #[serde(rename = "role:write")]
    #[strum(serialize = "role:write")]
    RoleWrite,
    #[serde(rename = "role:delete")]
    #[strum(serialize = "role:delete")]
    RoleDelete,
    #[serde(rename = "role:*")]
    #[strum(serialize = "role:*")]
    RoleAll,
}

/// 权限码匹配:持有码 `*` 匹配一切;`前缀:*` 匹配同前缀的任意码。
///
/// 直接委托库的 `perms_match`(语义一致),本模块的单元测试即为等价性证明。
pub fn perms_match(self_code: &str, target_code: &str) -> bool {
    vivarium_rs::perms_match(self_code, target_code)
}

impl Perm {
    pub fn from_code(code: &str) -> Option<Self> {
        code.parse().ok()
    }

    pub fn code(&self) -> &'static str {
        self.into()
    }

    pub const fn description(&self) -> &'static str {
        match self {
            Self::All => "超级用户",
            Self::UserRead => "查看用户信息",
            Self::UserWrite => "创建/修改用户",
            Self::UserDelete => "删除用户",
            Self::UserAll => "用户管理所有权限",
            Self::RoleRead => "查看角色信息",
            Self::RoleWrite => "创建/修改角色",
            Self::RoleDelete => "删除角色",
            Self::RoleAll => "角色管理所有权限",
        }
    }

    pub fn matches(&self, target_code: &str) -> bool {
        perms_match(self.code(), target_code)
    }

    pub fn all() -> Vec<Perm> {
        Self::iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_consistency() {
        for perm in Perm::iter() {
            let serde_json_str = serde_json::to_string(&perm).expect("Serialize failed");
            let serde_val = serde_json_str.trim_matches('"');

            let strum_val: &'static str = perm.into();

            assert_eq!(serde_val, strum_val, "serde 与 strum 码不一致: {perm}");
        }
    }

    #[test]
    fn test_perms_match() {
        assert!(perms_match("*", "user:read"));
        assert!(perms_match("user:*", "user:delete"));
        assert!(perms_match("user:read", "user:read"));
        assert!(!perms_match("user:read", "user:write"));
        assert!(!perms_match("user:*", "role:read"));
    }
}
