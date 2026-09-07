/**
 * Multipart middleware specific to the Web platform.
 * Used only in the Admin frontend (fetch adapter); the uniapp side does not reference this file.
 *
 * Uploads on the uniapp side go through `@alova/adapter-uniapp`'s `requestType: 'upload'` → `uni.uploadFile()`,
 * so no FormData conversion is needed.
 */

import type { RequestInfo } from './event'

export function transformToFormData(data: unknown): FormData {
  if (data instanceof FormData) return data
  const formData = new FormData()
  if (data && typeof data === 'object' && data !== null) {
    Object.entries(data as Record<string, unknown>).forEach(([key, value]) => {
      formData.append(key, value instanceof Blob ? value : String(value))
    })
  }
  return formData
}

export function multipartBeforeRequest(request: RequestInfo) {
  if (request.meta?.multipart || request.data instanceof FormData) {
    if (!(request.data instanceof FormData)) {
      request.data = transformToFormData(request.data)
    }
  }
}
