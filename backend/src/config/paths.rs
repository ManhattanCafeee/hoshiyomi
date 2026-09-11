use std::path::PathBuf;

use crate::config::{ENV_PREFIX, PROJECT_NAME};

/// 配置/数据/日志目录的解析入口(local mode 由 `HOSHIYOMI_LOCAL_MODE` 或 `./.hoshiyomi` 触发)
#[derive(Debug)]
pub struct Paths;

impl Paths {
    fn local_dir() -> PathBuf {
        PathBuf::from(format!(".{PROJECT_NAME}"))
    }

    fn is_local_mode() -> bool {
        Self::local_dir().exists() || std::env::var(format!("{ENV_PREFIX}_LOCAL_MODE")).is_ok()
    }

    /// 配置目录:local mode 为 ./.hoshiyomi,否则为系统配置目录
    pub fn config_dir() -> PathBuf {
        let local = Self::local_dir();
        if Self::is_local_mode() {
            return local;
        }
        dirs::config_dir()
            .map(|p| p.join(PROJECT_NAME))
            .unwrap_or(local)
    }

    /// 数据目录:local mode 为 ./.hoshiyomi,否则为系统数据目录
    pub fn data_dir() -> PathBuf {
        let local = Self::local_dir();
        if Self::is_local_mode() {
            return local;
        }
        dirs::data_dir()
            .map(|p| p.join(PROJECT_NAME))
            .unwrap_or(local)
    }

    /// 配置文件路径:配置目录下的 config.toml
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// 日志目录:数据目录下的 logs
    pub fn log_dir() -> PathBuf {
        Self::data_dir().join("logs")
    }
}
