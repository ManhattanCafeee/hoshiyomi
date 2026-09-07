export function navigateBack() {
  const router = useRouter()
  router.back()
}

export function refreshPage() {
  const router = useRouter()
  router.go(0)
}

export function redirectTo(to: string, opts?: { query?: Record<string, string>; clearStates: boolean }) {
  const { query, clearStates } = opts ?? {}

  if (!to.startsWith('http')) to = `${location.origin}${to}`
  if (query) to = `${to.split('?')[0]}?${new URLSearchParams(query).toString()}`
  if (clearStates) sessionStorage.clear()
  location.href = to
}
