import { defineConfig } from '@alova/wormhole'

const outputDir = 'src/api/generated'

export default defineConfig({
  generator: [
    {
      input: '../backend/docs/openapi.json',
      // 不使用 defaultsPlugin/toPlugin:utoipa 将纯响应 schema 内联进 ApiResponse_* 信封,
      // 两个插件生成的 defaults.ts/to.ts 会引用未导出的类型名,且本项目无代码消费这两个文件
      plugins: [],
      platform: 'swagger',
      output: outputDir,
      type: 'ts',
    },
  ],
})
