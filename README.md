# hoshiyomi

Rust/axum 后端 + Nuxt 管理后台 + uni-app 移动端的 monorepo。

```
hoshiyomi/
├── backend/    # Rust 单 crate:axum + sqlx(MySQL)+ utoipa,CLI 管理命令
├── frontend/   # pnpm 工作区:Nuxt 4 SPA 管理后台 + 生成的 alova SDK + 内嵌 Nuxt 模块
├── uniapp/     # 独立 pnpm uni-app 移动端骨架(H5 开发,mp-weixin 构建)
└── docker-compose.yml   # 本地 MySQL 8.4
```

## 快速开始

```bash
# 1. 基础设施(MySQL:root/changeme,库 hoshiyomi,账号 hoshiyomi/password)
docker compose up -d

# 2. 后端(默认 0.0.0.0:8080,首次启动自动建表)
#    根 Cargo.toml 为虚拟工作区:build/test/check 可在根执行;cargo run 需 cd backend 或加 -p hoshiyomi
cd backend
cargo run -- init                                          # 初始化默认角色(幂等)
cargo run -- create-superuser -u admin -p 'Admin123!' -e admin@example.com
cargo run                                                   # 启动服务

# 3. 管理后台(:3000,开发代理 /api/v1 → 127.0.0.1:8080)
cd frontend && pnpm install && pnpm gen:api
pnpm --filter admin dev

# 4. uniapp(H5 :5173;独立目录、独立锁文件,勿与 frontend 混用)
cd uniapp && pnpm install && pnpm gen:api
pnpm dev
```

API 文档:<http://127.0.0.1:8080/api-docs/scalar>(Scalar)与 <http://127.0.0.1:8080/api-docs/swagger-ui>。

## 代码生成链

前后端契约的唯一来源是后端 utoipa 注解,两端 SDK 由同一份规范生成:

```bash
cd backend && cargo run --example dump_openapi > docs/openapi.json   # 导出规范(无需数据库)
cd frontend && pnpm gen:api      # → packages/apisdk/lib/api(生成文件勿手改)
cd uniapp && pnpm gen:api        # → src/api/generated(index.ts 由 .handwritten 恢复)
```

接口契约变更时,同一提交内更新 `backend/docs/openapi.json` 并重新生成两端 SDK。

## 端口与凭据

| 组件 | 地址 | 凭据 |
| --- | --- | --- |
| MySQL | 127.0.0.1:3306 | root/changeme;hoshiyomi/password |
| API | 127.0.0.1:8080 | 演示超级用户 admin/Admin123!(仅本地) |
| 管理后台 | 127.0.0.1:3000 | 会话 Cookie 登录 |
| uniapp H5 | 127.0.0.1:5173 | JWT 登录 |

## 生产部署注记

- **管理后台须与 API 同源或反代 `/api/v1`**:`/users` 系列接口走会话 Cookie,后端 CORS 不携带凭据,跨源部署时 Cookie 不可用。
- **uniapp**:构建时设置 `VITE_API_BASE_URL` 指向真实 API 域名;微信小程序要求 https 且域名加入白名单。
- 生产环境务必更换 `JWT_SECRET`(默认 `change-me-in-production`)与数据库密码。

## 目录说明

- `backend/`:Rust 后端,细节见 `backend/README.md`(API 总览、CLI、配置)。
- `frontend/`:管理后台工作区,细节见 `frontend/apps/admin/README.md`。
- `uniapp/`:移动端骨架,细节见 `uniapp/README.md`。

## 许可证

[MIT](LICENSE)
