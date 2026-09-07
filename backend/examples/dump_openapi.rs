//! 输出 OpenAPI 规范 JSON(纯内存构造,无需数据库):
//! cargo run --example dump_openapi > docs/openapi.json
use hoshiyomi::modules;
use utoipa_axum::router::OpenApiRouter;

fn main() {
    let (_, api) = OpenApiRouter::new()
        .nest("/api/v1", modules::router())
        .split_for_parts();
    println!("{}", api.to_pretty_json().expect("OpenAPI 序列化失败"));
}
