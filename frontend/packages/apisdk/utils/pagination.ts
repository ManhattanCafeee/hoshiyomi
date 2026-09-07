export type QueryParams = Record<string, string | number | boolean | undefined>

export function asRecord(r: unknown): Record<string, unknown> | null {
  return typeof r === 'object' && r !== null ? (r as Record<string, unknown>) : null
}

export function extractTotal(resp: unknown): number {
  const d = asRecord(resp)
  if (!d) return 0
  if (typeof d.total === 'number') return d.total
  const p = d.pagination
  if (typeof p === 'object' && p !== null) {
    const pt = (p as Record<string, unknown>).total
    if (typeof pt === 'number') return pt
  }
  return 0
}

export function extractData<T = unknown>(resp: unknown): T[] {
  const d = asRecord(resp)
  if (!d) return []
  if (Array.isArray(d)) return d
  const v = d.content ?? d.list ?? d.items ?? d.data
  return Array.isArray(v) ? v : []
}
