import { BizError } from '@hoshiyomi/alova/lib'
import type { AuthState } from '@hoshiyomi/apisdk/stores/authState'

let handling401 = false

export default defineNuxtPlugin(() => {
  const authState = useAuthState()
  const notify = useNotify()

  // 全局 401 拦截:会话过期 → 提示 + 退出 + 跳登录(滑动续期由后端中间件负责,无需刷新令牌)
  eventSystem.subscribe('request:error', async ({ error }: { error: unknown }) => {
    if (!(error instanceof BizError)) return
    if (error.response?.status !== 401) return
    if (window.location.pathname === '/login') return
    if (handling401) return
    handling401 = true

    notify.error('会话已过期,请重新登录')
    authState.logout()
    await navigateTo('/login')
    handling401 = false
  })

  defineAuthAdapter({
    url: { login: '/login', home: '/' },
    init: async () => {
      // 会话 Cookie 是唯一真源:首次导航时拉取 /auth/me 恢复登录态
      try {
        const data = (await Apis.auth
          .Auth__me({
            meta: { noMessage: () => true },
          })
          .send()) as unknown as AuthState
        authState.setAuthState(data)
      } catch {
        authState.logout()
      }
    },
    isAuthenticated: () => !!authState.user(),
    getPermissions: () => authState.permissions() ?? [],
    checkPermission: (permissions: string[], required: string | string[]) => {
      const req = Array.isArray(required) ? required : [required]
      return req.every((p) =>
        permissions.some((g) => {
          if (g === '*' || g === p) return true
          if (g.endsWith('.*')) return p.startsWith(g.slice(0, -2) + '.')
          return false
        }),
      )
    },
  })
})
