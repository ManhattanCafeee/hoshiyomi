export function getPermission(...permissions: PermT[]): PermT | undefined {
  for (const p of permissions) {
    if (typeof p === 'function') {
      if (p()) return p
    } else {
      if (unref(p)) return p
    }
  }
}

export function hasPermission(...permissions: PermT[]): boolean {
  return !!getPermission(...permissions)
}

export type PermT = (() => boolean) | MaybeRef<boolean>
