import { addImportsDir, createResolver, defineNuxtModule } from '@nuxt/kit'

export default defineNuxtModule({
  meta: {
    name: '@hoshiyomi/alova',
  },
  moduleDependencies: {
    '@hoshiyomi/util': {},
  },

  hooks: {
    'prepare:types': ({ references }) => {
      references.push({
        types: '@hoshiyomi/alova/types',
      })
    },
  },

  async setup(_options, _nuxt) {
    const resolver = createResolver(import.meta.url)

    // Add utils
    addImportsDir(resolver.resolve('./utils'))
  },
})
