import type { AuthAdapter } from '../types/auth'

const KEY = '$authAdapter'

export function defineAuthAdapter(adapter: AuthAdapter) {
  const nuxtApp = useNuxtApp()
  nuxtApp.provide('authAdapter', adapter)
}

export function useAuthAdapter(): AuthAdapter {
  const nuxtApp = useNuxtApp()
  if (!(KEY in nuxtApp)) {
    throw createError({
      statusCode: 500,
      statusMessage: 'Auth adapter not registered. Create a plugin that calls defineAuthAdapter().',
    })
  }
  return nuxtApp[KEY] as AuthAdapter
}
