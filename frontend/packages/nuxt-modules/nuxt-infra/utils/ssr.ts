export function clientOnly<T>(callback: T) {
  if (import.meta.client) return callback
  return undefined
}

export function serverOnly<T>(callback: T) {
  if (import.meta.server) return callback
  return undefined
}
