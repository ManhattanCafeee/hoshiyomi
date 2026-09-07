import { defineConfig } from '@uni-helper/unh'

/**
 * unh config file
 * See https://uni-helper.js.org/unh/ for more config options.
 */
export default defineConfig({
  platform: {
    // Default platform
    default: 'h5',
    // Platform aliases
    alias: {
      'h5': ['w', 'h'],
      'mp-weixin': 'wx',
    },
  },
  autoGenerate: {
    pages: true,
    manifest: true,
  },
})
