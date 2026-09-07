import adapterUniapp from '@alova/adapter-uniapp'
import { createAlova } from 'alova'
import { createApis, mountApis, withConfigType } from './createApis'
import { decodeJwtExp, useAuthStore } from '@/stores/auth'
import { PP } from '@/router/config'

export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL

export class ApiError extends Error {
  code: string | number
  data: unknown

  constructor(code: string | number, message: string, data?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.data = data
  }
}

interface StoredAuth {
  userId: number
  token: string
  refreshToken: string
  expiresAt: number
}

function readAuthFromStorage(): StoredAuth | null {
  try {
    const raw = uni.getStorageSync('token')
    if (!raw)
      return null
    return JSON.parse(raw) as StoredAuth
  }
  catch {
    return null
  }
}

function clearAuthAndRedirect() {
  try {
    useAuthStore().logout()
  }
  catch {
    /* 忽略 store 未初始化场景 */
  }
  const pages = getCurrentPages()
  const currentPath = pages.length > 0 ? `/${pages[pages.length - 1]!.route}` : ''
  if (currentPath !== PP.LOGIN) {
    uni.reLaunch({ url: PP.LOGIN })
  }
}

let refreshPromise: Promise<boolean> | null = null

/** 401 单飞刷新:并发 401 只发一次 Auth__jwtRefresh,成功更新存储后重放 */
async function tryRefreshToken(): Promise<boolean> {
  if (refreshPromise)
    return refreshPromise
  refreshPromise = (async () => {
    try {
      const auth = readAuthFromStorage()
      if (!auth?.refreshToken || !auth?.userId)
        return false
      const resp = (await Apis.auth.Auth__jwtRefresh({
        data: { refresh_token: auth.refreshToken },
      })) as unknown as { access_token: string; refresh_token: string }
      if (!resp?.access_token)
        return false
      const next: StoredAuth = {
        ...auth,
        token: resp.access_token,
        refreshToken: resp.refresh_token || auth.refreshToken,
        expiresAt: decodeJwtExp(resp.access_token),
      }
      uni.setStorageSync('token', JSON.stringify(next))
      useAuthStore().state = next
      return true
    }
    catch {
      return false
    }
  })()
  try {
    return await refreshPromise
  }
  finally {
    refreshPromise = null
  }
}

/** 解包 hoshiyomi 信封:code 为数字 0 表示成功,否则抛 ApiError(中文 message) */
function extractBizData(raw: unknown): unknown {
  if (typeof raw === 'string') {
    try {
      raw = JSON.parse(raw)
    }
    catch {
      return raw
    }
  }
  if (!raw || typeof raw !== 'object')
    return raw
  const body = raw as { code?: number; data?: unknown; message?: string }
  if (typeof body.code === 'number' && body.code !== 0) {
    throw new ApiError(body.code, body.message || '请求失败', body.data)
  }
  return body.data !== undefined ? body.data : raw
}

function methodUrl(method: unknown): string {
  if (!method || typeof method !== 'object')
    return ''
  const m = method as Record<string, unknown>
  const config = m.config as Record<string, unknown> | undefined
  return (config?.url || m.url || '') as string
}

/** 登录/注册请求:无令牌可用,401/403 直接走业务错误,不触发刷新 */
function isAuthRequest(method: unknown): boolean {
  const url = methodUrl(method)
  return url.includes('/auth/jwt/login') || url.includes('/auth/register')
}

function isRefreshRequest(method: unknown): boolean {
  return methodUrl(method).includes('/auth/jwt/refresh')
}

export const alovaInstance = createAlova({
  baseURL: API_BASE_URL,
  ...adapterUniapp(),
  beforeRequest(method) {
    try {
      const auth = readAuthFromStorage()
      if (auth?.token) {
        method.config.headers.Authorization = `Bearer ${auth.token}`
      }
    }
    catch {
      /* 忽略存储读取失败 */
    }
  },
  cacheFor: null,
  responded: {
    async onSuccess(response, method) {
      const status = (response as { statusCode?: number; status?: number; data?: unknown })?.statusCode
        ?? (response as { status?: number })?.status
      if (status === 401) {
        if (isAuthRequest(method)) {
          const rawData = (response as { data?: unknown })?.data
          return extractBizData(rawData)
        }
        if (isRefreshRequest(method)) {
          clearAuthAndRedirect()
          throw new Error('会话已过期')
        }
        const refreshed = await tryRefreshToken()
        if (refreshed)
          return method.send()
        clearAuthAndRedirect()
        throw new Error('会话已过期')
      }
      const rawData = (response as { data?: unknown })?.data
      if (typeof status === 'number' && status >= 400) {
        extractBizData(rawData) // 信封带业务错误码时抛 ApiError
        throw new ApiError(`http.${status}`, `请求失败(HTTP ${status})`)
      }
      return extractBizData(rawData)
    },
    async onError(error, method) {
      const status = (error as { statusCode?: number; status?: number })?.statusCode
        ?? (error as { status?: number })?.status
      if (status === 401) {
        if (isAuthRequest(method)) {
          throw error
        }
        if (isRefreshRequest(method)) {
          clearAuthAndRedirect()
          throw error
        }
        const refreshed = await tryRefreshToken()
        if (refreshed)
          return method.send()
        clearAuthAndRedirect()
      }
      throw error
    },
  },
})

export const $$userConfigMap = withConfigType({})

const Apis = createApis(alovaInstance, $$userConfigMap)

mountApis(Apis)

export default Apis
