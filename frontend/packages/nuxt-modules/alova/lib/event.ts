import { defaultDict } from '@hoshiyomi/util/lib/collection'
import type { AlovaCustomTypeMeta } from '../types'

export interface RequestInfo {
  method: string
  data?: unknown
  meta?: AlovaCustomTypeMeta
  headers: Record<string, string>
  credentials?: string
}

export interface ResponseInfo {
  status: number
  ok: boolean
  headers: { get: (name: string) => string | null }
}

export interface EventGenerics<Req extends RequestInfo = RequestInfo, Resp extends ResponseInfo = ResponseInfo> {
  Req: Req
  Resp: Resp
}

export class BizError extends Error {
  constructor(
    public response: ResponseInfo,
    public request: RequestInfo,
    public data: unknown,
  ) {
    const msg = data && typeof data === 'object' ? (data as Record<string, unknown>).message : undefined
    super(typeof msg === 'string' ? msg : 'Business error')
  }
}

export interface EventMap<G extends EventGenerics, Req = G['Req'], Resp = G['Resp']> {
  'request:start': { request: Req }
  'request:success': { request: Req; response: Resp; data: unknown }
  'request:error': { request: Req; error: unknown; data?: unknown }
  'request:unknown_error': { request: Req; error: unknown; data?: unknown }
  'request:bizerror': { request: Req; error: BizError; data?: unknown }
  'request:complete': { request: Req }
}

export type EventSubscriber<T> = (data: T) => void

export type EventSystem<G extends EventGenerics> = ReturnType<typeof createEventSystem<G>>

export const createEventSystem = <G extends EventGenerics>() => {
  type M = EventMap<G>
  const subscribers = defaultDict<keyof M, EventSubscriber<M[keyof M]>[]>(() => [])

  return {
    /// Subscribe to an event, return a function to unsubscribe
    subscribe<K extends keyof M>(type: K, handler: EventSubscriber<M[K]>) {
      subscribers[type].push(handler as EventSubscriber<M[keyof M]>)
      return () => this.unsubscribe(type, handler)
    },
    unsubscribe<K extends keyof M>(type: K, handler: EventSubscriber<M[K]>) {
      subscribers[type] = subscribers[type].filter((h) => h !== handler)
    },
    emit<K extends keyof M>(type: K, data: M[K]) {
      for (const handler of subscribers[type]) {
        handler(data)
      }
    },
  }
}
