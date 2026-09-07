import type { HTMLAttributes } from 'vue'
import type { AuthMeta, AuthStrategy } from './auth'

export {}

declare global {
  type StrBoolean = 'true' | 'false'

  type HTMLAttrClass = HTMLAttributes['class']
}

declare module 'nuxt/app' {
  interface PageMeta {
    title?: string
    tags?: string[]
    icon?: string
    auth?: AuthStrategy | AuthMeta
  }
}
