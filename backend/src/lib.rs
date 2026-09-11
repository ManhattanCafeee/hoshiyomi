//! hoshiyomi 后端库:对外暴露 `build_app`(路由装配)与 `api_document`(OpenAPI spec),错误类型与响应信封统一由 `vivarium_rs` 提供
//!
//! 对外只开放 bin 与集成测试需要的部分:cli(bin 调用)、config/state(测试构造 AppState)

/// 命令行接口:子命令定义与分发入口
pub mod cli;
/// 配置加载、字段树与热重载
pub mod config;
/// 应用状态与领域服务容器
pub mod state;

// 其余模块仅库内使用,收为 crate 可见,避免无意义的公开面
pub(crate) mod db;
pub(crate) mod middleware;
pub(crate) mod modules;
pub(crate) mod pagination;
pub(crate) mod serve;
pub(crate) mod texts;

pub use vivarium_rs::{ApiError, ErrorKind, Result};

use axum::{Router, response::IntoResponse};
use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use utoipa_axum::router::OpenApiRouter;

use state::AppState;

/// 唯一的 spec 构造入口:`build_app` 与 `examples/dump_openapi` 共用,避免线上 spec 与提交的 json 分叉。
pub fn api_document() -> utoipa::openapi::OpenApi {
    let (_, mut api) = OpenApiRouter::new()
        .nest("/api/v1", modules::router())
        .split_for_parts();
    set_info(&mut api);
    api
}

fn set_info(api: &mut utoipa::openapi::OpenApi) {
    api.info = vivarium_rs::openapi::info(
        "hoshiyomi API",
        env!("CARGO_PKG_VERSION"),
        "hoshiyomi 后端 REST API",
    );
    // 库自有 schema 的描述是英文 rustdoc,统一换成中文(见 texts::localize_schema)
    let _ = texts::localize_schema(api);
}

/// 装配完整应用路由:业务路由、OpenAPI 文档挂载、会话层与 CORS/Trace 中间件
pub fn build_app(state: AppState) -> Router {
    // 任何库调用之前先装中文文案(测试直接调 build_app,故必须在这里装)
    texts::install_chinese_texts();

    // spec 取自本次装配路由的 split_for_parts,线上文档与路由面因此不可能不一致
    let (router, mut api) = OpenApiRouter::new()
        .nest("/api/v1", modules::router())
        .split_for_parts();
    set_info(&mut api);

    let session = state.srv().session().clone();

    // 先灌入状态得到 Router<()>:文档挂载与下面的中间件都是无状态的
    router
        .with_state(state)
        // 挂载 /api-docs/openapi.json、/api-docs/scalar、/api-docs/swagger-ui
        .merge(vivarium_rs::openapi::mount(Router::new(), "/api-docs", api))
        // fallback 必须先于 layer 注册,否则 404 响应不经过 CORS/Trace 中间件
        .fallback(not_found)
        // 会话解析/惰性删除/半 TTL 滑动续期都由库的中间件负责
        .layer(vivarium_rs::session_layer(session))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO))
                        .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
                )
                .layer(middleware::cors::cors()),
        )
}

async fn not_found() -> impl IntoResponse {
    // 走库的 ApiError → 统一的 ApiResponse 信封(404 与「未找到」)
    ApiError::not_found("未找到")
}
