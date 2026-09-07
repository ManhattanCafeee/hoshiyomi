// 会话认证下的登录态:仅存内存(Cookie 是唯一真源),
// 页面刷新后由 authAdapter.init() 调用 GET /auth/me 恢复。
export interface AuthUser {
  id: number
  username: string
  email: string
  created_at: string
  updated_at: string
}

export interface AuthState {
  user: AuthUser
  permissions: string[]
}

export const useAuthState = defineCachedFn(() => {
  const state = useState<AuthState | null>('auth', () => null)

  const actions = {
    state() {
      return state.value
    },
    setAuthState(s: AuthState) {
      state.value = { user: s.user, permissions: s.permissions ?? [] }
    },
    logout() {
      state.value = null
    },
    user() {
      return state.value?.user ?? null
    },
    userChecked() {
      if (!state.value?.user) throw new Error('未登录')
      return state.value.user
    },
    isSelf(userId: number) {
      return state.value?.user?.id === userId
    },
    permissions() {
      return state.value?.permissions ?? []
    },
  }
  return actions
})
