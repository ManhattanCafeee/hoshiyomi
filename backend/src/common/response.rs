use serde::Serialize;
use utoipa::ToSchema;

pub use vivarium_rs::ApiResponse;

/// 分页数据载荷
#[derive(Debug, Serialize, ToSchema)]
pub struct PageData<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}
