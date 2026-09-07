<script setup lang="ts">
import { Plus } from '@lucide/vue'
import { formatDateTime } from '@hoshiyomi/apisdk/utils/format'
import type { UserDialogRecord, UserPayload } from '~/components/admin/UserDialog.vue'

type UserRow = UserDialogRecord

definePageMeta({ auth: 'authenticated' as const, permissions: ['user:read'] })
useHead({ title: '用户管理' })

const notify = useNotify()
const { hasPermission } = usePermissions()

const page = ref(1)
const perPage = 20

const {
  data: resp,
  loading,
  send: refresh,
} = useRequest(() => Apis.user.User__list({ params: { page: page.value, per_page: perPage } }))

const items = computed(() => extractData(resp.value) as UserRow[])
const total = computed(() => extractTotal(resp.value))

const dialogOpen = ref(false)
const editingUser = ref<UserRow | null>(null)
const deletingUser = ref<UserRow | null>(null)
const deleteOpen = ref(false)
const deleting = ref(false)

function onPageChange(p: number) {
  page.value = p
  refresh()
}

async function handleSave(payload: UserPayload) {
  try {
    if (editingUser.value) {
      await Apis.user
        .User__updateUsername({
          pathParams: { id: editingUser.value.id },
          data: { username: payload.username },
        })
        .send()
      notify.success('用户名已更新')
    } else {
      await Apis.user.User__create({ data: payload }).send()
      notify.success('用户已创建')
    }
    dialogOpen.value = false
    refresh()
  } catch (err) {
    notify.apiError(err)
  }
}

async function handleDelete() {
  if (!deletingUser.value) return
  deleting.value = true
  try {
    await Apis.user.User__delete({ pathParams: { id: deletingUser.value.id } }).send()
    notify.success('用户已删除')
    deleteOpen.value = false
    deletingUser.value = null
    refresh()
  } catch (err) {
    notify.apiError(err)
  } finally {
    deleting.value = false
  }
}

function openCreate() {
  editingUser.value = null
  dialogOpen.value = true
}

function openEdit(user: UserRow) {
  editingUser.value = user
  dialogOpen.value = true
}

function openDelete(user: UserRow) {
  deletingUser.value = user
  deleteOpen.value = true
}
</script>

<template>
  <div>
    <div class="mb-4 flex items-center justify-between">
      <p class="text-sm text-muted-foreground">共 {{ total }} 位用户</p>
      <Button v-if="hasPermission('user:write')" @click="openCreate"> <Plus class="mr-1 size-4" /> 新建用户 </Button>
    </div>

    <Card>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>ID</TableHead>
            <TableHead>用户名</TableHead>
            <TableHead>邮箱</TableHead>
            <TableHead>创建时间</TableHead>
            <TableHead class="text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="user in items" :key="user.id">
            <TableCell class="font-mono text-xs">{{ user.id }}</TableCell>
            <TableCell class="font-medium">{{ user.username }}</TableCell>
            <TableCell class="text-xs">{{ user.email }}</TableCell>
            <TableCell class="text-xs text-muted-foreground">{{ formatDateTime(user.created_at) }}</TableCell>
            <TableCell class="text-right">
              <Button v-if="hasPermission('user:write')" variant="ghost" size="sm" @click="openEdit(user)">
                改名
              </Button>
              <Button
                v-if="hasPermission('user:delete')"
                variant="ghost"
                size="sm"
                class="text-destructive"
                @click="openDelete(user)"
              >
                删除
              </Button>
            </TableCell>
          </TableRow>
          <TableSkeleton v-if="loading" :colspan="5" />
          <TableRow v-if="!loading && items.length === 0">
            <TableCell :colspan="5" class="text-center text-muted-foreground">暂无数据</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </Card>

    <AppPagination :total="total" :page="page" :size="perPage" @change="onPageChange" />
    <UserDialog v-model:open="dialogOpen" :record="editingUser" @save="handleSave" />
    <ConfirmDelete v-model:open="deleteOpen" :loading="deleting" @confirm="handleDelete" />
  </div>
</template>
