import { addImports, addImportsDir, addPlugin, createResolver, defineNuxtModule } from '@nuxt/kit'

export default defineNuxtModule({
  meta: {
    name: '@hoshiyomi/apisdk',
  },

  hooks: {
    'prepare:types': ({ references }) => {
      references.push({
        types: '@hoshiyomi/apisdk/types',
      })
    },
  },

  async setup(_options, _nuxt) {
    const resolver = createResolver(import.meta.url)

    // 将 alova baseURL 指向 runtimeConfig.public.apiBase
    addPlugin({
      src: resolver.resolve('./runtime/apiBase'),
      mode: 'client',
    })

    addImports([
      { from: resolver.resolve('./lib/api'), name: 'default', as: 'Apis' },
      { from: resolver.resolve('./lib/api'), name: 'alovaInstance' },
      { from: resolver.resolve('./lib/api'), name: 'eventSystem' },
    ])

    addImportsDir([
      resolver.resolve('./lib/domain/models'),
      resolver.resolve('./lib/domain/choices'),
      resolver.resolve('./composables'),
      resolver.resolve('./stores'),
      resolver.resolve('./utils'),
    ])
  },
})
