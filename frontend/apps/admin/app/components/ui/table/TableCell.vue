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
  <td
    data-slot="table-cell"
    :data-col-key="props.dataColKey"
    :class="cn('p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0', props.class, props.stickyClass)"
    :style="props.stickyStyle"
  >
    <slot />
  </td>
</template>
