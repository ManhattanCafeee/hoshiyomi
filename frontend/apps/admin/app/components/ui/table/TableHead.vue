<script setup lang="ts">
import type { HTMLAttributes, StyleValue } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
  dataColKey?: string
  stickyClass?: string
  stickyStyle?: StyleValue
}>()
</script>

<template>
  <th
    data-slot="table-head"
    :data-col-key="props.dataColKey"
    :class="
      cn(
        'text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap [&:has([role=checkbox])]:pr-0',
        props.class,
        props.stickyClass,
      )
    "
    :style="props.stickyStyle"
  >
    <slot />
  </th>
</template>
