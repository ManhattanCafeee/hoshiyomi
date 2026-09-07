// eslint-disable-next-line @typescript-eslint/no-explicit-any
type State = Record<string, any>

export const useGlobalCookie = defineCachedFn(() => {
  const maxAge = 60 * 60 * 24 * 365 * 10
  return useCookie<State>('c', { sameSite: 'lax', maxAge, default: () => ({}) })
})

export const useGlobalLocal = defineCachedFn(() => {
  return useLocalStorage<State>('l', () => ({}))
})

export const useGlobalSession = defineCachedFn(() => {
  return useSessionStorage<State>('s', () => ({}))
})

export const useStorageProp = <T>(store: Ref<State>, key: string, opts?: { default: () => T }) => {
  return computed<T>({
    get: () => store.value[key] ?? opts?.default?.(),
    set: (v) => {
      store.value[key] = v
    },
  })
}

export const useCookieState = <T>(key: string, s?: { default: () => T }) => {
  return useStorageProp(useGlobalCookie(), key, s)
}

export const useLocalState = <T>(key: string, s?: { default: () => T }) => {
  return useStorageProp(useGlobalLocal(), key, s)
}

export const useSessionState = <T>(key: string, s?: { default: () => T }) => {
  return useStorageProp(useGlobalSession(), key, s)
}
