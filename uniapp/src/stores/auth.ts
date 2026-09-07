import { defineStore } from 'pinia'

export interface TokenState {
  userId: number
  token: string
  refreshToken: string
  expiresAt: number
}

/** 从 JWT payload 解码 exp(秒),失败回退为 15 分钟(与后端 access token 默认 TTL 一致) */
export function decodeJwtExp(token: string): number {
  const now = Date.now()
  try {
    const payload = token.split('.')[1]!.replace(/-/g, '+').replace(/_/g, '/')
    const json = decodeURIComponent(
      Array.from(atob(payload))
        .map(c => `%${(`00${c.charCodeAt(0).toString(16)}`).slice(-2)}`)
        .join(''),
    )
    const exp = JSON.parse(json).exp as number | undefined
    if (exp)
      return exp * 1000
  }
  catch {
    /* 解析失败走回退 */
  }
  return now + 900000
}

function readPersisted(): TokenState | null {
  try {
    const raw = uni.getStorageSync('token')
    if (!raw)
      return null
    return JSON.parse(raw) as TokenState
  }
  catch {
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const state = ref<TokenState | null>(readPersisted())

  // 不校验过期:access token 过期由 API 层 401 单飞刷新兜底
  const isLoggedIn = computed(() => !!state.value?.token)

  const tokenValue = computed(() => state.value?.token ?? '')

  function setState(s: TokenState) {
    state.value = s
    uni.setStorageSync('token', JSON.stringify(s))
  }

  function logout() {
    state.value = null
    uni.removeStorageSync('token')
    uni.removeStorageSync('user')
  }

  return { state, isLoggedIn, tokenValue, setState, logout }
})
