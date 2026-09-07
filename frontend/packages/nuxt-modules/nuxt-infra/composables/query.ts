/* eslint-disable @typescript-eslint/no-dynamic-delete */
/**
 * Examples:<br/>
 * useQueryInt('page', { default: () => 1, validate: (v) => v > 0 })<br/>
 * useQueryInt('page', { showError: true }) // throw error if query is not found<br/>
 * useQueryInt('page', { fallback: '/login }) // redirect to /login if query is not found
 * useQueryInt(['page', 'p'], { default: () => 1 }) // query 'page' has higher priority
 */

import type { LocationQueryRaw, RouteLocationNormalizedLoaded } from 'vue-router'

type Opts<T> = {
  /**
   * Get default value if query string is not found or parse error
   * @default undefined
   */
  default?: () => T
  /**
   * Customize route
   * @default useRoute()
   */
  route?: RouteLocationNormalizedLoaded
  /**
   * Throw error if query is not found
   * @default false
   */
  showError?: boolean
  /**
   * Redirect to this path if query is not found
   * @default undefined
   */
  fallback?: string
  /**
   * Validate query
   * @default undefined
   */
  validate?: (v: T) => boolean
  /**
   * Don't immediate update query string
   * @default true
   */
  lazy?: boolean
}

type Serializer<T> = { in: (v: unknown) => T | undefined; out: (v: unknown) => string }

const StringSerializer: Serializer<string> = {
  in: (v) => `${v}`,
  out: (v) => `${v}`,
}

const NumberSerializer: Serializer<number> = {
  in: (v) => {
    const value = Number.parseInt(`${v}`)
    if (Number.isNaN(value)) return
    return value
  },
  out: (v) => `${v}`,
}

export function toAutoController<T>({
  data,
  update,
  errorMessage,
  opts,
}: {
  data: Ref<T | undefined>
  update: () => void
  errorMessage: () => string
  opts?: Opts<T>
}) {
  if (data.value === undefined) {
    const error = createError({ statusCode: 404, statusMessage: errorMessage() })
    if (opts?.showError) {
      showError(error)
      throw error
    }
    if (opts?.fallback) {
      navigateTo(opts.fallback)
      throw error
    }
  }
  if (opts?.lazy === false) update()
  return computed({
    get: () => data.value,
    set: (v) => {
      const updated = v !== data.value
      data.value = v
      if (updated) update()
    },
  })
}

function _useQuery<T>(keyOrKeys: string | string[], serializer: Serializer<T>, opts?: Opts<T>) {
  const keys = Array.isArray(keyOrKeys) ? keyOrKeys : [keyOrKeys]
  const key = keys[0]!
  const route = (opts?.route || useRoute()) as RouteLocationNormalizedLoaded & {
    meta: { __useQuery: LocationQueryRaw }
  }

  const data = ref<T | undefined>()
  for (const key of keys) {
    if (key in route.query) {
      data.value = serializer.in(route.query[key])
      break
    }
  }
  if (data.value !== undefined && opts?.validate?.(data.value) === false) data.value = undefined
  data.value = data.value ?? opts?.default?.()

  return toAutoController({
    data,
    update: () => {
      const value = data.value === undefined ? undefined : serializer.out(data.value)
      for (const key of keys) delete route.query[key]
      if (value === undefined) delete route.query[key]
      else route.query[key] = value
      navigateTo({
        path: route.path,
        query: { ...route.query, ...route.meta.__useQuery },
        replace: true,
        force: true,
      })
    },
    errorMessage: () => `Cannot parse ${keyOrKeys}: ${route.query[key]}`,
    opts,
  })
}

function _usePathParam<T>(key: string, serializer: Serializer<T>, opts?: Opts<T>) {
  const route = (opts?.route || useRoute()) as RouteLocationNormalizedLoaded & {
    meta: { __usePathParam: { key: string; oldValue: unknown; newValue: unknown }[] }
  }

  const data = ref<T | undefined>(serializer.in(route.params[key as keyof typeof route.params]))
  if (data.value !== undefined && opts?.validate?.(data.value) === false) data.value = undefined
  data.value = data.value ?? opts?.default?.()

  return toAutoController({
    data,
    update: () => {
      const value = data.value === undefined ? undefined : serializer.out(data.value)
      route.meta.__usePathParam = {
        ...route.meta.__usePathParam,
        [key]: {
          oldValue: route.params[key as keyof typeof route.params]?.[0],
          newValue: value,
        },
      }
      // navigateTo({
      //   path: Object.values(route.meta.__usePathParam).reduce(
      //     (path, { oldValue, newValue }) => path.replace(`${endpoint}${oldValue}`, `${endpoint}${newValue}`),
      //     route.path,
      //   ),
      //   query: route.query,
      //   replace: true,
      // })
    },
    errorMessage: () => `Cannot parse path param ${key}: ${route.params[key as keyof typeof route.params]}`,
    opts,
  })
}

export function toUseQuery<A>(serializer: Serializer<A>) {
  function useQuery<T = A>(
    key: string | string[],
    opts: Opts<T> & ({ default: () => T } | { showError: true } | { fallback: string }),
  ): Ref<T>
  function useQuery<T = A>(key: string | string[], opts?: Opts<T>): Ref<T | undefined>
  function useQuery(key: string | string[], opts?: Opts<A>) {
    return _useQuery<A>(key, serializer, opts)
  }
  return useQuery
}

export function toUsePathParam<A>(serializer: Serializer<A>) {
  function usePathParam<T = A>(
    key: string,
    opts: Opts<T> & ({ default: () => T } | { showError: true } | { fallback: string }),
  ): Ref<T>
  function usePathParam<T = A>(key: string, opts?: Opts<T>): Ref<T | undefined>
  function usePathParam(key: string, opts?: Opts<A>) {
    return _usePathParam<A>(key, serializer, opts)
  }
  return usePathParam
}

export const useQueryStr = toUseQuery(StringSerializer)

export const useQueryInt = toUseQuery(NumberSerializer)

export const usePathParamStr = toUsePathParam(StringSerializer)

export const usePathParamInt = toUsePathParam(NumberSerializer)
