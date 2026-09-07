import { ref, computed, watch, nextTick, onMounted, onUnmounted, toValue, type MaybeRefOrGetter } from 'vue'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export interface ColumnDef<TRow = any> {
  key: string
  label: string
  [key: string]: unknown
  render?: (row: TRow) => unknown
}

export interface DisplayColumn<TRow = unknown> {
  col: ColumnDef<TRow>
  pinned: 'left' | 'right' | null
  stickyStyle: Record<string, string>
  cellStickyStyles: Record<string, string>[]
}

export function useTableColumns<TRow = unknown>(options: {
  columns: MaybeRefOrGetter<ColumnDef<TRow>[]>
  initialVisible?: (col: ColumnDef<TRow>) => boolean
  initialPinned?: Record<string, 'left' | 'right'>
}) {
  const resolvedColumns = computed(() => toValue(options.columns))

  const visibleKeys = ref<string[]>(
    resolvedColumns.value.filter((c) => options.initialVisible?.(c) ?? true).map((c) => c.key),
  )

  const pinnedState = ref<Record<string, 'left' | 'right' | null>>({})
  if (options.initialPinned) {
    for (const [key, side] of Object.entries(options.initialPinned)) {
      pinnedState.value[key] = side
    }
  }

  const columnWidths = ref<Record<string, number>>({})
  const columnCellWidths = ref<Record<string, number[]>>({})
  const measured = ref(false)
  const headerRowRef = ref<HTMLElement | null>(null)
  let resizeObserver: ResizeObserver | null = null

  function measureColumns() {
    if (!headerRowRef.value) return
    const cells = headerRowRef.value.querySelectorAll('th[data-col-key]')
    const newWidths: Record<string, number> = {}
    const newCellWidths: Record<string, number[]> = {}
    cells.forEach((cell) => {
      const key = cell.getAttribute('data-col-key')
      if (key) {
        const width = cell.getBoundingClientRect().width
        newWidths[key] = (newWidths[key] || 0) + width
        if (!newCellWidths[key]) newCellWidths[key] = []
        newCellWidths[key].push(width)
      }
    })
    columnWidths.value = newWidths
    columnCellWidths.value = newCellWidths
    measured.value = true
  }

  onMounted(() => {
    nextTick(() => {
      measureColumns()
      if (headerRowRef.value) {
        resizeObserver = new ResizeObserver(() => measureColumns())
        resizeObserver.observe(headerRowRef.value)
      }
    })
  })

  onUnmounted(() => {
    resizeObserver?.disconnect()
  })

  const visibleColumns = computed(() => resolvedColumns.value.filter((c) => visibleKeys.value.includes(c.key)))

  const displayColumns = computed<DisplayColumn<TRow>[]>(() => {
    const left: ColumnDef<TRow>[] = []
    const unpinned: ColumnDef<TRow>[] = []
    const right: ColumnDef<TRow>[] = []

    for (const col of visibleColumns.value) {
      const pin = pinnedState.value[col.key] || null
      if (pin === 'left') left.push(col)
      else if (pin === 'right') right.push(col)
      else unpinned.push(col)
    }

    const ordered: ColumnDef<TRow>[] = [...left, ...unpinned, ...right]
    const result: DisplayColumn<TRow>[] = []

    const shadowRight = '2px 0 4px rgba(0,0,0,0.08)'
    const shadowLeft = '-2px 0 4px rgba(0,0,0,0.08)'

    let leftOffset = 0
    for (let i = 0; i < ordered.length; i++) {
      const col = ordered[i]!
      const pin: 'left' | 'right' | null = pinnedState.value[col.key] || null
      const dispCol: DisplayColumn<TRow> = {
        col,
        pinned: pin,
        stickyStyle: {},
        cellStickyStyles: [],
      }

      const isLastLeft =
        pin === 'left' && (i === ordered.length - 1 || pinnedState.value[ordered[i + 1]!.key] !== 'left')
      const isFirstRight = pin === 'right' && (i === 0 || pinnedState.value[ordered[i - 1]!.key] !== 'right')

      // Grouped columns (multiple cells sharing a key): each cell computes its own offset
      const cellWidths = columnCellWidths.value[col.key] ?? []
      const cellCount = Math.max(cellWidths.length, 1)
      const cellStickyStyles: Record<string, string>[] = []

      if (pin === 'left') {
        if (measured.value) {
          for (let cellIndex = 0; cellIndex < cellCount; cellIndex++) {
            const before = cellWidths.slice(0, cellIndex).reduce((a, b) => a + b, 0)
            cellStickyStyles.push({
              position: 'sticky',
              left: `${leftOffset + before}px`,
              zIndex: '1',
              ...(isLastLeft && cellIndex === cellCount - 1 ? { boxShadow: shadowRight } : {}),
            })
          }
        }
        leftOffset += columnWidths.value[col.key] || 0
      } else if (pin === 'right') {
        let rightOffset = 0
        for (let j = i + 1; j < ordered.length; j++) {
          if (pinnedState.value[ordered[j]!.key] === 'right') {
            rightOffset += columnWidths.value[ordered[j]!.key] || 0
          }
        }
        if (measured.value) {
          for (let cellIndex = 0; cellIndex < cellCount; cellIndex++) {
            const after = cellWidths.slice(cellIndex + 1).reduce((a, b) => a + b, 0)
            cellStickyStyles.push({
              position: 'sticky',
              right: `${rightOffset + after}px`,
              zIndex: '1',
              ...(isFirstRight && cellIndex === 0 ? { boxShadow: shadowLeft } : {}),
            })
          }
        }
      }

      if (cellStickyStyles.length === 0) {
        cellStickyStyles.push({})
      }
      dispCol.cellStickyStyles = cellStickyStyles
      dispCol.stickyStyle = cellStickyStyles[0]!

      result.push(dispCol)
    }

    return result
  })

  watch(visibleColumns, () => {
    nextTick(measureColumns)
  })

  function toggleColumn(key: string) {
    if (visibleKeys.value.includes(key)) {
      visibleKeys.value = visibleKeys.value.filter((k) => k !== key)
    } else {
      visibleKeys.value = [...visibleKeys.value, key]
    }
  }

  function setAllVisible(show: boolean) {
    visibleKeys.value = show ? resolvedColumns.value.map((c) => c.key) : []
  }

  function pinLeft(key: string) {
    pinnedState.value = { ...pinnedState.value, [key]: 'left' }
    nextTick(measureColumns)
  }

  function pinRight(key: string) {
    pinnedState.value = { ...pinnedState.value, [key]: 'right' }
    nextTick(measureColumns)
  }

  function unpin(key: string) {
    pinnedState.value = { ...pinnedState.value, [key]: null }
    nextTick(measureColumns)
  }

  return {
    visibleKeys,
    pinnedState,
    visibleColumns,
    displayColumns,
    headerRowRef,
    toggleColumn,
    setAllVisible,
    pinLeft,
    pinRight,
    unpin,
  }
}
