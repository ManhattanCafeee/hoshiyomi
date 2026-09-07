<script setup lang="ts">
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from '@lucide/vue'

const props = defineProps<{
  total: number
  page: number
  size: number
}>()

const emit = defineEmits<{
  change: [page: number]
}>()

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.size)))

function goTo(page: number) {
  if (page >= 1 && page <= totalPages.value && page !== props.page) {
    emit('change', page)
  }
}
</script>

<template>
  <div v-if="total > size" class="flex items-center justify-between pt-4">
    <span class="text-sm text-muted-foreground">共 {{ total }} 条,第 {{ page }}/{{ totalPages }} 页</span>
    <div class="flex items-center gap-1">
      <Button variant="ghost" size="icon-sm" :disabled="page <= 1" aria-label="第一页" @click="goTo(1)">
        <ChevronsLeft class="size-4" />
      </Button>
      <Button variant="ghost" size="icon-sm" :disabled="page <= 1" aria-label="上一页" @click="goTo(page - 1)">
        <ChevronLeft class="size-4" />
      </Button>
      <span class="flex h-8 min-w-[2.5rem] items-center justify-center rounded-md border text-sm">
        {{ page }}
      </span>
      <Button variant="ghost" size="icon-sm" :disabled="page >= totalPages" aria-label="下一页" @click="goTo(page + 1)">
        <ChevronRight class="size-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        :disabled="page >= totalPages"
        aria-label="最后一页"
        @click="goTo(totalPages)"
      >
        <ChevronsRight class="size-4" />
      </Button>
    </div>
  </div>
</template>
