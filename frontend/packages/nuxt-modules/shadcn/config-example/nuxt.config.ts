import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
  modules: ['@hoshiyomi/nuxt-color-mode', '@hoshiyomi/tailwindcss', '@hoshiyomi/shadcn'],
  colorMode: {
    preference: 'dark',
  },
  vite: {
    plugins: [tailwindcss()],
  },
})
