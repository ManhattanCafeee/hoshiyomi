use std::path::PathBuf;

use crate::config::{ENV_PREFIX, PROJECT_NAME};

pub struct Paths;

impl Paths {
    fn local_dir() -> PathBuf {
        PathBuf::from(format!(".{PROJECT_NAME}"))
    }

    fn is_local_mode() -> bool {
        Self::local_dir().exists() || std::env::var(format!("{ENV_PREFIX}_LOCAL_MODE")).is_ok()
    }

    pub fn config_dir() -> PathBuf {
        let local = Self::local_dir();
        if Self::is_local_mode() {
            return local;
        }
        dirs::config_dir()
            .map(|p| p.join(PROJECT_NAME))
            .unwrap_or(local)
    }

    pub fn data_dir() -> PathBuf {
        let local = Self::local_dir();
        if Self::is_local_mode() {
            return local;
        }
        dirs::data_dir()
            .map(|p| p.join(PROJECT_NAME))
            .unwrap_or(local)
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn log_dir() -> PathBuf {
        Self::data_dir().join("logs")
    }
}
