//! 黑盒集成测试:以进程内 `oneshot` 驱动 `build_app` 装配的路由;
//! 连接池惰性指向不可达地址,故不依赖数据库与环境变量。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use hoshiyomi::{
    build_app,
    config::{AppConfig, RawAppConfig},
    state::AppState,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::mysql::MySqlPoolOptions;
use tower::ServiceExt;

fn lazy_state() -> AppState {
    let pool = MySqlPoolOptions::new()
        .connect_lazy("mysql://nobody:nope@127.0.0.1:1/none")
        .expect("lazy pool 构造失败");
    AppState::new(
        AppConfig::new(RawAppConfig::default()).expect("配置构造失败"),
        pool,
    )
}

async fn send(method: &str, uri: &str, body: Option<Value>) -> (StatusCode, String, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let req = match body {
        Some(b) => builder.body(Body::from(b.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let res = build_app(lazy_state()).oneshot(req).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or_default();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&bytes).expect("响应体应为合法 JSON");
    (status, content_type, value)
}

async fn get_json(uri: &str) -> (StatusCode, String, Value) {
    send("GET", uri, None).await
}

async fn post_json(uri: &str, body: Value) -> (StatusCode, String, Value) {
    send("POST", uri, Some(body)).await
}

#[tokio::test]
async fn health_returns_ok_envelope() {
    let (status, content_type, body) = get_json("/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "ok");
    assert_eq!(body["data"], "ok");
}

#[tokio::test]
async fn get_user_without_cookie_maps_to_401_envelope() {
    let (status, content_type, body) = get_json("/api/v1/users/1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 401);
    assert_eq!(body["message"], "缺少会话 Cookie");
    assert_eq!(body["data"], Value::Null);
}

#[tokio::test]
async fn get_user_with_invalid_id_maps_to_400_envelope() {
    let (status, content_type, body) = get_json("/api/v1/users/abc").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("用户 id 无效"),
        "message 应说明 id 无效: {}",
        body["message"]
    );
    assert_eq!(body["data"], Value::Null);
}

#[tokio::test]
async fn unmatched_route_returns_404_envelope() {
    let (status, content_type, body) = get_json("/api/v1/nothing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 404);
    assert_eq!(body["message"], "未找到");
    assert_eq!(body["data"], Value::Null);
}

#[tokio::test]
async fn login_with_unreachable_db_maps_to_500_envelope() {
    let (status, content_type, body) = post_json(
        "/api/v1/auth/login",
        json!({"username": "u", "password": "p"}),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 500);
    assert_eq!(body["message"], "数据库错误");
    assert_eq!(body["data"], Value::Null);
}

/// 校验失败的契约:422 + 结构化 errors + 中文文案(库的 catalog 已注入中文),
/// 且 params 不回显提交值、规则消息取自 DTO 的 validator 声明。
#[tokio::test]
async fn validation_failure_is_422_with_structured_errors() {
    let (status, content_type, body) = post_json(
        "/api/v1/auth/register",
        json!({"username": "ab", "email": "nope", "password": "short"}),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        content_type.starts_with("application/json"),
        "content-type 应为 JSON: {content_type}"
    );
    assert_eq!(body["code"], 422);
    assert_eq!(body["message"], "校验失败");
    assert_eq!(body["errors"]["username"][0]["code"], "length");
    assert_eq!(
        body["errors"]["username"][0]["message"],
        "用户名长度需在 3-20 之间"
    );
    assert!(
        body["errors"]["username"][0]["params"]
            .get("value")
            .is_none(),
        "params 不得回显提交值: {}",
        body["errors"]["username"][0]["params"]
    );
    assert_eq!(body["data"], Value::Null);
}

fn has_chinese(text: &str) -> bool {
    text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// 递归收集 `value` 下所有 `description`(object 与 array 都下钻)。
fn collect_descriptions(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get("description") {
                out.push(text.clone());
            }
            for child in map.values() {
                collect_descriptions(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_descriptions(item, out);
            }
        }
        _ => {}
    }
}

/// 库自有 schema 的描述来自英文 rustdoc,由 `texts::localize_schema` 统一换成中文。
/// 上游改写文案会让本地化表静默失配(描述退回英文),故在此断言覆盖完整。
#[test]
fn openapi_schema_descriptions_are_chinese() {
    let spec = serde_json::to_value(hoshiyomi::api_document()).expect("spec 序列化失败");
    let schemas = spec
        .get("components")
        .and_then(|components| components.get("schemas"))
        .expect("spec 缺少 components.schemas");

    let mut descriptions = Vec::new();
    collect_descriptions(schemas, &mut descriptions);
    assert!(
        !descriptions.is_empty(),
        "components.schemas 下没有任何描述,该断言失去意义"
    );

    let untranslated: Vec<&String> = descriptions
        .iter()
        .filter(|text| !has_chinese(text))
        .collect();
    assert!(
        untranslated.is_empty(),
        "以下 schema 描述未本地化,请补 texts::localize_schema 的表: {untranslated:#?}"
    );
}
