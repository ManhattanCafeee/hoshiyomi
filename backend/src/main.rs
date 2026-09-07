use hoshiyomi::cli;

#[tokio::main]
async fn main() -> hoshiyomi::Result<()> {
    cli::run().await
}
