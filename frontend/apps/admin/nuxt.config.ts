// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  srcDir: 'app/',
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },

  runtimeConfig: {
    public: {
      // 留空 = 同源相对路径(开发走 Vite 代理 /api/v1,生产由 Nginx 同源反代);
      // 跨源部署时填后端源站(不带尾斜杠),或在构建时用 NUXT_PUBLIC_API_BASE 覆盖。
      apiBase: '',
    },
  },
  modules: [
    '@hoshiyomi/nuxt-infra',
    '@hoshiyomi/tailwindcss',
    '@hoshiyomi/shadcn',
    '@hoshiyomi/alova',
    '@hoshiyomi/apisdk',
  ],
  shadcn: {
    prefix: '',
  },
  components: [{ path: '~/components', pathPrefix: false }],
  app: {
    buildAssetsDir: 'static',
    rootAttrs: {
      id: 'root',
    },
    head: {
      link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
    },
  },
  css: ['~/assets/css/tailwind.css', '~/assets/css/utilities.css', '~/assets/css/transition.css'],
  routeRules: {
    '/**': {
      ssr: false,
      prerender: true,
    },
  },

  colorMode: {
    preference: 'system',
  },

  nitro: {
    compressPublicAssets: true,
    prerender: {
      routes: ['/', '/login'],
    },
  },
  vite: {
    resolve: {
      alias: [
        { find: /^dayjs$/, replacement: 'dayjs/esm/index.js' },
        { find: /^dayjs\/plugin\/(.+)$/, replacement: 'dayjs/esm/plugin/$1/index.js' },
        { find: /^dayjs\/locale\/(.+)$/, replacement: 'dayjs/esm/locale/$1.js' },
      ],
      dedupe: ['alova'],
    },
    server: {
      proxy: {
        '/api/v1': {
          target: 'http://127.0.0.1:8080',
          changeOrigin: true,
        },
        '/api-docs': {
          target: 'http://127.0.0.1:8080',
          changeOrigin: true,
        },
      },
    },
    build: {
      target: 'es2020',
      terserOptions: {
        compress: {
          drop_console: true,
        },
      },
    },
  },

  typescript: {
    tsConfig: {
      compilerOptions: {},
    },
  },
})
