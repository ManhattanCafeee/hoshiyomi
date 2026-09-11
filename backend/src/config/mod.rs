pub mod legacy;
pub mod meta;
pub mod paths;
pub mod schema;

use std::sync::Arc;

use vivarium_rs::{
    ApiError, Config, ConfigError, ConfigOptions, ConfigWatcher, ErrorKind, HandlerId, Result,
};

pub use meta::*;
pub use paths::Paths;
pub use schema::*;

use legacy::{LegacyFlatEnv, LegacyOverrides};

/// 分层配置的唯一来源(库的 `Config<T>` 薄包装)。
///
/// 优先级从低到高:代码默认值 → `config.toml`(可选,缺失不算错)→ `HOSHIYOMI__*`
/// (`__` 分隔嵌套)→ 9 个旧版平铺环境变量。
#[derive(Clone)]
pub struct AppConfig {
    inner: Arc<Config<RawAppConfig>>,
}

impl AppConfig {
    /// 只挂代码默认值的栈(测试用)
    pub fn new(config: RawAppConfig) -> Result<Self> {
        let inner = Config::load_with(ConfigOptions::new(Paths::config_file()).defaults(&config))
            .map_err(config_error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn load() -> Result<Self> {
        // 数字型 legacy 变量先解析:失败要报原有的中文文案,而不是 figment 的错误
        let legacy = LegacyOverrides::from_env()?;
        let inner = Config::load_with(
            ConfigOptions::new(Paths::config_file())
                .defaults(&RawAppConfig::default())
                .file()
                .env_prefixed(ENV_PREFIX)
                .separator("__")
                .merge(LegacyFlatEnv { values: legacy }),
        )
        .map_err(config_error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// 当前配置快照(热重载后返回新一代)
    pub fn get(&self) -> Arc<RawAppConfig> {
        self.inner.get()
    }

    /// 监听配置文件。返回的 watcher 必须活到进程结束:drop 即停止监听。
    pub fn watch(&self) -> Result<ConfigWatcher> {
        Arc::clone(&self.inner).watch().map_err(config_error)
    }

    pub fn register<F>(&self, handler: F) -> HandlerId
    where
        F: Fn(&RawAppConfig) + Send + Sync + 'static,
    {
        self.inner.register(handler)
    }

    pub fn on_error<F>(&self, handler: F) -> HandlerId
    where
        F: Fn(&ConfigError) + Send + Sync + 'static,
    {
        self.inner.on_error(handler)
    }
}

fn config_error(err: ConfigError) -> ApiError {
    ApiError::new(ErrorKind::Internal, "配置错误").with_source(err)
}
