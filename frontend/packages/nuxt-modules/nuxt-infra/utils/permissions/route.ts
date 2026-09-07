import type { RouteLocationAsString } from 'vue-router'
import type { ModuleOptions } from '../../types/module'
import { hasPermission, type PermT } from './base'

type RouteTo =
  | RouteLocationAsString
  | {
      path: RouteLocationAsString
      redirect?: boolean
    }

type RouteCase = {
  /**
   * authentication permissions
   * @default undefined
   */
  auth: PermT[]
  /**
   * route to url when authentication succeeds
   * @default undefined
   */
  to?: RouteTo
  /**
   * whether to continue verifying other permissions when authentication succeeds
   * @default false
   */
  continueVerify?: boolean
}

type RouteAuthOptions = {
  /**
   * fallback url when authentication fails
   * @default undefined
   */
  fallback?: RouteLocationAsString
  /**
   * throw error if all authentication fails
   * @default false
   */
  showError?: boolean
}

function redirectTo(to: RouteLocationAsString) {
  if (!to.startsWith('http')) to = `${location.origin}${to}`
  location.href = to
}

function routeTo(to: RouteTo) {
  const config = useRuntimeConfig().public.hmeqoNuxtInfra as ModuleOptions | undefined
  if (typeof to === 'object') {
    const { path, redirect } = to
    if (redirect) return redirectTo(path)
    if (config?.routeAuth?.defaultRedirect) return redirectTo(path)
    return navigateTo(path)
  }
  if (config?.routeAuth?.defaultRedirect) return redirectTo(to)
  return navigateTo(to)
}

export async function routeAuth(cases: RouteCase[], routeAuthOptions?: RouteAuthOptions) {
  const { fallback, showError } = routeAuthOptions || {}
  for (const { auth: permissions, to, continueVerify } of cases) {
    if (hasPermission(...permissions)) {
      if (to) return routeTo(to)
      if (continueVerify !== true) return
    }
  }
  if (fallback) return routeTo(fallback)
  if (showError) throw createError({ statusCode: 403 })
}
