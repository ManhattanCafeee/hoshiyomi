/// 项目名,取自包名(Cargo 的 CARGO_PKG_NAME)
pub static PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
/// 环境变量前缀,固定为 HOSHIYOMI
pub static ENV_PREFIX: &str = "HOSHIYOMI";
