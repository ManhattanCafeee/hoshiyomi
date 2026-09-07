<script setup lang="ts">
import type { UserInfo } from '@/stores/user'
import Apis from '@/api/generated'
import { PP } from '@/router/config'
import { decodeJwtExp, useAuthStore } from '@/stores/auth'
import { useUserStore } from '@/stores/user'

// 生成 SDK 的信封类型将 data 内联(ApiResponse_LoginResp.data),未导出独立 LoginResp 接口
interface LoginResp {
  access_token: string
  refresh_token: string
  state: {
    user: Omit<UserInfo, 'permissions'>
    permissions: string[]
  }
}

definePage({
  layout: 'default',
  style: { navigationBarTitleText: '登录' },
})

const loading = ref(false)
const username = ref('')
const password = ref('')
const authStore = useAuthStore()
const userStore = useUserStore()

async function handleLogin() {
  if (loading.value)
    return
  if (!username.value || !password.value) {
    uni.showToast({ title: '请输入用户名和密码', icon: 'none' })
    return
  }
  loading.value = true
  try {
    const resp = (await Apis.auth.Auth__jwtLogin({
      data: { username: username.value, password: password.value },
    })) as unknown as LoginResp
    if (!resp?.access_token) {
      throw new Error('登录失败:服务端未返回令牌')
    }
    authStore.setState({
      userId: resp.state.user.id,
      token: resp.access_token,
      refreshToken: resp.refresh_token,
      expiresAt: decodeJwtExp(resp.access_token),
    })
    userStore.setUser({ ...resp.state.user, permissions: resp.state.permissions ?? [] })
    uni.reLaunch({ url: PP.HOME })
  }
  catch (e: any) {
    uni.showToast({ title: e?.message || '登录失败', icon: 'none' })
  }
  finally {
    loading.value = false
  }
}

function goRegister() {
  uni.navigateTo({ url: PP.REGISTER })
}
</script>

<template>
  <AppNavbar title="登录" :fixed="false" />
  <view class="login-page">
    <view class="login-content">
      <view class="login-header">
        <text class="login-logo">
          H
        </text>
        <text class="login-title">
          hoshiyomi
        </text>
        <text class="login-subtitle">
          请登录后继续
        </text>
        <view class="form-card">
          <view class="form-item">
            <text class="form-label">
              用户名
            </text>
            <input v-model="username" class="form-input" placeholder="请输入用户名">
          </view>
          <view class="form-item">
            <text class="form-label">
              密码
            </text>
            <input v-model="password" class="form-input" password placeholder="请输入密码">
          </view>
        </view>
        <button
          class="login-btn"
          :disabled="loading"
          @click="handleLogin"
        >
          <text v-if="!loading">
            登录
          </text>
          <text v-else>
            登录中…
          </text>
        </button>
        <text class="register-link" @click="goRegister">
          没有账号?去注册
        </text>
      </view>
    </view>
  </view>
</template>

<style lang="scss" scoped>
.login-page {
  min-height: 100vh;
  background: $app-bg;
  display: flex;
  align-items: center;
  justify-content: center;
}
.login-content {
  width: 100%;
  padding: 0 64rpx;
}
.login-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}
.login-logo {
  width: 120rpx;
  height: 120rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $app-primary;
  color: $app-card;
  font-size: $app-font-2xl;
  font-weight: 700;
  border-radius: $app-r-xl;
  margin-bottom: 32rpx;
}
.login-title {
  font-size: $app-font-2xl;
  font-weight: 700;
  color: $app-text;
  margin-bottom: 12rpx;
}
.login-subtitle {
  font-size: $app-font-base;
  color: $app-text-sub;
  margin-bottom: 40rpx;
}
.form-card {
  width: 100%;
  background: $app-card;
  border-radius: $app-r-lg;
  padding: 24rpx;
  margin-bottom: 32rpx;
  border: 1rpx solid $app-border;
}
.form-item {
  padding: 12rpx 0;
}
.form-item:first-child {
  border-bottom: 1rpx solid $app-divider;
}
.form-label {
  font-size: $app-font-sm;
  color: $app-text-sub;
  margin-bottom: 8rpx;
  display: block;
}
.form-input {
  width: 100%;
  height: 72rpx;
  padding: 0;
  font-size: $app-font-base;
  color: $app-text;
}
.login-btn {
  width: 100%;
  height: 96rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $app-primary;
  color: $app-card;
  font-size: $app-font-lg;
  font-weight: 600;
  border-radius: $app-r-xl;
  border: none;
}
.login-btn[disabled] {
  opacity: 0.5;
}
.register-link {
  margin-top: 32rpx;
  font-size: $app-font-sm;
  color: $app-primary;
}
</style>
