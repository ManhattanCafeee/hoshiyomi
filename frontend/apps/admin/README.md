# hoshiyomi 管理后台

Nuxt 4 SPA 管理后台:会话 Cookie 认证(同源),alova 生成 SDK(`@hoshiyomi/apisdk`),shadcn-vue + Tailwind v4 UI。

- `pnpm dev` 开发(代理 `/api/v1` → `http://127.0.0.1:8080`);`pnpm build` 构建;`pnpm type-check` 类型检查。
- 代码生成:后端 `cargo run --example dump_openapi > ../backend/docs/openapi.json` 后,在工作区根执行 `pnpm gen:api`。
- 页面:登录、仪表盘、用户管理(需 `user:read`/`user:write`/`user:delete`)、个人资料;角色管理为 CLI-only,无前端页面。
