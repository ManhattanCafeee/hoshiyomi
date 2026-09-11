//! 9 个旧版平铺环境变量的兼容层。
//!
//! 它们在分层里优先级最高(在 `HOSHIYOMI__*` 之后 merge),保持现有 `.env` 写法有效。
//! 数字变量**先在这里解析成强类型**:自定义 provider 返回的 `Dict` 不会被 figment 展开或
//! 做 TOML 式解析,字符串塞进去无法反序列化成数字;顺带也保住了原有的中文错误文案。

use std::env;

use figment::value::{Dict, Map, Value};
use figment::{Metadata, Profile, Provider};
use vivarium_rs::{ApiError, ErrorKind, Result};

/// 9 个旧版平铺环境变量的解析结果(未设置的变量为 None)
pub struct LegacyOverrides {
    database_url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    rust_log: Option<String>,
    log_level: Option<String>,
    jwt_secret: Option<String>,
    jwt_expires_in_seconds: Option<u64>,
    session_cookie_name: Option<String>,
    session_ttl_hours: Option<u64>,
}

impl std::fmt::Debug for LegacyOverrides {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 连接串与 JWT 密钥含敏感值,只标注是否设置
        f.debug_struct("LegacyOverrides")
            .field("database_url", &self.database_url.is_some())
            .field("host", &self.host)
            .field("port", &self.port)
            .field("rust_log", &self.rust_log)
            .field("log_level", &self.log_level)
            .field("jwt_secret", &self.jwt_secret.is_some())
            .field("jwt_expires_in_seconds", &self.jwt_expires_in_seconds)
            .field("session_cookie_name", &self.session_cookie_name)
            .field("session_ttl_hours", &self.session_ttl_hours)
            .finish()
    }
}

impl LegacyOverrides {
    /// 从进程环境读取 9 个旧版变量,数字变量非法时返回中文错误
    pub fn from_env() -> Result<Self> {
        let var = |name: &str| env::var(name).ok();

        let port = match var("PORT") {
            Some(v) => Some(v.parse().map_err(|_| {
                ApiError::new(
                    ErrorKind::Internal,
                    format!("PORT 无效: {v:?},应为 1-65535 的数字"),
                )
            })?),
            None => None,
        };
        let jwt_expires_in_seconds = match var("JWT_EXPIRES_IN_SECONDS") {
            Some(v) => Some(v.parse().map_err(|_| {
                ApiError::new(
                    ErrorKind::Internal,
                    format!("JWT_EXPIRES_IN_SECONDS 无效: {v:?}"),
                )
            })?),
            None => None,
        };
        let session_ttl_hours = match var("SESSION_TTL_HOURS") {
            Some(v) => Some(v.parse().map_err(|_| {
                ApiError::new(
                    ErrorKind::Internal,
                    format!("SESSION_TTL_HOURS 无效: {v:?}"),
                )
            })?),
            None => None,
        };

        Ok(Self {
            database_url: var("DATABASE_URL"),
            host: var("HOST"),
            port,
            rust_log: var("RUST_LOG"),
            log_level: var("LOG_LEVEL"),
            jwt_secret: var("JWT_SECRET"),
            jwt_expires_in_seconds,
            session_cookie_name: var("SESSION_COOKIE_NAME"),
            session_ttl_hours,
        })
    }
}

/// 把 `LegacyOverrides` 适配成 figment provider 的包装
#[derive(Debug)]
pub struct LegacyFlatEnv {
    /// 已解析的旧版变量,作为最高优先级的最后一层 merge
    pub values: LegacyOverrides,
}

impl Provider for LegacyFlatEnv {
    fn metadata(&self) -> Metadata {
        Metadata::named("hoshiyomi legacy flat environment")
    }

    fn data(&self) -> std::result::Result<Map<Profile, Dict>, figment::Error> {
        let v = &self.values;
        let mut root = Dict::new();

        if let Some(url) = &v.database_url {
            let mut database = Dict::new();
            database.insert("url".to_owned(), Value::from(url.clone()));
            root.insert("database".to_owned(), Value::from(database));
        }

        let mut server = Dict::new();
        if let Some(host) = &v.host {
            server.insert("host".to_owned(), Value::from(host.clone()));
        }
        if let Some(port) = v.port {
            server.insert("port".to_owned(), Value::from(u64::from(port)));
        }
        if !server.is_empty() {
            root.insert("server".to_owned(), Value::from(server));
        }

        // RUST_LOG 先写、LOG_LEVEL 后写:两者同时设置时 LOG_LEVEL 生效(与原实现一致)
        let mut log = Dict::new();
        if let Some(level) = &v.rust_log {
            log.insert("level".to_owned(), Value::from(level.clone()));
        }
        if let Some(level) = &v.log_level {
            log.insert("level".to_owned(), Value::from(level.clone()));
        }
        if !log.is_empty() {
            root.insert("log".to_owned(), Value::from(log));
        }

        let mut jwt = Dict::new();
        if let Some(secret) = &v.jwt_secret {
            jwt.insert("secret".to_owned(), Value::from(secret.clone()));
        }
        if let Some(seconds) = v.jwt_expires_in_seconds {
            jwt.insert("expires_in_seconds".to_owned(), Value::from(seconds));
        }
        let mut session = Dict::new();
        if let Some(name) = &v.session_cookie_name {
            session.insert("cookie_name".to_owned(), Value::from(name.clone()));
        }
        if let Some(hours) = v.session_ttl_hours {
            session.insert("ttl_hours".to_owned(), Value::from(hours));
        }
        let mut auth = Dict::new();
        if !jwt.is_empty() {
            auth.insert("jwt".to_owned(), Value::from(jwt));
        }
        if !session.is_empty() {
            auth.insert("session".to_owned(), Value::from(session));
        }
        if !auth.is_empty() {
            root.insert("auth".to_owned(), Value::from(auth));
        }

        Ok(Profile::Default.collect(root))
    }
}
