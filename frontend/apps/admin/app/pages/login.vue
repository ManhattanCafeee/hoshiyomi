<script setup lang="ts">
import type { AuthState } from '@hoshiyomi/apisdk/stores/authState'

definePageMeta({ layout: false, auth: 'guest' as const })
useHead({ title: '登录' })

const router = useRouter()
const notify = useNotify()

const username = ref('')
const password = ref('')
const loading = ref(false)

async function handleLogin() {
  if (!username.value || !password.value) {
    notify.apiError(new Error('请输入用户名和密码'))
    return
  }
  loading.value = true
  try {
    // 会话认证:POST /auth/login 会下发 session Cookie(同源请求自动携带)
    const resp = (await Apis.auth
      .Auth__login({
        data: { username: username.value, password: password.value },
      })
      .send()) as unknown as AuthState
    useAuthState().setAuthState(resp)
    router.push('/')
  } catch (err) {
    notify.apiError(err, '登录失败')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-muted/30">
    <Card class="w-full max-w-sm">
      <CardHeader class="items-center text-center">
        <div
          class="mb-2 flex size-12 items-center justify-center rounded-xl bg-primary text-primary-foreground text-xl font-bold"
        >
          H
        </div>
        <CardTitle>hoshiyomi 管理后台</CardTitle>
        <CardDescription>请登录后继续</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="space-y-4" @submit.prevent="handleLogin">
          <div class="space-y-2">
            <Label for="username">用户名</Label>
            <Input id="username" v-model="username" placeholder="请输入用户名" />
          </div>
          <div class="space-y-2">
            <Label for="password">密码</Label>
            <Input id="password" v-model="password" type="password" placeholder="请输入密码" />
          </div>
          <Button type="submit" class="w-full" :disabled="loading">
            {{ loading ? '登录中…' : '登录' }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
