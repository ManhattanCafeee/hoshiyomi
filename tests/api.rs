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
    AppState::new(AppConfig::new(RawAppConfig::default()), pool)
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
    assert_eq!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("用户 id 无效"),
        true,
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
