export const defineCachedFn = <A extends unknown[], R>(fn: (...args: A) => NoPromise<R>): ((...args: A) => R) => {
  let hasResult = false
  let cachedResult: R
  return (...args: A) => {
    if (hasResult) return cachedResult
    const result = fn(...args)
    hasResult = true
    return (cachedResult = result)
  }
}
