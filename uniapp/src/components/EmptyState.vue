<script setup lang="ts">
defineProps<{
  icon?: string
  text?: string
  tip?: string
  actionText?: string
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<template>
  <view class="empty-state">
    <view class="icon-wrap">
      <text
        class="icon"
        :class="icon || 'i-carbon-document'"
      />
    </view>
    <text class="text">
      {{ text || '暂无数据' }}
    </text>
    <text
      v-if="tip"
      class="tip"
    >
      {{ tip }}
    </text>
    <view v-if="actionText" class="action-btn" @click="emit('action')">
      <text>{{ actionText }}</text>
    </view>
    <slot />
  </view>
</template>

<style lang="scss" scoped>
.empty-state {
  @include app-flex-center;
  flex-direction: column;
  padding: 120rpx 40rpx;
}
.icon-wrap {
  margin-bottom: 24rpx;
}
.icon {
  font-size: 96rpx;
  color: $app-text-hint;
}
.text {
  font-size: $app-font-base;
  color: $app-text-sub;
  margin-bottom: 12rpx;
}
.tip {
  font-size: $app-font-sm;
  color: $app-text-hint;
}
.action-btn {
  margin-top: 32rpx;
  padding: 12rpx 48rpx;
  background: $app-card;
  border: 1rpx solid $app-border;
  border-radius: $app-r-full;
}
.action-btn text {
  font-size: $app-font-base;
  color: $app-primary;
}
</style>
