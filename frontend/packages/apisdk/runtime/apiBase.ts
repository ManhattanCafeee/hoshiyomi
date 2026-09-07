import { alovaInstance } from '../lib/api'

/**
 * Unified API baseURL: runtimeConfig.public.apiBase (overridable at build time via NUXT_PUBLIC_API_BASE).
 * - Empty: keep relative paths and use the same-origin reverse proxy (dev Vite proxy / production Nginx location /root/api).
 * - Non-empty: all alova requests and public-key requests go directly to that origin (cross-origin deployment).
 *
 * alova creates a Method and snapshots the baseURL on each call; this plugin runs before the first request
 * (onNuxtReady auth init / user actions), so the setting always takes effect.
 */
export default defineNuxtPlugin(() => {
  const {
    public: { apiBase },
  } = useRuntimeConfig()
  if (apiBase) {
    alovaInstance.options.baseURL = apiBase
  }
})
