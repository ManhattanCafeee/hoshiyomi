<script setup lang="ts">
import { LogOut, Sun, Moon } from '@lucide/vue'
import { SidebarTrigger } from '@/components/ui/sidebar'

const { user } = useAuthState()
const route = useRoute()
const { darkMode } = useThemeMode()

const pageTitle = computed(() => {
  const map: Record<string, string> = {
    '/': '仪表盘',
    '/users': '用户管理',
    '/profile': '个人资料',
  }
  return map[route.path] || ''
})

defineEmits<{ logout: [] }>()
</script>

<template>
  <header class="flex h-14 items-center gap-2 border-b px-4">
    <SidebarTrigger />
    <h1 class="text-sm font-medium">{{ pageTitle }}</h1>
    <div class="flex-1" />
    <div class="flex items-center gap-2">
      <span class="text-sm text-muted-foreground">{{ user()?.username }}</span>
      <Button variant="ghost" size="icon-sm" aria-label="切换主题" @click="darkMode = !darkMode">
        <Sun v-if="darkMode" class="size-4" />
        <Moon v-else class="size-4" />
      </Button>
      <Button variant="ghost" size="icon-sm" aria-label="退出登录" @click="$emit('logout')">
        <LogOut class="size-4" />
      </Button>
    </div>
  </header>
</template>
