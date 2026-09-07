pub mod meta;
pub mod paths;
pub mod schema;

use std::{env, sync::Arc};

use config::{Environment, File};
use derive_more::Deref;
pub use meta::*;
pub use paths::Paths;
pub use schema::*;

use crate::error::{ErrorKind, Result};

#[derive(Debug, Clone, Deref)]
pub struct AppConfig {
    inner: Arc<RawAppConfig>,
}

impl AppConfig {
    pub fn new(config: RawAppConfig) -> Self {
        Self {
            inner: Arc::new(config),
        }
    }

    pub fn load() -> Result<Self> {
        load_raw().map(Self::new)
    }
}

pub fn load_raw() -> Result<RawAppConfig> {
    let mut raw: RawAppConfig = config::Config::builder()
        .add_source(config::Config::try_from(&RawAppConfig::default())?)
        .add_source(File::from(Paths::config_file().as_path()).required(false))
        .add_source(Environment::with_prefix(ENV_PREFIX).separator("__"))
        .build()?
        .try_deserialize()?;

    // 旧版裸环境变量兼容层:在 deserialize 之后逐项覆盖,保持 .env 现有写法有效(若同时设了 HOSHIYOMI__ 前缀变量,旧名优先,按本层覆盖顺序)
    if let Ok(v) = env::var("DATABASE_URL") {
        raw.database.url = v;
    }
    if let Ok(v) = env::var("HOST") {
        raw.server.host = v;
    }
    if let Ok(v) = env::var("PORT") {
        raw.server.port = v
            .parse()
            .map_err(|_| ErrorKind::Config.msg(format!("PORT 无效: {v:?},应为 1-65535 的数字")))?;
    }
    if let Ok(v) = env::var("RUST_LOG") {
        raw.log.level = v;
    }
    if let Ok(v) = env::var("LOG_LEVEL") {
        raw.log.level = v;
    }
    if let Ok(v) = env::var("JWT_SECRET") {
        raw.auth.jwt.secret = v;
    }
    if let Ok(v) = env::var("JWT_EXPIRES_IN_SECONDS") {
        raw.auth.jwt.expires_in_seconds = v
            .parse()
            .map_err(|_| ErrorKind::Config.msg(format!("JWT_EXPIRES_IN_SECONDS 无效: {v:?}")))?;
    }
    if let Ok(v) = env::var("SESSION_COOKIE_NAME") {
        raw.auth.session.cookie_name = v;
    }
    if let Ok(v) = env::var("SESSION_TTL_HOURS") {
        raw.auth.session.ttl_hours = v
            .parse()
            .map_err(|_| ErrorKind::Config.msg(format!("SESSION_TTL_HOURS 无效: {v:?}")))?;
    }
    Ok(raw)
}
