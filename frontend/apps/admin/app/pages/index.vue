<script setup lang="ts">
import { Users, KeyRound } from '@lucide/vue'
import { PERM_LABELS } from '~/config/perm-labels'

definePageMeta({ auth: 'authenticated' as const })
useHead({ title: '仪表盘' })

const authState = useAuthState()

const userName = computed(() => authState.user()?.username ?? '')
const email = computed(() => authState.user()?.email ?? '')

const quickLinks = [
  { to: '/users', label: '用户管理', desc: '用户列表、创建与编辑', icon: Users },
  { to: '/profile', label: '个人资料', desc: '账户信息与密码修改', icon: KeyRound },
]
</script>

<template>
  <div class="space-y-6">
    <Card>
      <CardContent class="py-6">
        <h1 class="text-lg font-semibold">欢迎回来,{{ userName }}</h1>
        <p class="mt-1 text-sm text-muted-foreground">{{ email }}</p>
        <div class="mt-4 flex flex-wrap gap-2">
          <Badge v-for="p in authState.permissions()" :key="p" variant="outline" class="text-xs">
            {{ PERM_LABELS[p] || p }}
          </Badge>
        </div>
      </CardContent>
    </Card>

    <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <NuxtLink
        v-for="s in quickLinks"
        :key="s.to"
        :to="s.to"
        class="flex items-center gap-3 rounded-xl border p-4 text-sm transition-colors hover:bg-muted/50"
      >
        <div class="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <component :is="s.icon" class="size-5" />
        </div>
        <div>
          <p class="font-medium">{{ s.label }}</p>
          <p class="text-xs text-muted-foreground">{{ s.desc }}</p>
        </div>
      </NuxtLink>
    </div>
  </div>
</template>
