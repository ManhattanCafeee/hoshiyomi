//! 输出 OpenAPI 规范 JSON(纯内存构造,无需数据库):
//! cargo run --example dump_openapi > docs/openapi.json
fn main() {
    let api = hoshiyomi::api_document();
    println!("{}", api.to_pretty_json().expect("OpenAPI 序列化失败"));
}
