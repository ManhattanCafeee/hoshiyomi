<script setup lang="ts">
import Apis from '@/api/generated'
import { PP } from '@/router/config'
import { useAuthStore } from '@/stores/auth'
import { useUserStore } from '@/stores/user'

definePage({
  layout: 'default',
  style: { navigationStyle: 'custom', navigationBarTitleText: '我的' },
})

const authStore = useAuthStore()
const userStore = useUserStore()
const loggingOut = ref(false)

async function handleLogout() {
  if (loggingOut.value)
    return
  loggingOut.value = true
  try {
    // 尽力撤销服务端刷新令牌;失败不阻塞本地退出
    await Apis.auth.Auth__jwtLogout({})
  }
  catch {
    /* 忽略 */
  }
  authStore.logout()
  userStore.clear()
  uni.reLaunch({ url: PP.LOGIN })
  loggingOut.value = false
}

onShow(() => {
  // 初始加载不经导航拦截器,需自行守卫
  if (!authStore.isLoggedIn) {
    uni.reLaunch({ url: PP.LOGIN })
    return
  }
  userStore.fetchMe().catch(() => {})
})
</script>

<template>
  <view class="profile-page">
    <view class="safe-top" />
    <view class="profile-header">
      <text class="profile-title">
        我的
      </text>
    </view>

    <view class="profile-card">
      <view class="info-row">
        <text class="info-label">
          ID
        </text>
        <text class="info-value">
          {{ userStore.info?.id }}
        </text>
      </view>
      <view class="info-row">
        <text class="info-label">
          用户名
        </text>
        <text class="info-value">
          {{ userStore.info?.username }}
        </text>
      </view>
      <view class="info-row">
        <text class="info-label">
          邮箱
        </text>
        <text class="info-value">
          {{ userStore.info?.email }}
        </text>
      </view>
      <view class="info-row">
        <text class="info-label">
          注册时间
        </text>
        <text class="info-value">
          {{ userStore.info?.created_at }}
        </text>
      </view>
    </view>

    <button class="logout-btn" :disabled="loggingOut" @click="handleLogout">
      <text>{{ loggingOut ? '退出中…' : '退出登录' }}</text>
    </button>
  </view>
</template>

<style lang="scss" scoped>
.profile-page {
  min-height: 100vh;
  background: $app-bg;
  padding: 32rpx;
  box-sizing: border-box;
}
.safe-top {
  @include app-safe-top;
}
.profile-header {
  margin-bottom: 32rpx;
}
.profile-title {
  font-size: $app-font-2xl;
  font-weight: 700;
  color: $app-text;
}
.profile-card {
  background: $app-card;
  border: 1rpx solid $app-border;
  border-radius: $app-r-lg;
  padding: 24rpx;
  margin-bottom: 48rpx;
}
.info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16rpx 0;
}
.info-row + .info-row {
  border-top: 1rpx solid $app-divider;
}
.info-label {
  font-size: $app-font-sm;
  color: $app-text-sub;
}
.info-value {
  font-size: $app-font-base;
  color: $app-text;
}
.logout-btn {
  width: 100%;
  height: 96rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $app-card;
  color: $app-danger;
  font-size: $app-font-lg;
  font-weight: 600;
  border-radius: $app-r-xl;
  border: 1rpx solid $app-border;
}
.logout-btn[disabled] {
  opacity: 0.5;
}
</style>
