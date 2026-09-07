# hoshiyomi-uniapp

基于 vitesse-uni-app(uni-helper 工具链)的 uni-app 移动端骨架:JWT 认证(单飞刷新)、工作台、我的、注册/登录。

- `pnpm dev`(H5)/ `pnpm dev wx`(微信小程序)/ `pnpm build` 构建;`pnpm type-check`、`pnpm lint` 检查。
- 代码生成:后端 `cargo run --example dump_openapi > ../backend/docs/openapi.json` 后执行 `pnpm gen:api`(`index.ts` 由 `.handwritten` 恢复)。
- 环境变量:`VITE_API_BASE_URL`(见 `.env.example`,小程序生产需 https + 域名白名单)。
