<script setup lang="ts">
import { PP } from '@/router/config'

const props = withDefaults(defineProps<{
  title?: string
  back?: boolean | 'auto'
  variant?: 'white' | 'gradient'
  fixed?: boolean
}>(), {
  back: 'auto',
  variant: 'white',
  fixed: true,
})

const emit = defineEmits<{
  back: []
}>()

const showBack = computed(() => {
  if (props.back === 'auto')
    return getCurrentPages().length > 1
  return props.back
})

function handleBack() {
  if (getCurrentPages().length > 1) {
    uni.navigateBack()
  }
  else {
    uni.switchTab({ url: PP.HOME })
  }
  emit('back')
}
</script>

<template>
  <view
    class="app-navbar"
    :class="[
      `variant-${variant}`,
      { 'is-fixed': fixed },
    ]"
  >
    <view class="safe-top" />
    <view class="nav-content">
      <slot name="left">
        <view
          v-if="showBack"
          class="back-btn"
          @click="handleBack"
        >
          <text class="back-icon i-carbon-chevron-left" />
        </view>
        <view v-else class="back-spacer" />
      </slot>
      <view class="title-area">
        <slot>
          <text class="title-text">
            {{ title || '' }}
          </text>
        </slot>
      </view>
      <view class="right-slot">
        <slot name="right" />
      </view>
    </view>
  </view>
</template>

<style lang="scss" scoped>
.app-navbar {
  left: 0;
  right: 0;
  top: 0;
  z-index: 100;
}
.app-navbar.is-fixed {
  position: fixed;
}
.app-navbar.variant-white {
  background: $app-card;
  border-bottom: 1rpx solid $app-border;
}
.app-navbar.variant-gradient {
  background: linear-gradient(180deg, $app-primary 0%, $app-card 100%);
}
.safe-top {
  @include app-safe-top;
}
.nav-content {
  display: flex;
  align-items: center;
  height: 88rpx;
  padding: 0 16rpx;
}
.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 72rpx;
  height: 72rpx;
  flex-shrink: 0;
}
.back-spacer {
  width: 72rpx;
  flex-shrink: 0;
}
.back-icon {
  font-size: $app-font-xl;
  color: $app-text;
}
.title-area {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
}
.title-text {
  font-size: $app-font-lg;
  font-weight: 600;
  color: $app-text;
}
.right-slot {
  width: 72rpx;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
}
</style>
