use serde::{Deserialize, Serialize};

/// 原始配置树,聚合 server/log/database/auth 四组配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct RawAppConfig {
    /// HTTP 服务监听配置(默认 0.0.0.0:8080)
    pub server: ServerConfig,
    /// 日志级别配置(默认 info)
    pub log: LogConfig,
    /// 数据库连接配置
    pub database: DatabaseConfig,
    /// 认证配置:会话与 JWT
    pub auth: AuthConfig,
}

/// HTTP 服务监听配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    /// 监听地址,默认 0.0.0.0
    pub host: String,
    /// 监听端口,默认 8080
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

/// 日志级别配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LogConfig {
    /// 日志过滤级别,默认 info
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// 数据库连接配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DatabaseConfig {
    /// MySQL 连接串,默认指向本机 hoshiyomi 库
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi".to_string(),
        }
    }
}

/// 认证配置:会话与 JWT 两组旋钮
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct AuthConfig {
    /// 会话 Cookie 配置
    pub session: SessionConfig,
    /// JWT 配置
    pub jwt: JwtConfig,
}

/// 会话 Cookie 配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SessionConfig {
    // 显式 snake_case 重命名:config crate 将 HOSHIYOMI__AUTH__SESSION__COOKIE_NAME
    // 转为 auth.session.cookie_name,必须与字段名一致才能匹配(TOML 亦用 snake_case)
    /// 会话 Cookie 名,默认 session_id
    #[serde(rename = "cookie_name")]
    pub cookie_name: String,
    /// 会话有效期(小时),默认 24
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

/// JWT 配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct JwtConfig {
    /// 签名密钥,默认 change-me-in-production(仅开发用)
    pub secret: String,
    /// 访问令牌有效期(秒),默认 900
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
