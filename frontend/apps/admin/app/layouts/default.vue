<script setup lang="ts">
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar'

const router = useRouter()
const { logout } = useAuthState()

async function handleLogout() {
  try {
    await Apis.auth.Auth__logout({ data: {} }).send()
  } catch {
    /* 忽略登出接口错误,本地状态必须清除 */
  }
  logout()
  router.push('/login')
}
</script>

<template>
  <SidebarProvider>
    <AppSidebar @logout="handleLogout" />
    <SidebarInset class="max-h-screen">
      <AppTopbar @logout="handleLogout" />
      <AppBreadcrumb />
      <div class="flex-1 overflow-auto p-6">
        <NuxtErrorBoundary>
          <slot />
          <template #error="{ error, clearError }">
            <div class="flex flex-col items-center justify-center h-64 space-y-3">
              <p class="text-destructive font-medium">页面加载失败</p>
              <p class="text-sm text-muted-foreground">{{ error?.message || '未知错误' }}</p>
              <Button variant="outline" size="sm" @click="clearError()">重试</Button>
            </div>
          </template>
        </NuxtErrorBoundary>
      </div>
      <AppFooter />
    </SidebarInset>
  </SidebarProvider>
</template>
