import { defineNuxtModule } from '@nuxt/kit'

export default defineNuxtModule({
  meta: {
    name: '@hoshiyomi/shadcn',
  },

  moduleDependencies: {
    'shadcn-nuxt': {
      // defaults: {
      //   /**
      //    * Prefix for all the imported component.
      //    * @default "Ui"
      //    */
      //   prefix: 'Ui',
      //   /**
      //    * Directory that the component lives in.
      //    * Will respect the Nuxt aliases.
      //    * @link https://nuxt.com/docs/api/nuxt-config#alias
      //    * @default "@/components/ui"
      //    */
      //   componentDir: '@/components/ui',
      // },
    },
  },

  setup(_options, _nuxt) {
    // const resolver = createResolver(import.meta.url)
    // Add components
    // addComponentsDir({
    //   path: resolver.resolve('./components')
    // })
  },
})
