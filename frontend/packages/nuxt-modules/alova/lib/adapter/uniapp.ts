import type { Method } from 'alova'
import type { RequestInfo, ResponseInfo } from '../event'

export const toRequestInfo = (method: Method): RequestInfo => ({
  get method() {
    return method.type
  },
  get data() {
    return method.data
  },
  set data(v) {
    method.data = v
  },
  get meta() {
    return method.meta as Record<string, unknown>
  },
  get headers() {
    return method.config.headers
  },
  get credentials() {
    return method.config.credentials
  },
  set credentials(v) {
    method.config.credentials = v
  },
})

export interface UniAppResponse {
  data?: unknown
  statusCode: number
  header?: Record<string, string>
  errMsg?: string
}

export function toResponseInfo(response: UniAppResponse): ResponseInfo {
  return {
    status: response.statusCode,
    ok: response.statusCode >= 200 && response.statusCode < 400,
    headers: {
      get: (name: string) => {
        if (!response.header) return null
        const lowerName = name.toLowerCase()
        for (const key of Object.keys(response.header)) {
          if (key.toLowerCase() === lowerName) return response.header[key]
        }
        return null
      },
    },
  }
}

export async function getResponseData(response?: UniAppResponse | unknown) {
  if (!response) return undefined
  return (response as UniAppResponse).data
}
