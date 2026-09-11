//! 二进制入口:调用 `hoshiyomi` 库的 CLI(无子命令时启动 HTTP 服务)。

use hoshiyomi::cli;

#[tokio::main]
async fn main() -> hoshiyomi::Result<()> {
    cli::run().await
}
