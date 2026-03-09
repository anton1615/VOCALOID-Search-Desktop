export function formatStorageSize(sizeKb: number | null): string {
  if (sizeKb === null) return 'Unavailable'
  if (sizeKb >= 1024 * 1024) {
    return `${(sizeKb / (1024 * 1024)).toFixed(1)} GB`
  }
  if (sizeKb >= 1024) {
    return `${(sizeKb / 1024).toFixed(1)} MB`
  }
  return `${sizeKb} KB`
}

export function formatVideoCount(count: number | null): string {
  if (count === null) return 'Unavailable'
  return count.toLocaleString('en-US')
}
