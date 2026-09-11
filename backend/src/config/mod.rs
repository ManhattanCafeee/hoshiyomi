/// 旧版平铺环境变量的 figment 兼容层
pub mod legacy;
/// 项目元信息常量(名称与环境变量前缀)
pub mod meta;
/// 配置/数据/日志目录解析
pub mod paths;
/// 配置字段树:server/log/database/auth
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

    /// 按「代码默认 → config.toml → HOSHIYOMI__* → 旧版平铺变量」分层加载
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

    /// 注册配置变更处理器,热重载成功后按新快照回调
    pub fn register<F>(&self, handler: F) -> HandlerId
    where
        F: Fn(&RawAppConfig) + Send + Sync + 'static,
    {
        self.inner.register(handler)
    }

    /// 注册重载失败处理器,失败时保留旧值并上报
    pub fn on_error<F>(&self, handler: F) -> HandlerId
    where
        F: Fn(&ConfigError) + Send + Sync + 'static,
    {
        self.inner.on_error(handler)
    }
}

impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 配置含 JWT 密钥等敏感值:只输出类型名与 `..`,不打印任何字段
        f.debug_struct("AppConfig").finish_non_exhaustive()
    }
}

fn config_error(err: ConfigError) -> ApiError {
    ApiError::new(ErrorKind::Internal, "配置错误").with_source(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Debug` 不得泄露配置中的敏感值(连接串、JWT 密钥)。
    #[test]
    fn debug_redacts_sensitive_values() {
        let config = AppConfig::new(RawAppConfig::default()).expect("配置构造失败");
        let snapshot = config.get();
        let printed = format!("{config:?}");

        assert!(
            !printed.contains(&snapshot.database.url),
            "Debug 不应打印连接串: {printed}"
        );
        assert!(
            !printed.contains(&snapshot.auth.jwt.secret),
            "Debug 不应打印 JWT 密钥: {printed}"
        );
    }
}
