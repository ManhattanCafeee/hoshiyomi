//! 分页响应载荷。
//!
//! 库只提供内部用的 `Page<T>`(u32 页号/页大小);对外的分页契约保持本仓的 `PageData`
//! (u64,`total`/`page`/`per_page` 均为 int64),以免改动线上 schema 名与字段类型。

use serde::Serialize;
use utoipa::ToSchema;

/// 分页数据载荷(库的 `Page<T>` 映射而来)
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct PageData<T> {
    /// 本页数据
    pub(crate) items: Vec<T>,
    /// 记录总数
    pub(crate) total: u64,
    /// 当前页码(回显请求值)
    pub(crate) page: u64,
    /// 每页条数(回显请求值)
    pub(crate) per_page: u64,
}
