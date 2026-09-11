use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use sqlx::MySqlPool;
use vivarium_rs::{
    CookieOptions, JwtConfig, JwtVerifier, KeyRing, RefreshTokenManager, SessionAuth,
};

use crate::{
    config::{AppConfig, RawAppConfig},
    modules::{
        auth::{
            service::AuthService,
            stores::{MySqlRefreshTokenStore, MySqlSessionStore},
        },
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
        let services = Services::new(db.clone(), &config.get());
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

/// 认证运行时快照:JWT 校验与刷新令牌管理器共享同一份配置。
///
/// 配置热重载时**整体重建并原子替换**,因此同一请求内应只取一次快照,
/// 避免一个请求里出现两代运行时。
///
/// 注意**会话层不在这里**:`session_layer` 在构建时就把 `SessionAuth`(cookie 名与 TTL)
/// 作为 layer state 捕获,配置变更无法影响它,故 `auth.session.*` 属于「需重启生效」。
#[derive(Clone)]
pub struct AuthRuntime {
    pub jwt: JwtVerifier,
    pub tokens: RefreshTokenManager<MySqlRefreshTokenStore>,
}

/// 领域服务依赖注入容器
pub struct Services {
    pub user: UserService,
    pub role: RoleService,
    pub auth: AuthService,
    /// 会话层:构造后固定(见 `AuthRuntime` 的说明)
    session: SessionAuth<MySqlSessionStore>,
    pool: MySqlPool,
    runtime: Arc<ArcSwap<AuthRuntime>>,
}

impl Clone for Services {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            role: self.role.clone(),
            auth: self.auth.clone(),
            session: self.session.clone(),
            pool: self.pool.clone(),
            runtime: Arc::clone(&self.runtime),
        }
    }
}

impl Services {
    pub fn new(pool: MySqlPool, raw: &RawAppConfig) -> Self {
        Self {
            user: UserService::new(pool.clone()),
            role: RoleService::new(pool.clone()),
            auth: AuthService::new(pool.clone()),
            session: build_session(&pool, raw),
            runtime: Arc::new(ArcSwap::from_pointee(build_runtime(&pool, raw))),
            pool,
        }
    }

    /// 当前 JWT/刷新令牌运行时快照
    pub fn runtime(&self) -> Arc<AuthRuntime> {
        self.runtime.load_full()
    }

    /// 会话层(启动时构造,见 `AuthRuntime`)
    pub fn session(&self) -> &SessionAuth<MySqlSessionStore> {
        &self.session
    }

    /// 配置热重载:按新配置重建 JWT/刷新令牌运行时
    pub fn apply_config(&self, raw: &RawAppConfig) {
        self.runtime.store(Arc::new(build_runtime(&self.pool, raw)));
    }
}

fn build_session(pool: &MySqlPool, raw: &RawAppConfig) -> SessionAuth<MySqlSessionStore> {
    SessionAuth::new(
        MySqlSessionStore::new(pool.clone()),
        // 本项目当前不设 Secure(HTTP 开发),CookieOptions 默认 secure=true
        CookieOptions::new(raw.auth.session.cookie_name.clone()).insecure(),
        Duration::from_secs(raw.auth.session.ttl_hours * 3600),
        // 无绝对上限,与原实现一致
        None,
    )
}

fn build_runtime(pool: &MySqlPool, raw: &RawAppConfig) -> AuthRuntime {
    AuthRuntime {
        jwt: JwtVerifier::new(
            JwtConfig::default(),
            KeyRing::new(raw.auth.jwt.secret.clone()),
        ),
        tokens: RefreshTokenManager::new(
            MySqlRefreshTokenStore::new(pool.clone()),
            raw.auth.jwt.secret.clone(),
            Duration::from_secs(raw.auth.jwt.expires_in_seconds),
            // refresh token 有效期仍是硬编码 30 天(原实现如此)
            Duration::from_secs(30 * 24 * 3600),
        ),
    }
}
