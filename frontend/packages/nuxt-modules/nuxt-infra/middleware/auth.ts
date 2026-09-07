import { createError, defineNuxtRouteMiddleware, navigateTo, useState } from 'nuxt/app'
import { useAuthAdapter } from '../composables/authAdapter'
import type { AuthMeta, AuthStrategy } from '../types/auth'

function normalizeShorthand(raw: AuthStrategy): AuthMeta | null {
  if (raw === 'optional') return null
  if (raw === 'guest') return { guest: true }
  if (raw === 'authenticated') return { required: true }
  return null
}

export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.prerender) return

  const raw = to.meta.auth as AuthStrategy | AuthMeta | undefined
  if (!raw) return

  const meta = typeof raw === 'string' ? normalizeShorthand(raw) : raw
  if (!meta) return

  const adapter = useAuthAdapter()

  // 会话 Cookie 流:登录态无本地持久化,首次客户端导航必须先等待
  // adapter.init()(GET /auth/me)恢复,再决定是否放行(参考实现依赖
  // localStorage 同步恢复,不适用于会话认证)。
  const authInit = useState('authInit', () => false)
  if (import.meta.client && !authInit.value) {
    authInit.value = true
    await adapter.init()
  }

  const isAuthed = adapter.isAuthenticated()

  if (meta.redirect) {
    if (meta.redirect.authed && isAuthed) return navigateTo(meta.redirect.authed)
    if (meta.redirect.guest && !isAuthed) return navigateTo(meta.redirect.guest)
  }

  if (meta.guest && isAuthed) return navigateTo(adapter.url.home)
  if (meta.required && !isAuthed) return navigateTo(adapter.url.login)

  if (meta.permissions && isAuthed) {
    const required = Array.isArray(meta.permissions) ? meta.permissions : [meta.permissions]
    if (!adapter.checkPermission(adapter.getPermissions(), required)) {
      if (meta.forbidden) return navigateTo(meta.forbidden)
      throw createError({ statusCode: 403, fatal: true })
    }
  }
})
