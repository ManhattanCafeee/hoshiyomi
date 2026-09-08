use std::str::FromStr;

use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};

use crate::error::{ErrorKind, Result, ResultExt};

pub async fn connect(database_url: &str) -> Result<MySqlPool> {
    let connect_options = MySqlConnectOptions::from_str(database_url)
        .err_kind_msg(ErrorKind::Config, "DATABASE_URL 无效,请参考 .env.example")?;

    MySqlPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // 会话时区固定 UTC:DATETIME 无时区语义,
                // 写入端 UTC_TIMESTAMP() 与读取端 DateTime<Utc> 必须一致
                sqlx::query("SET time_zone = '+00:00'")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "数据库连接失败");
            ErrorKind::Internal.err_msg(e, "数据库连接失败")
        })
}

pub async fn migrate(pool: &MySqlPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await.map_err(|e| {
        tracing::error!(error = %e, "数据库迁移失败");
        ErrorKind::Internal.err_msg(e, "数据库迁移失败")
    })
}
