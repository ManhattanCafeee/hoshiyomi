use hoshiyomi::{build_app, config::Config, state::AppState};
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
        )
        .init();

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("数据库连接失败，请检查 DATABASE_URL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败");

    let app = build_app(AppState { db: pool });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("端口绑定失败");
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("服务器错误");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
