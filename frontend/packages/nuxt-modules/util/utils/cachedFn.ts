import type { NoPromise } from '../types'

declare module '#app' {
  interface NuxtApp {
    _cachedFn?: Record<symbol, unknown>
  }
}

export const defineCachedFn = <A extends unknown[], R>(fn: (...args: A) => NoPromise<R>): ((...args: A) => R) => {
  const key = Symbol('cachedFn')

  return (...args: A) => {
    const nuxtApp = useNuxtApp()
    if (!nuxtApp._cachedFn) nuxtApp._cachedFn = {}
    // If the function has already run for the current request, return the cached result directly
    if (key in nuxtApp._cachedFn) {
      return nuxtApp._cachedFn[key] as R
    }
    // Otherwise run the function and store the result in the current request context
    const result = fn(...args)
    return (nuxtApp._cachedFn[key] = result)
  }
}
