export function withOriginUrl(url: string) {
  return `${location.origin}${url}`
}

export function toThumbnailUrl(url: string, opts?: { size?: number }) {
  const params = new URLSearchParams({
    size: `${opts?.size || 256}`,
  })
  return `${url}?${params}`
}
