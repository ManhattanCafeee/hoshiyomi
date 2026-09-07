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

export const toResponseInfo = (response: Response): ResponseInfo => ({
  status: response.status,
  ok: response.ok,
  headers: response.headers,
})

export const getResponseData = async (response?: Response) => {
  if (!response) return undefined
  const isJson = response.headers.get('Content-Type')?.includes('application/json')
  const data = isJson ? await response.json() : await response.text()
  return data
}
