use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = env!("CARGO_CRATE_NAME"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
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

#[derive(Subcommand)]
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
