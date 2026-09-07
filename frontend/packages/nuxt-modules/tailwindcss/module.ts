import { createResolver, defineNuxtModule } from '@nuxt/kit'
import tailwindcss from '@tailwindcss/vite'
import defu from 'defu'

export default defineNuxtModule({
  meta: {
    name: '@hoshiyomi/tailwindcss',
  },

  moduleDependencies: {},

  async setup(options, nuxt) {
    const _resolver = createResolver(import.meta.url)

    nuxt.options.vite = defu(nuxt.options.vite, {
      plugins: [tailwindcss()],
    })
  },
})
