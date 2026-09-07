use crate::{
    build_app,
    config::AppConfig,
    db,
    error::{ErrorKind, Result, ResultExt},
    infra,
    state::AppState,
};

pub async fn serve() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cfg = AppConfig::load()?;

    infra::init_tracing(&cfg.log);

    let pool = db::connect(&cfg.database.url).await?;
    db::migrate(&pool).await?;

    let app = build_app(AppState::new(cfg.clone(), pool));

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .err_kind_msg(ErrorKind::Config, "端口绑定失败")?;
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .err_kind_msg(ErrorKind::Internal, "服务器错误")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
