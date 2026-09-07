function matchPermission(granted: string, required: string): boolean {
  if (granted === '*' || granted === required) return true
  if (granted.endsWith('.*')) {
    return required.startsWith(granted.slice(0, -2) + '.')
  }
  return false
}

export function usePermissions() {
  const state = useAuthState()
  const permissions = computed(() => state.permissions())

  function hasPermission(slug: string): boolean {
    return permissions.value.some((p) => matchPermission(p, slug))
  }

  function hasAnyPermission(slugs: string[]): boolean {
    return slugs.some((s) => hasPermission(s))
  }

  function hasAllPermissions(slugs: string[]): boolean {
    return slugs.every((s) => hasPermission(s))
  }

  return { permissions, hasPermission, hasAnyPermission, hasAllPermissions }
}
