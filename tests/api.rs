use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hoshiyomi::{build_app, state::AppState};
use sqlx::mysql::MySqlPoolOptions;
use tower::ServiceExt;

fn lazy_state() -> AppState {
    AppState {
        db: MySqlPoolOptions::new()
            .connect_lazy("mysql://nobody:nope@127.0.0.1:1/none")
            .expect("lazy pool 构造失败"),
    }
}

#[tokio::test]
async fn health_returns_ok_envelope() {
    let res = build_app(lazy_state())
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("\"code\":0"));
}

#[tokio::test]
async fn get_user_with_unreachable_db_maps_to_500_envelope() {
    let res = build_app(lazy_state())
        .oneshot(Request::get("/api/v1/users/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("\"code\":500"));
}
