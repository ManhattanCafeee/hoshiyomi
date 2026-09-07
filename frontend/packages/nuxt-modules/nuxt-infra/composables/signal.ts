type Opts = { equals?: boolean }

export type Signal<T> = [() => T, (v: T | ((prev: T) => T)) => void]

export function createSignal<T>(value: T, options: Opts = {}): Signal<T> {
  const r = shallowRef<T>(value)
  const get = () => r.value
  const set = (v: T | ((prev: T) => T)) => {
    r.value = v instanceof Function ? v(r.value) : v
    if (options?.equals === false) triggerRef(r)
  }
  return [get, set]
}
