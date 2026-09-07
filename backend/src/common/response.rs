use serde::Serialize;
use utoipa::ToSchema;

/// 统一响应信封:code=0 成功,非 0 为 HTTP 状态码;data 恒序列化(错误时为 null)
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub errors: Option<serde_json::Value>,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            errors: None,
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            errors: None,
            data: None,
        }
    }

    pub fn error_with_errors(
        code: i32,
        message: impl Into<String>,
        errors: serde_json::Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            errors: Some(errors),
            data: None,
        }
    }
}

/// 分页数据载荷
#[derive(Debug, Serialize, ToSchema)]
pub struct PageData<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}
