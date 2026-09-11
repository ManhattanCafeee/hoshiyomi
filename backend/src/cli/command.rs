use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = env!("CARGO_CRATE_NAME"))]
/// 命令行参数根结构
pub struct Cli {
    /// 待执行的子命令,缺省时启动服务
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
/// 顶层子命令:配置打印、RBAC 初始化、超级用户创建、角色管理与权限查看
pub enum Commands {
    /// 打印配置 JSON
    Config,

    /// 初始化默认角色(幂等)
    Init,

    /// 创建超级用户(缺省参数时交互式输入)
    CreateSuperuser {
        /// 用户名
        #[arg(short, long)]
        username: Option<String>,

        /// 密码
        #[arg(short, long)]
        password: Option<String>,

        /// 邮箱(users.email 非空)
        #[arg(short, long)]
        email: Option<String>,
    },

    /// 角色管理
    #[command(subcommand)]
    Role(RoleCommands),

    /// 列出全部权限
    Perms,
}

impl std::fmt::Debug for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config => f.write_str("Config"),
            Self::Init => f.write_str("Init"),
            // 命令行口令不打印
            Self::CreateSuperuser {
                username, email, ..
            } => f
                .debug_struct("CreateSuperuser")
                .field("username", username)
                .field("email", email)
                .field("password", &"<redacted>")
                .finish(),
            Self::Role(role) => f.debug_tuple("Role").field(role).finish(),
            Self::Perms => f.write_str("Perms"),
        }
    }
}

#[derive(Debug, Subcommand)]
/// 角色管理子命令:列出、创建或删除角色
pub enum RoleCommands {
    /// 列出所有角色
    List,

    /// 创建角色
    Create {
        /// 角色名
        #[arg(short, long)]
        name: String,

        /// 角色描述
        #[arg(short, long)]
        description: Option<String>,

        /// 权限码,逗号分隔(如 "user:read,user:write")
        #[arg(short, long)]
        perms: Option<String>,
    },

    /// 删除角色
    Delete {
        /// 角色名
        #[arg(short, long)]
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Debug` 不得打印命令行口令。
    #[test]
    fn debug_redacts_password_argument() {
        let cli = Cli::try_parse_from([
            "hoshiyomi",
            "create-superuser",
            "-u",
            "root",
            "-p",
            "s3cret",
        ])
        .expect("参数解析失败");
        let printed = format!("{cli:?}");

        assert!(printed.contains("root"), "用户名应可打印: {printed}");
        assert!(
            !printed.contains("s3cret"),
            "口令不应出现在 Debug 输出: {printed}"
        );
    }
}
