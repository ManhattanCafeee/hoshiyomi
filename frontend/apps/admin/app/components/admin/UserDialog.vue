<script setup lang="ts">
export interface UserDialogRecord {
  id: number
  username: string
  email: string
  created_at: string
  updated_at: string
}

export interface UserPayload {
  username: string
  email?: string
  password?: string
}

const props = defineProps<{
  record: UserDialogRecord | null
}>()

const open = defineModel<boolean>('open')

const emit = defineEmits<{ save: [payload: UserPayload] }>()

const form = reactive({
  username: '',
  email: '',
  password: '',
})

const isEdit = computed(() => !!props.record)

watch(open, (v) => {
  if (v) {
    form.username = props.record?.username ?? ''
    form.email = props.record?.email ?? ''
    form.password = ''
  }
})

async function handleSave() {
  const payload: UserPayload = { username: form.username }
  if (!isEdit.value) {
    payload.email = form.email
    payload.password = form.password
  }
  emit('save', payload)
}
</script>

<template>
  <Dialog :open="open" @update:open="open = $event">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ isEdit ? '修改用户名' : '新建用户' }}</DialogTitle>
        <DialogDescription>
          {{ isEdit ? '仅修改用户名(用户名长度 3-20)' : '创建新用户(密码长度 8-32)' }}
        </DialogDescription>
      </DialogHeader>
      <form class="space-y-4" @submit.prevent="handleSave">
        <div class="space-y-2">
          <Label for="user-username">用户名</Label>
          <Input id="user-username" v-model="form.username" placeholder="3-20 位字符" />
        </div>
        <div v-if="!isEdit" class="space-y-2">
          <Label for="user-email">邮箱</Label>
          <Input id="user-email" v-model="form.email" type="email" placeholder="example@example.com" />
        </div>
        <div v-if="!isEdit" class="space-y-2">
          <Label for="user-password">密码</Label>
          <Input id="user-password" v-model="form.password" type="password" placeholder="8-32 位字符" />
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" @click="open = false">取消</Button>
          <Button type="submit">保存</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
