<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from '@/components/ui/alert-dialog'

const _props = withDefaults(
  defineProps<{
    title?: string
    description?: string
    loading?: boolean
  }>(),
  {
    title: '确认删除',
    description: '确定要删除这条记录吗?此操作不可撤销。',
    loading: false,
  },
)

const open = defineModel<boolean>('open')

const emit = defineEmits<{ confirm: [] }>()
</script>

<template>
  <AlertDialog :open="open" @update:open="open = $event">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>{{ title }}</AlertDialogTitle>
        <AlertDialogDescription>{{ description }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>取消</AlertDialogCancel>
        <AlertDialogAction :disabled="loading" @click="emit('confirm')">
          {{ loading ? '删除中…' : '确认删除' }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
