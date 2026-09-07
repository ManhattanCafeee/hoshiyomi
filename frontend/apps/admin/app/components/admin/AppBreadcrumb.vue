<script setup lang="ts">
import { ChevronRight, Home } from '@lucide/vue'

const route = useRoute()

const crumbs = computed(() => {
  const parts = route.path.split('/').filter(Boolean)
  const map: Record<string, string> = {
    '': '仪表盘',
    users: '用户管理',
    profile: '个人资料',
  }

  const result: { label: string; to?: string }[] = [{ label: '首页', to: '/' }]
  let path = ''
  for (const part of parts) {
    path += '/' + part
    result.push({ label: map[part] || part, to: part === parts[parts.length - 1] ? undefined : path })
  }
  return result
})
</script>

<template>
  <nav v-if="crumbs.length > 1" class="flex items-center gap-1 border-b px-4 py-1.5 text-xs text-muted-foreground">
    <template v-for="(crumb, idx) in crumbs" :key="idx">
      <Home v-if="idx === 0" class="size-3" />
      <NuxtLink v-if="crumb.to" :to="crumb.to" class="transition-colors hover:text-foreground">
        {{ crumb.label }}
      </NuxtLink>
      <span v-else class="text-foreground font-medium">{{ crumb.label }}</span>
      <ChevronRight v-if="idx < crumbs.length - 1" class="size-3" />
    </template>
  </nav>
</template>
