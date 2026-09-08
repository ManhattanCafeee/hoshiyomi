use std::fmt::{Debug, Display};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use thiserror::Error;

use crate::common::response::ApiResponse;

type DynError = dyn std::error::Error + Send + Sync + 'static;
type BoxedDynError = Box<DynError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    // ========================================================
    // 请求与数据
    // ========================================================
    BadRequest,
    DataParse,
    ValidationFailed,
    InvalidParameter,

    // ========================================================
    // 认证
    // ========================================================
    Unauthorized,
    Forbidden,
    PermissionDenied,
    InvalidCredentials,

    // ========================================================
    // 资源
    // ========================================================
    NotFound,
    AlreadyExists,

    // ========================================================
    // 系统与环境
    // ========================================================
    Config,
    External,
    Internal,
}

impl ErrorKind {
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::BadRequest => "请求无效",
            Self::DataParse => "数据解析失败",
            Self::ValidationFailed => "校验失败",
            Self::InvalidParameter => "参数无效",
            Self::Unauthorized => "未授权",
            Self::Forbidden => "禁止访问",
            Self::PermissionDenied => "权限不足",
            Self::InvalidCredentials => "凭证无效",
            Self::NotFound => "未找到",
            Self::AlreadyExists => "资源已存在",
            Self::Config => "配置错误",
            Self::External => "外部服务错误",
            Self::Internal => "数据库错误",
        }
    }

    pub fn is_internal_error(&self) -> bool {
        matches!(self, Self::Config | Self::Internal)
    }

    pub fn to_error(self) -> AppError {
        AppError::new(self)
    }

    pub fn msg(self, msg: impl Into<String>) -> AppError {
        AppError::new(self).with_msg(msg)
    }

    pub fn err<E>(self, err: E) -> AppError
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        AppError::new(self).with_err(err)
    }

    pub fn dyn_err(self, err: BoxedDynError) -> AppError {
        AppError::new(self).with_dyn_err(err)
    }

    pub fn err_msg<E>(self, err: E, msg: impl Into<String>) -> AppError
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        AppError::new(self).with_msg(msg).with_err(err)
    }

    pub fn wrap_internal<E>(err: E) -> AppError
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Internal.err(err)
    }
}

#[derive(Error)]
pub struct AppError {
    kind: ErrorKind,
    message: Option<String>,
    errors: Option<Value>,
    #[source]
    source: Option<BoxedDynError>,
}

impl AppError {
    fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            errors: None,
            source: None,
        }
    }

    fn with_msg(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    // 源错误自动填充 message,除非是 internal 错误(internal 不向客户端泄漏细节)
    fn with_err<E>(mut self, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        if self.message.is_none() && !self.kind.is_internal_error() {
            self.message = Some(err.to_string());
        }
        self.source = Some(Box::new(err));
        self
    }

    fn with_dyn_err(mut self, err: BoxedDynError) -> Self {
        if self.message.is_none() && !self.kind.is_internal_error() {
            self.message = Some(err.to_string());
        }
        self.source = Some(err);
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        self.message
            .as_deref()
            .unwrap_or_else(|| self.kind.default_message())
    }

    pub fn errors(&self) -> Option<Value> {
        if let Some(errors) = self.errors.as_ref() {
            return Some(errors.clone());
        }
        let err = self.source.as_ref()?;
        if let Some(v) = err.downcast_ref::<validator::ValidationErrors>() {
            return Some(serde_json::to_value(v).unwrap_or(Value::Null));
        }
        None
    }

    pub fn trace_source(&self) {
        if self.kind.is_internal_error() {
            tracing::error!("Internal error: {}", self);
        }
    }

    pub fn source_ref(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|s| s as &(dyn std::error::Error + 'static))
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppError")
            .field("kind", &self.kind)
            .field("message", &self.message())
            .field("source", &self.source)
            .finish()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message()).and_then(|_| {
            if let Some(err) = self.source.as_ref() {
                write!(f, "\nCause: {}", err)
            } else {
                Ok(())
            }
        })
    }
}

pub trait OptionAppExt<T> {
    fn ok_or_err_msg(self, kind: ErrorKind, msg: impl Into<String>) -> Result<T>;
}

impl<T> OptionAppExt<T> for Option<T> {
    fn ok_or_err_msg(self, kind: ErrorKind, msg: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| kind.msg(msg))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

pub trait ResultExt<T> {
    fn err_kind(self, kind: ErrorKind) -> Result<T>;
    fn err_kind_msg(self, kind: ErrorKind, msg: impl Into<String>) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn err_kind(self, kind: ErrorKind) -> Result<T> {
        self.map_err(|e| kind.err(e))
    }

    fn err_kind_msg(self, kind: ErrorKind, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| kind.err_msg(e, msg))
    }
}

impl From<ErrorKind> for AppError {
    fn from(kind: ErrorKind) -> Self {
        kind.to_error()
    }
}

/// MySQL 重复键错误(1062)映射为 AlreadyExists,其余原样返回。
/// 用于「先查后插」并发竞态下把唯一键兜底错误还原为 409。
pub fn map_duplicate_key(e: AppError, msg: impl Into<String>) -> AppError {
    let is_dup = e
        .source_ref()
        .and_then(|s| s.downcast_ref::<sqlx::Error>())
        .and_then(|db| db.as_database_error())
        .and_then(|d| d.code())
        .is_some_and(|code| code == "1062");
    if is_dup {
        if let Some(src) = e.source_ref() {
            tracing::warn!(error = %src, "唯一键冲突(MySQL 1062),转为资源冲突响应");
        }
        ErrorKind::AlreadyExists.msg(msg)
    } else {
        e
    }
}

macro_rules! register_errors {
    ( $( $err_type:ty => $kind:expr $(, $msg:literal)? );* $(;)? ) => {
        $(
            impl From<$err_type> for AppError {
                fn from(e: $err_type) -> Self {
                    let kind = $kind;
                    $(
                        return kind.err_msg(e, $msg);
                    )?
                    #[allow(unreachable_code)]
                    kind.err(e)
                }
            }
        )*
    };
}

register_errors! {
    std::io::Error                           => ErrorKind::Internal;
    serde_json::Error                        => ErrorKind::DataParse;
    config::ConfigError                      => ErrorKind::Config;
    sqlx::Error                              => ErrorKind::Internal;
    inquire::error::InquireError             => ErrorKind::Internal;
    axum::extract::rejection::PathRejection  => ErrorKind::InvalidParameter;
    axum::extract::rejection::QueryRejection => ErrorKind::InvalidParameter;
    axum::extract::rejection::JsonRejection  => ErrorKind::DataParse;
    validator::ValidationErrors              => ErrorKind::ValidationFailed, "校验失败";
    tokio::task::JoinError                   => ErrorKind::Internal;
}

fn error_status_code(kind: &ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::BadRequest
        | ErrorKind::DataParse
        | ErrorKind::InvalidParameter
        | ErrorKind::ValidationFailed => StatusCode::BAD_REQUEST,
        ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
        ErrorKind::Forbidden | ErrorKind::PermissionDenied | ErrorKind::InvalidCredentials => {
            StatusCode::FORBIDDEN
        }
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::AlreadyExists => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.trace_source();
        let status = error_status_code(self.kind());
        let message = self.message().to_string();
        let body = match self.errors() {
            Some(errors) => {
                ApiResponse::<()>::error_with_errors(status.as_u16() as i32, message, errors)
            }
            None => ApiResponse::<()>::error(status.as_u16() as i32, message),
        };
        (status, Json(body)).into_response()
    }
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return ::core::result::Result::Err($crate::ErrorKind::Internal.msg($msg))
    };
    ($fmt:literal, $($arg:tt)+) => {
        return ::core::result::Result::Err($crate::ErrorKind::Internal.msg(format!($fmt, $($arg)+)))
    };
    ($kind:expr $(,)?) => {
        return ::core::result::Result::Err($crate::ErrorKind::to_error($kind))
    };
    ($kind:expr, $($arg:tt)+) => {
        return ::core::result::Result::Err($crate::ErrorKind::msg($kind, format!($($arg)+)))
    };
}
