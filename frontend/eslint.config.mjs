// @ts-check
import { withNuxt } from './apps/admin/.nuxt/eslint.config.mjs'

export default withNuxt(
  // Global ignores for auto-generated code
  {
    ignores: [
      'packages/apisdk/lib/api/defaults.ts',
      'packages/apisdk/lib/api/to.ts',
      'packages/apisdk/lib/api/apiDefinitions.ts',
    ],
  },
  // UI components and pages — single-word names are by design
  {
    files: [
      'apps/admin/app/components/ui/**/*.vue',
      'apps/admin/app/pages/**/*.vue',
      'apps/admin/app/layouts/**/*.vue',
      'packages/nuxt-modules/*/components/**/*.vue',
    ],
    rules: {
      'vue/multi-word-component-names': 'off',
      'vue/require-default-prop': 'off',
    },
  },
  // Prettier outputs self-closing void elements (<input />); keep in sync with prettier
  {
    files: ['**/*.vue'],
    rules: {
      'vue/html-self-closing': [
        'warn',
        { html: { void: 'always', normal: 'never', component: 'always' }, svg: 'always', math: 'always' },
      ],
    },
  },
)
