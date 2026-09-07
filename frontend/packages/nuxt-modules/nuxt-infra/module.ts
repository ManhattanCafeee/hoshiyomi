import type { ModuleOptions as NuxtIconOptions } from '@nuxt/icon'
import {
  addComponentsDir,
  addImportsDir,
  addPlugin,
  addRouteMiddleware,
  createResolver,
  defineNuxtModule,
} from '@nuxt/kit'
import type { ModuleOptions as DayjsOptions } from 'dayjs-nuxt'
import { defu } from 'defu'
import type { ModuleOptions as NuxtOgImageOptions } from 'nuxt-og-image'
import type { ModuleOptions } from './types/module'

export default defineNuxtModule<ModuleOptions>({
  meta: {
    name: '@hoshiyomi/nuxt-infra',
    configKey: 'hmeqoNuxtInfra',
  },

  defaults: {
    routeAuth: {
      defaultRedirect: false,
    },
  },

  hooks: {
    'prepare:types': ({ references }) => {
      references.push({
        types: '@hoshiyomi/nuxt-infra/types',
      })
    },
  },

  moduleDependencies: {
    '@nuxt/fonts': {},
    '@nuxt/eslint': {},
    '@nuxt/test-utils': {},
    '@nuxtjs/seo': {},
    'nuxt-og-image': {
      defaults: <Partial<NuxtOgImageOptions>>{
        enabled: false,
      },
    },
    '@nuxtjs/device': {},
    '@nuxt/icon': {
      defaults: <Partial<NuxtIconOptions>>{
        mode: 'svg',
        provider: 'server',
      },
    },
    '@vueuse/nuxt': {},
    'dayjs-nuxt': {
      defaults: <Partial<DayjsOptions>>{
        locales: ['en', 'zh-cn'],
        plugins: ['relativeTime', 'utc', 'timezone', 'quarterOfYear', 'localizedFormat'],
        defaultLocale: 'zh-cn',
        defaultTimezone: 'Asia/Shanghai',
      },
    },
    '@hoshiyomi/util': {},
    '@hoshiyomi/nuxt-color-mode': {},
  },

  setup(options, nuxt) {
    const resolver = createResolver(import.meta.url)

    nuxt.options.experimental.typedPages = true
    nuxt.options.runtimeConfig.public = defu(nuxt.options.runtimeConfig.public, {
      hmeqoNuxtInfra: options,
    })

    addComponentsDir({
      path: resolver.resolve('./components'),
    })

    addPlugin(resolver.resolve('./plugins/error.ts'))

    addImportsDir([resolver.resolve('./composables'), resolver.resolve('./utils'), resolver.resolve('./stores')])

    addRouteMiddleware({
      name: 'auth',
      path: resolver.resolve('./middleware/auth.ts'),
      global: true,
    })
  },
})
