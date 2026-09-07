interface OpenOptions {
  /** Delay before closing (milliseconds) */
  delay?: number
  /** Initial state */
  initialValue?: boolean
}

export const useOpen = (options: OpenOptions = {}) => {
  const { delay = 0, initialValue = false } = options

  const isOpen = ref(initialValue)
  let timer: ReturnType<typeof setTimeout> | null = null

  function open() {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
    isOpen.value = true
  }

  function close() {
    if (timer) clearTimeout(timer)

    if (delay > 0) {
      timer = setTimeout(() => {
        isOpen.value = false
        timer = null
      }, delay)
    } else {
      isOpen.value = false
    }
  }

  function toggle() {
    if (isOpen.value) {
      close()
    } else {
      open()
    }
  }

  return {
    isOpen,
    open,
    close,
    toggle,
  }
}
