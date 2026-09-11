use std::sync::{Arc, Mutex};

use crate::{
    build_app,
    config::{AppConfig, Paths, RawAppConfig},
    db,
    state::AppState,
};
use vivarium_rs::serve::telemetry::{TelemetryOptions, init as init_telemetry};
use vivarium_rs::{ApiError, ErrorKind, Result};

pub async fn serve() -> Result<()> {
    let _ = dotenvy::dotenv();

    // watch() 监听的是 config.toml 的父目录,故目录必须先存在,否则热重载不可用
    let _ = std::fs::create_dir_all(Paths::config_dir());

    let cfg = AppConfig::load()?;

    // 日志:RUST_LOG 优先(与现行为一致);未设置时用 {crate}={level},tower_http={level} 回退串
    let level = if std::env::var_os("RUST_LOG").is_some() {
        None
    } else {
        let lvl = cfg.get().log.level.clone();
        Some(format!(
            "{name}={lvl},tower_http={lvl}",
            name = env!("CARGO_CRATE_NAME")
        ))
    };
    // guard 必须活到进程结束:drop 即 flush 并停止写文件
    let _telemetry = init_telemetry(TelemetryOptions {
        level,
        json_file: Some(Paths::log_dir().join("access.log")),
        ansi: true,
    })
    .map_err(|e| ApiError::new(ErrorKind::Internal, "日志初始化失败").with_source(e))?;

    let pool = db::connect(&cfg.get().database.url).await?;
    db::migrate(&pool).await?;

    let state = AppState::new(cfg.clone(), pool);

    // 热重载 1/2:重建 JWT 与刷新令牌运行时(auth.jwt.* 立即生效)
    cfg.register({
        let services = state.srv().clone();
        move |raw: &RawAppConfig| services.apply_config(raw)
    });

    // 热重载 2/2:只在启动时生效的字段显式告警,不假装生效
    let last = Arc::new(Mutex::new(cfg.get().as_ref().clone()));
    cfg.register(move |raw: &RawAppConfig| {
        let Ok(mut last) = last.lock() else { return };
        if raw.server != last.server {
            tracing::warn!("server.host/port 已变更，需重启生效");
        }
        if raw.database != last.database {
            tracing::warn!("database.url 已变更，需重启生效");
        }
        if raw.log != last.log {
            tracing::warn!("log.level 已变更，需重启生效");
        }
        if raw.auth.session != last.auth.session {
            tracing::warn!("auth.session 已变更，需重启生效");
        }
        *last = raw.clone();
    });
    cfg.on_error(|err| tracing::error!(error = %err, "配置热重载失败，继续使用旧值"));

    let app = build_app(state);

    let addr = format!("{}:{}", cfg.get().server.host, cfg.get().server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ApiError::new(ErrorKind::Internal, "端口绑定失败").with_source(e))?;
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());

    // watcher 必须绑定到活到进程结束的变量:`let _ =` 会立刻 drop 并停止监听
    let _config_watcher = cfg.watch()?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ApiError::new(ErrorKind::Internal, "服务器错误").with_source(e))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
