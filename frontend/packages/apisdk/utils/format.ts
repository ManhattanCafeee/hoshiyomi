export function formatCurrency(amount: string | number): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount
  return `¥${(isNaN(num) ? 0 : num).toFixed(2)}`
}

export function formatDate(dateStr: string | null | undefined, fallback = '-'): string {
  if (!dateStr) return fallback
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return fallback
  }
}

export function formatShortDate(dateStr: string | null | undefined, fallback = '-'): string {
  if (!dateStr) return fallback
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return fallback
  }
}

export function formatDateTime(dateStr: string | null | undefined, fallback = '-'): string {
  return formatDate(dateStr, fallback)
}

export function formatPercentage(value: number, decimals = 2): string {
  return `${(value || 0).toFixed(decimals)}%`
}
