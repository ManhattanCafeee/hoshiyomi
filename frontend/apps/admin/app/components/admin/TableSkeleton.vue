<script setup lang="ts">
defineProps<{
  colspan: number
  rows?: number
}>()
</script>

<template>
  <TableRow v-for="i in rows ?? 5" :key="i">
    <TableCell :colspan="colspan" class="p-2">
      <Skeleton class="h-8 w-full" />
    </TableCell>
  </TableRow>
</template>
