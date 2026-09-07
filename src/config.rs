use std::env;

pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("未设置 DATABASE_URL，请参考 .env.example"),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        }
    }
}
