<script setup lang="ts">
declare module 'vue-router' {
  interface RouteMeta {
    backGuardInstIds: number[]
  }
}

const route = useRoute()
const router = useRouter()

const opened = defineModel<boolean>('opened', { default: false })

const count = useState('back-guard-count', () => 0)
const popstateCalls = useState('back-guard-popstate-calls', () => 0)
const lastId = useState('back-guard-last-id', () => 0)
let instId = 0
let unregisterNavigationGuard: () => void

const open = () => {
  instId = lastId.value += 1
  route.meta.backGuardInstIds = [...(route.meta.backGuardInstIds || []), instId]

  window.addEventListener('popstate', onPopstate)

  unregisterNavigationGuard = router.beforeEach((to, from, next) => next(false))
}

const close = () => {
  route.meta.backGuardInstIds = route.meta.backGuardInstIds.filter((x) => x !== instId)

  window.removeEventListener('popstate', onPopstate)

  unregisterNavigationGuard()
}

const onPopstate = () => {
  // Ensure only one close per round of events; precompute the guard count
  if (!count.value) count.value = route.meta.backGuardInstIds.length

  if ((popstateCalls.value += 1) === count.value && route.meta.backGuardInstIds.at(-1) === instId) opened.value = false

  // popstate fires twice; the second firing may be due to `next(false)`; reset only on the last event
  if (popstateCalls.value >= count.value * 2 - 1) {
    count.value = 0
    popstateCalls.value = 0
  }
}

defineExpose({ open, close })

watch(
  opened,
  (v) => {
    if (v) open()
    else close()
  },
  { immediate: true },
)
</script>

<template>
  <slot />
</template>
