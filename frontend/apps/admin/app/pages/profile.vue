<script setup lang="ts">
import { formatDateTime } from '@hoshiyomi/apisdk/utils/format'

definePageMeta({ auth: 'authenticated' as const })
useHead({ title: '个人资料' })

const notify = useNotify()
const authState = useAuthState()

const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const loading = ref(false)

async function handleChangePassword() {
  if (!oldPassword.value || !newPassword.value) {
    notify.apiError(new Error('请填写旧密码与新密码'))
    return
  }
  if (newPassword.value !== confirmPassword.value) {
    notify.apiError(new Error('两次输入的新密码不一致'))
    return
  }
  const user = authState.user()
  if (!user) return
  loading.value = true
  try {
    await Apis.user
      .User__changePassword({
        pathParams: { id: user.id },
        data: { old_password: oldPassword.value, new_password: newPassword.value },
      })
      .send()
    notify.success('密码修改成功')
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  } catch (err) {
    notify.apiError(err)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="grid gap-6 lg:grid-cols-2">
    <Card>
      <CardHeader>
        <CardTitle>账户信息</CardTitle>
        <CardDescription>当前登录账户的基本资料</CardDescription>
      </CardHeader>
      <CardContent class="space-y-2 text-sm">
        <div class="flex justify-between border-b pb-2">
          <span class="text-muted-foreground">ID</span>
          <span class="font-mono">{{ authState.user()?.id }}</span>
        </div>
        <div class="flex justify-between border-b pb-2">
          <span class="text-muted-foreground">用户名</span>
          <span class="font-medium">{{ authState.user()?.username }}</span>
        </div>
        <div class="flex justify-between border-b pb-2">
          <span class="text-muted-foreground">邮箱</span>
          <span>{{ authState.user()?.email }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">创建时间</span>
          <span>{{ formatDateTime(authState.user()?.created_at) }}</span>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>修改密码</CardTitle>
        <CardDescription>只能修改本人账户的密码</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="space-y-4" @submit.prevent="handleChangePassword">
          <div class="space-y-2">
            <Label for="old-password">旧密码</Label>
            <Input id="old-password" v-model="oldPassword" type="password" placeholder="请输入旧密码" />
          </div>
          <div class="space-y-2">
            <Label for="new-password">新密码</Label>
            <Input id="new-password" v-model="newPassword" type="password" placeholder="8-32 位,请输入新密码" />
          </div>
          <div class="space-y-2">
            <Label for="confirm-password">确认新密码</Label>
            <Input id="confirm-password" v-model="confirmPassword" type="password" placeholder="再次输入新密码" />
          </div>
          <Button type="submit" :disabled="loading">
            {{ loading ? '提交中…' : '修改密码' }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
