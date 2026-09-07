use sqlx::MySqlPool;

use crate::{
    config::AppConfig,
    modules::{
        auth::{service::AuthService, session::SessionService, token::TokenService},
        role::service::RoleService,
        user::service::UserService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: MySqlPool,
    pub services: Services,
}

impl AppState {
    pub fn new(config: AppConfig, db: MySqlPool) -> Self {
        let services = Services::new(
            db.clone(),
            &config.auth.jwt.secret,
            config.auth.jwt.expires_in_seconds,
            config.auth.session.ttl_hours,
        );
        Self {
            config,
            db,
            services,
        }
    }

    pub fn cfg(&self) -> &AppConfig {
        &self.config
    }

    pub fn db(&self) -> &MySqlPool {
        &self.db
    }

    pub fn srv(&self) -> &Services {
        &self.services
    }
}

/// 领域服务依赖注入容器
#[derive(Clone)]
pub struct Services {
    pub user: UserService,
    pub role: RoleService,
    pub auth: AuthService,
    pub session: SessionService,
    pub token: TokenService,
}

impl Services {
    pub fn new(
        db: MySqlPool,
        jwt_secret: &str,
        jwt_expires_in_seconds: u64,
        session_ttl_hours: u64,
    ) -> Self {
        Self {
            user: UserService::new(db.clone()),
            role: RoleService::new(db.clone()),
            auth: AuthService::new(db.clone()),
            session: SessionService::new(db.clone(), session_ttl_hours),
            token: TokenService::new(db, jwt_secret, jwt_expires_in_seconds),
        }
    }
}
