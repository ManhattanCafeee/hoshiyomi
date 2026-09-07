import { presetUni } from '@uni-helper/unocss-preset-uni'
import {
  defineConfig,
  presetIcons,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'

export default defineConfig({
  theme: {
    colors: {
      'app-primary': '#1a1a1a',
      'app-bg': '#fafafa',
      'app-card': '#ffffff',
      'app-border': '#e4e4e4',
      'app-divider': '#ededed',
      'app-text': '#0a0a0a',
      'app-text-sub': '#555555',
      'app-text-hint': '#8a8a8a',
      'app-placeholder': '#c8c8c8',
      'app-success': '#1a1a1a',
      'app-warning': '#b8860b',
      'app-danger': '#8b2c2c',
      'app-info': '#8a8a8a',
    },
  },
  presets: [
    presetUni(),
    presetIcons({
      scale: 1.2,
      warn: true,
      extraProperties: {
        'display': 'inline-block',
        'vertical-align': 'middle',
      },
    }),
  ],
  transformers: [transformerDirectives(), transformerVariantGroup()],
})
