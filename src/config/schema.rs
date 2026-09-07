use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct RawAppConfig {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LogConfig {
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct AuthConfig {
    pub session: SessionConfig,
    pub jwt: JwtConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session: SessionConfig::default(),
            jwt: JwtConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SessionConfig {
    // 显式 snake_case 重命名:config crate 将 HOSHIYOMI__AUTH__SESSION__COOKIE_NAME
    // 转为 auth.session.cookie_name,必须与字段名一致才能匹配(TOML 亦用 snake_case)
    #[serde(rename = "cookie_name")]
    pub cookie_name: String,
    #[serde(rename = "ttl_hours")]
    pub ttl_hours: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: "session_id".to_string(),
            ttl_hours: 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct JwtConfig {
    pub secret: String,
    #[serde(rename = "expires_in_seconds")]
    pub expires_in_seconds: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-me-in-production".to_string(),
            expires_in_seconds: 900,
        }
    }
}
