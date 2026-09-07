<script setup lang="ts">
import { PP } from '@/router/config'
import { useAuthStore } from '@/stores/auth'
import { useUserStore } from '@/stores/user'

definePage({
  layout: 'default',
  type: 'home',
  style: { navigationStyle: 'custom', navigationBarTitleText: '工作台' },
})

const authStore = useAuthStore()
const userStore = useUserStore()

const todayStr = computed(() => {
  const d = new Date()
  const days = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const dt = String(d.getDate()).padStart(2, '0')
  return `${y}.${m}.${dt} · ${days[d.getDay()]}`
})

const greetingText = computed(() => {
  const h = new Date().getHours()
  if (h < 12)
    return '早上好'
  if (h < 18)
    return '下午好'
  return '晚上好'
})

const displayName = computed(() => userStore.info?.username || '')
const displayEmail = computed(() => userStore.info?.email || '')
const permissions = computed(() => userStore.info?.permissions ?? [])

function goProfile() {
  uni.switchTab({ url: PP.PROFILE })
}

onShow(() => {
  // tab 页常驻,数据在 onShow 拉取(而非 onMounted);初始加载不经导航拦截器,需自行守卫
  if (!authStore.isLoggedIn) {
    uni.reLaunch({ url: PP.LOGIN })
    return
  }
  userStore.fetchMe().catch(() => {})
})
</script>

<template>
  <view class="workbench-page">
    <view class="safe-top" />
    <view class="workbench-header">
      <text class="workbench-greeting">
        {{ greetingText }},{{ displayName }}
      </text>
      <text class="workbench-date">
        {{ todayStr }}
      </text>
    </view>

    <view class="workbench-card">
      <text class="card-label">
        账户
      </text>
      <text class="card-value">
        {{ displayName }}
      </text>
      <text class="card-sub">
        {{ displayEmail }}
      </text>
      <view v-if="permissions.length" class="perm-list">
        <text v-for="p in permissions" :key="p" class="perm-chip">
          {{ p }}
        </text>
      </view>
    </view>

    <view class="workbench-actions">
      <view class="action-item" @click="goProfile">
        <text class="action-title">
          我的
        </text>
        <text class="action-desc">
          账户信息与退出登录
        </text>
      </view>
    </view>
  </view>
</template>

<style lang="scss" scoped>
.workbench-page {
  min-height: 100vh;
  background: $app-bg;
  padding: 32rpx;
  box-sizing: border-box;
}
.safe-top {
  @include app-safe-top;
}
.workbench-header {
  display: flex;
  flex-direction: column;
  gap: 8rpx;
  margin-bottom: 32rpx;
}
.workbench-greeting {
  font-size: $app-font-2xl;
  font-weight: 700;
  color: $app-text;
}
.workbench-date {
  font-size: $app-font-sm;
  color: $app-text-sub;
}
.workbench-card {
  background: $app-card;
  border: 1rpx solid $app-border;
  border-radius: $app-r-lg;
  padding: 24rpx;
  margin-bottom: 32rpx;
  display: flex;
  flex-direction: column;
  gap: 8rpx;
}
.card-label {
  font-size: $app-font-sm;
  color: $app-text-sub;
}
.card-value {
  font-size: $app-font-lg;
  font-weight: 600;
  color: $app-text;
}
.card-sub {
  font-size: $app-font-base;
  color: $app-text-sub;
}
.perm-list {
  margin-top: 8rpx;
  display: flex;
  flex-wrap: wrap;
  gap: 12rpx;
}
.perm-chip {
  font-size: $app-font-sm;
  color: $app-primary;
  background: $app-bg;
  border: 1rpx solid $app-border;
  border-radius: $app-r-full;
  padding: 6rpx 20rpx;
}
.workbench-actions {
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}
.action-item {
  background: $app-card;
  border: 1rpx solid $app-border;
  border-radius: $app-r-lg;
  padding: 24rpx;
  display: flex;
  flex-direction: column;
  gap: 4rpx;
}
.action-title {
  font-size: $app-font-base;
  font-weight: 600;
  color: $app-text;
}
.action-desc {
  font-size: $app-font-sm;
  color: $app-text-sub;
}
</style>
