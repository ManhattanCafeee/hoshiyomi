import type { RouteLocationAsString } from 'vue-router'

export interface AuthAdapter {
  url: {
    login: RouteLocationAsString
    home: RouteLocationAsString
  }
  init(): Promise<void>
  isAuthenticated(): boolean
  getPermissions(): string[]
  checkPermission(permissions: string[], required: string | string[]): boolean
}

export type AuthStrategy = 'authenticated' | 'guest' | 'optional'

export interface AuthMeta {
  required?: boolean
  guest?: boolean
  permissions?: string | string[]
  forbidden?: RouteLocationAsString
  redirect?: {
    authed?: RouteLocationAsString
    guest?: RouteLocationAsString
  }
}
