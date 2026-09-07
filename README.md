# hoshiyomi

基于 Rust 的 MySQL 后端 JSON REST API 服务:用户管理、RBAC 权限控制、双认证(会话 Cookie + JWT 及刷新令牌轮换)、自动生成的 OpenAPI 文档,以及一套管理用命令行工具。

## 特性

- **用户管理**:注册、登录、查询、分页列表、修改用户名、修改密码、删除
- **RBAC 权限**:角色-权限 JSON 模型,权限码支持通配符(`*`、`user:*`);内置 superuser / admin / user 三个默认角色
- **双认证体系**
  - 会话认证:服务端会话存储(UUID)、HttpOnly Cookie、滑动续期
  - JWT 认证:HS256 访问令牌 + 一次性轮换刷新令牌(并发重放返回 401)
- **安全细节**:Argon2 密码哈希、未知用户名登录时恒定时间校验、内部错误信息不外泄、MySQL 唯一键竞态回退 409
- **OpenAPI 文档**:utoipa 自动生成,Scalar + Swagger UI 双界面
- **管理 CLI**:角色与超级用户不通过 HTTP 暴露,由命令行管理

## 技术栈

| 层 | 选型 |
| --- | --- |
| Web 框架 / 运行时 | axum 0.8 · tokio |
| 数据库 | MySQL · sqlx 0.9(迁移内嵌二进制,启动时自动执行) |
| 认证安全 | argon2 · jsonwebtoken · uuid |
| 参数校验 | validator |
| 配置 | config + dotenvy(默认值 → config.toml → 环境变量分层) |
| 日志 | tracing(标准输出 + 按日滚动 JSON 文件) |
| CLI | clap · inquire |
| OpenAPI | utoipa 5 + utoipa-axum + Scalar / Swagger UI |

## 快速开始

前置要求:

- Rust ≥ 1.94(edition 2024;由 sqlx 0.9 决定的最低版本)
- MySQL 8+(已建库,如 `hoshiyomi`)

```bash
# 1. 准备环境变量(默认连接 mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi)
cp .env.example .env

# 2. 初始化默认角色(superuser / admin / user,幂等)
cargo run -- init

# 3. 创建超级用户(缺省参数时交互式输入)
cargo run -- create-superuser

# 4. 启动服务(默认 0.0.0.0:8080,启动时自动执行迁移)
cargo run
```

启动后访问:

- API 文档:Scalar UI <http://127.0.0.1:8080/api-docs/scalar> · Swagger UI <http://127.0.0.1:8080/api-docs/swagger-ui>
- OpenAPI 规范:<http://127.0.0.1:8080/api-docs/openapi.json>
- 健康检查:<http://127.0.0.1:8080/api/v1/health>

## 命令行

无子命令时启动 HTTP 服务;子命令为管理操作。除 `config`、`perms` 外均需连接数据库(不会执行迁移,假定 schema 已由首次 `serve` 创建)。

| 命令 | 说明 |
| --- | --- |
| `hoshiyomi` | 启动服务 |
| `hoshiyomi config` | 打印解析后的配置 JSON |
| `hoshiyomi init` | 初始化默认角色(幂等) |
| `hoshiyomi create-superuser [-u 用户名] [-p 密码] [-e 邮箱]` | 创建超级用户,缺省参数时交互式输入 |
| `hoshiyomi role list` | 列出所有角色 |
| `hoshiyomi role create --name 角色名 [--description 描述] [--perms 权限码]` | 创建角色,权限码逗号分隔,如 `user:read,user:write` |
| `hoshiyomi role delete --name 角色名` | 删除角色 |
| `hoshiyomi perms` | 列出全部权限码 |

## API 概览

统一前缀 `/api/v1`,统一响应封装:

```json
{ "code": 0, "message": "成功", "data": { } }
```

- `code = 0` 表示成功,否则等于 HTTP 状态码(400/401/403/404/409/500)
- `errors` 字段仅在校验失败时出现(字段级错误详情)
- 分页接口返回 `PageData { items, total, page, per_page }`

### 会话认证接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/v1/auth/register` | 注册(自动赋予 user 角色) |
| POST | `/api/v1/auth/login` | 登录,设置会话 Cookie |
| GET | `/api/v1/auth/me` | 当前用户及权限 |
| POST | `/api/v1/auth/logout` | 退出,注销会话 |

### JWT 认证接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/v1/auth/jwt/login` | 登录,返回访问令牌 + 刷新令牌 |
| POST | `/api/v1/auth/jwt/refresh` | 刷新令牌(一次性轮换) |
| POST | `/api/v1/auth/jwt/logout` | 撤销全部刷新令牌 |
| GET | `/api/v1/auth/jwt/me` | Bearer 认证下的当前用户及权限 |
| GET | `/api/v1/auth/jwt/echo` | 认证检查示例 |

### 用户管理接口(需 `user:*` 权限)

| 方法 | 路径 | 说明 | 权限 |
| --- | --- | --- | --- |
| GET | `/api/v1/users?page=&per_page=` | 分页列表 | `user:read` |
| POST | `/api/v1/users` | 创建用户 | `user:write` |
| GET | `/api/v1/users/{id}` | 用户详情 | `user:read` |
| PUT | `/api/v1/users/{id}/username` | 修改用户名 | `user:write` |
| PUT | `/api/v1/users/{id}/password` | 修改密码 | `user:write` |
| DELETE | `/api/v1/users/{id}` | 删除用户 | `user:delete` |

### 认证方式

- **会话**:`POST /auth/login` 响应携带 `Set-Cookie: session_id=…`(HttpOnly, SameSite=Lax,24h TTL,滑动续期),后续请求携带 Cookie 即可。
- **JWT**:`POST /auth/jwt/login` 返回 `access_token`(900s)与 `refresh_token`(30 天,一次性);访问受保护接口时携带 `Authorization: Bearer <access_token>`。

## 权限模型

- 权限码格式:`<资源>:<操作>`,支持通配符 `*`(全部)与 `<资源>:*`(同资源全部)。
- 内置权限码:`*`、`user:read`、`user:write`、`user:delete`、`user:*`、`role:read`、`role:write`、`role:delete`、`role:*`(执行 `hoshiyomi perms` 查看完整列表)。
- 默认角色:superuser(`*`)、admin(`user:*` + `role:*`)、user(`user:read`)。
- 用户的权限 = 其所有角色权限的并集;用户与角色为多对多关系(经 `user_roles` 表)。

## 配置

加载优先级(后者覆盖前者):

1. 代码默认值(`src/config/schema.rs`)
2. `config.toml`(配置目录,或本地模式下的 `./.hoshiyomi/config.toml`)
3. `HOSHIYOMI__` 前缀环境变量(双下划线分层,如 `HOSHIYOMI__AUTH__JWT__SECRET`)
4. 旧版扁平环境变量兼容层(见下表)

`.env` 文件由 dotenvy 加载(可缺省)。

| 环境变量 | 默认值 | 说明 |
| --- | --- | --- |
| `DATABASE_URL` | `mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi` | MySQL 连接串 |
| `HOST` | `0.0.0.0` | 监听地址 |
| `PORT` | `8080` | 监听端口 |
| `RUST_LOG` / `LOG_LEVEL` | `info` | 日志级别 |
| `JWT_SECRET` | `change-me-in-production` | JWT 签名密钥(生产必须更换) |
| `JWT_EXPIRES_IN_SECONDS` | `900` | 访问令牌有效期 |
| `SESSION_COOKIE_NAME` | `session_id` | 会话 Cookie 名 |
| `SESSION_TTL_HOURS` | `24` | 会话有效期 |
| `HOSHIYOMI_LOCAL_MODE` | 未设置 | 本地模式:配置与日志置于 `./.hoshiyomi/` |

## 开发

```bash
cargo build              # 构建
cargo test               # 测试(无需数据库、无需环境变量)
cargo fmt                # 格式化(纯默认配置)
cargo clippy             # Clippy(纯默认配置)
```

要点:

- **迁移**:位于 `migrations/`,命名格式 `{YYYYMMDDHHMMSS}_{描述}.sql`;由 `sqlx::migrate!` 内嵌进二进制,服务启动时自动执行。CLI 子命令不执行迁移。
- **时间**:数据库连接统一 `UTC` 会话时区,SQL 写入用 `UTC_TIMESTAMP()`,代码读取用 `DateTime<Utc>`。
- **测试**:集成测试(`tests/api.rs`)以进程内 `oneshot` 方式驱动 Router,使用惰性连接池(不要求真实数据库);单元测试内联在源文件 `#[cfg(test)]` 模块中。
- **错误处理**:单一 `AppError` + `ErrorKind`(`src/error.rs`),内部错误原因不向客户端泄露。

## 项目结构

```rust
src/
├── main.rs            # 二进制入口 → cli::run()
├── lib.rs             # build_app:路由组装(含 /api-docs、中间件、fallback)
├── serve.rs           # 服务启动:配置 → 日志 → 连接/迁移 → 监听
├── config/            # 分层配置(schema.rs 为默认值来源)
├── cli/               # clap 命令定义与实现
├── db.rs              # 连接池(UTC 时区)与迁移执行
├── error.rs           # AppError / ErrorKind / bail! / map_duplicate_key
├── state.rs           # AppState + Services 依赖注入容器
├── infra.rs           # tracing 初始化
├── common/            # ApiResponse 响应封装、校验提取器(AppPath/AppQuery/AppJson)
├── middleware/        # CORS、会话 Cookie 滑动续期
├── util/              # Argon2 密码工具
└── modules/
    ├── auth/          # 注册/登录/会话/JWT/刷新令牌(extractor/service/token/session)
    ├── user/          # 用户 CRUD
    └── role/          # 角色与权限(仅 CLI 管理,无 HTTP 接口)
migrations/            # SQL 迁移(内嵌、启动时执行)
tests/                 # 集成测试
```

每个领域模块遵循四层结构:`handlers.rs`(HTTP + utoipa)→ `service.rs`(业务逻辑)→ `repository.rs`(裸 sqlx 自由函数)→ `models.rs`(DTO 与 FromRow 结构)。

## 许可证

[MIT](LICENSE)
