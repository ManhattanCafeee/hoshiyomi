import type { RequestInfo, ResponseInfo } from '@hoshiyomi/alova/types'

declare module '@hoshiyomi/alova/types' {
  export interface AlovaCustomTypeMeta {
    noMessage?: (e: { request: RequestInfo; response?: ResponseInfo }) => boolean
  }
}
