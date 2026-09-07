const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'] as const
type Units = (typeof units)[number]

const defaultDecimals = 2

export interface FormatFileSizeOptions {
  unit?: Units
  decimals?: number
}

export function formatFileSize(size: string | number, options: FormatFileSizeOptions = {}): string {
  const { unit, decimals } = options

  let value = Number.parseInt(`${size}`)
  if (isNaN(value) || value < 0) {
    return '0B'
  }

  let unitIndex = 0

  if (unit) {
    unitIndex = units.indexOf(unit)
    if (unitIndex === -1) unitIndex = 0
    value /= Math.pow(1024, unitIndex)
  } else {
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024
      unitIndex++
    }
  }

  const formattedValue =
    decimals === 0 || (!decimals && value % 1 === 0)
      ? Math.round(value).toString()
      : value.toFixed(decimals || defaultDecimals)

  return `${formattedValue}${units[unitIndex]}`
}
