export const defaultDict = <K extends string | symbol, V>(defaultValue: () => V) =>
  new Proxy(<Record<K, V>>{}, {
    get: (target, key: K) => {
      return target[key] || (target[key] = defaultValue())
    },
  })
