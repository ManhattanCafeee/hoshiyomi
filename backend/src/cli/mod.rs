//! 命令行接口:子命令定义、实现与分发入口

/// 命令行参数与子命令定义
pub mod command;
/// 各子命令的具体实现
pub mod command_impl;
/// CLI 入口与子命令分发
pub mod run;

pub use run::run;
