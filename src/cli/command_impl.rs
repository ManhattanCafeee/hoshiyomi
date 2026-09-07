use crate::{
    bail,
    config::RawAppConfig,
    error::{ErrorKind, Result},
    modules::role::models::{DefaultRole, Perm},
    state::Services,
};

pub async fn init_rbac(services: &Services) -> Result<()> {
    println!("Initializing roles...");

    for role in DefaultRole::all() {
        let name = role.name().to_owned();
        let desc = Some(role.description().to_owned());
        let perms = role.default_permissions();
        match services.role.create(name, desc, perms).await {
            Ok(r) => println!(
                "  已创建角色: {} ({} 个权限)",
                r.name,
                r.parse_perms().len()
            ),
            Err(_) if services.role.find_by_name(role.name()).await?.is_some() => {
                println!("  跳过 {} (已存在)", role.name());
            }
            Err(e) => return Err(e),
        }
    }

    println!("\nRBAC 初始化完成!");
    Ok(())
}

pub async fn create_superuser(
    services: &Services,
    username: Option<String>,
    password: Option<String>,
    email: Option<String>,
) -> Result<()> {
    let username = match username {
        Some(u) => u,
        None => inquire::Text::new("用户名:").prompt()?,
    };
    let password = match password {
        Some(p) => p,
        None => inquire::Password::new("密码:").prompt()?,
    };
    let email = match email {
        Some(e) => e,
        None => inquire::Text::new("邮箱:").prompt()?,
    };

    println!("创建超级用户: {}", username);
    let user = services
        .user
        .create(username.clone(), email, password)
        .await?;
    println!("  用户已创建,ID: {}", user.id);

    let Some(superuser_role) = services.role.find_by_name("superuser").await? else {
        bail!(ErrorKind::NotFound, "未找到 superuser 角色,请先运行 init");
    };

    services
        .role
        .assign_to_user(user.id, superuser_role.id)
        .await?;
    println!("  已分配 superuser 角色");

    println!("\n超级用户 '{}' 创建成功!", username);
    Ok(())
}

pub async fn list_roles(services: &Services) -> Result<()> {
    let roles = services.role.list_all().await?;

    println!("Roles:");
    println!("{:-<80}", "");
    for role in roles {
        let perms = role.perm_codes().join(", ");
        println!(
            "  {} - {} [{}]",
            role.name,
            role.description.unwrap_or_default(),
            perms
        );
    }
    Ok(())
}

pub async fn create_role(
    services: &Services,
    name: String,
    description: Option<String>,
    perm_codes: Option<String>,
) -> Result<()> {
    let perms: Vec<Perm> = perm_codes
        .unwrap_or_default()
        .split(',')
        .filter_map(|c| Perm::from_code(c.trim()))
        .collect();
    let role = services.role.create(name, description, &perms).await?;
    println!(
        "已创建角色: {} (ID: {}) 共 {} 个权限",
        role.name,
        role.id,
        perms.len()
    );
    Ok(())
}

pub async fn delete_role(services: &Services, name: String) -> Result<()> {
    let Some(role) = services.role.find_by_name(&name).await? else {
        bail!(ErrorKind::NotFound, "角色不存在");
    };

    services.role.delete(role.id).await?;
    println!("已删除角色: {}", name);
    Ok(())
}

pub async fn list_permissions() -> Result<()> {
    println!("可用权限:");
    println!("{:-<40}", "");
    for perm in Perm::all() {
        println!("  {} - {}", perm.code(), perm.description());
    }
    Ok(())
}

pub fn print_config(config: &RawAppConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    println!("{}", json);
    Ok(())
}
