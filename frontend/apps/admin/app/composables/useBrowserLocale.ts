export function useBrowserLocale() {
  return computed(() => {
    if (import.meta.server) return 'zh-CN'
    const lang = navigator.language || 'zh-CN'
    if (!lang.includes('-')) return `${lang}-${lang.toUpperCase()}`
    return lang
  })
}
