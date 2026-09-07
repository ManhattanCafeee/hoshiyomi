import type { AsyncDataRequestStatus } from '#app'

export const useLockedSend = <T>(
  send: () => T | Promise<T>,
  opts?: {
    /**
     * Maximum number of concurrent requests
     * @default 1
     */
    concurrency?: number
    /**
     * Only execute the request once
     * @default false
     */
    once?: boolean
    status?: Ref<AsyncDataRequestStatus>
  },
) => {
  const maxConcurrency = opts?.concurrency ?? 1
  let pendingCount = 0
  let hasExecuted = false

  return {
    lockedSend: async () => {
      if (opts?.once && hasExecuted) return
      if (pendingCount >= maxConcurrency) return

      pendingCount++

      try {
        const result = await send()
        if ((opts?.status?.value ?? 'success') === 'success') {
          hasExecuted = true
          return result
        }
      } finally {
        pendingCount--
      }
    },
  }
}
