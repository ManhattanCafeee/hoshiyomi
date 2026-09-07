<script lang="ts" setup>
import type { DateValue } from 'reka-ui'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Button } from '@/components/ui/button'
import { Calendar as CalendarIcon } from '@lucide/vue'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{
    modelValue?: DateValue
    placeholder?: string
    class?: string
    monthOnly?: boolean
    minValue?: DateValue
  }>(),
  {
    modelValue: undefined,
    placeholder: 'Select a date',
    class: undefined,
    monthOnly: false,
    minValue: undefined,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: DateValue | undefined]
}>()

function formatDisplay(val: DateValue): string {
  if (props.monthOnly) {
    return `${val.year}-${String(val.month).padStart(2, '0')}`
  }
  return val.toString()
}

function handleUpdate(v: DateValue | undefined) {
  if (props.monthOnly && v) {
    v = v.set({ day: 1 })
  }
  emit('update:modelValue', v)
}
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        :class="
          cn('w-full justify-start text-left font-normal', !props.modelValue && 'text-muted-foreground', props.class)
        "
      >
        <CalendarIcon class="mr-1 size-4 shrink-0" />
        <span class="truncate">{{ props.modelValue ? formatDisplay(props.modelValue) : props.placeholder }}</span>
      </Button>
    </PopoverTrigger>
    <PopoverContent class="w-auto p-0">
      <Calendar
        :model-value="props.modelValue"
        :min-value="props.minValue"
        :layout="props.monthOnly ? 'month-and-year' : undefined"
        :no-grid="props.monthOnly"
        @update:model-value="handleUpdate"
      />
    </PopoverContent>
  </Popover>
</template>
