use clap::Parser;

use super::{
    command::{Cli, Commands, RoleCommands},
    command_impl,
};
use crate::{config::AppConfig, db, error::Result, state::Services};

pub async fn run() -> Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    match cli.command {
        None => crate::serve::serve().await,
        // 无需 DB 的子命令不建立连接
        Some(Commands::Config) => {
            let cfg = AppConfig::load()?;
            command_impl::print_config(&cfg)
        }
        Some(Commands::Perms) => command_impl::list_permissions().await,
        Some(cmd) => {
            let cfg = AppConfig::load()?;
            // CLI 不跑 migrate:命令假设 schema 已存在(由 serve 首次启动创建)
            let pool = db::connect(&cfg.database.url).await?;
            let services = Services::new(
                pool,
                &cfg.auth.jwt.secret,
                cfg.auth.jwt.expires_in_seconds,
                cfg.auth.session.ttl_hours,
            );

            match cmd {
                Commands::Init => command_impl::init_rbac(&services).await,
                Commands::CreateSuperuser {
                    username,
                    password,
                    email,
                } => command_impl::create_superuser(&services, username, password, email).await,
                Commands::Role(RoleCommands::List) => command_impl::list_roles(&services).await,
                Commands::Role(RoleCommands::Create {
                    name,
                    description,
                    perms,
                }) => command_impl::create_role(&services, name, description, perms).await,
                Commands::Role(RoleCommands::Delete { name }) => {
                    command_impl::delete_role(&services, name).await
                }
                Commands::Config | Commands::Perms => unreachable!("已在上层提前处理"),
            }
        }
    }
}
