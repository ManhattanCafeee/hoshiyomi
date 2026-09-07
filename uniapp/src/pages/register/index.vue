<script setup lang="ts">
import Apis from '@/api/generated'
import { PP } from '@/router/config'

definePage({
  layout: 'default',
  style: { navigationBarTitleText: '注册' },
})

const loading = ref(false)
const username = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')

async function handleRegister() {
  if (loading.value)
    return
  if (!username.value || !email.value || !password.value) {
    uni.showToast({ title: '请填写完整信息', icon: 'none' })
    return
  }
  if (password.value !== confirmPassword.value) {
    uni.showToast({ title: '两次输入的密码不一致', icon: 'none' })
    return
  }
  loading.value = true
  try {
    await Apis.auth.Auth__register({
      data: { username: username.value, email: email.value, password: password.value },
    })
    uni.showToast({ title: '注册成功,请登录', icon: 'success' })
    setTimeout(() => uni.reLaunch({ url: PP.LOGIN }), 600)
  }
  catch (e: any) {
    uni.showToast({ title: e?.message || '注册失败', icon: 'none' })
  }
  finally {
    loading.value = false
  }
}
</script>

<template>
  <AppNavbar title="注册" :fixed="false" />
  <view class="register-page">
    <view class="register-content">
      <view class="form-card">
        <view class="form-item">
          <text class="form-label">
            用户名
          </text>
          <input v-model="username" class="form-input" placeholder="3-20 位字符">
        </view>
        <view class="form-item">
          <text class="form-label">
            邮箱
          </text>
          <input v-model="email" class="form-input" type="text" placeholder="example@example.com">
        </view>
        <view class="form-item">
          <text class="form-label">
            密码
          </text>
          <input v-model="password" class="form-input" password placeholder="8-32 位字符">
        </view>
        <view class="form-item">
          <text class="form-label">
            确认密码
          </text>
          <input v-model="confirmPassword" class="form-input" password placeholder="再次输入密码">
        </view>
      </view>
      <button
        class="register-btn"
        :disabled="loading"
        @click="handleRegister"
      >
        <text v-if="!loading">
          注册
        </text>
        <text v-else>
          提交中…
        </text>
      </button>
    </view>
  </view>
</template>

<style lang="scss" scoped>
.register-page {
  min-height: 100vh;
  background: $app-bg;
  padding: 48rpx 64rpx;
  box-sizing: border-box;
}
.form-card {
  background: $app-card;
  border-radius: $app-r-lg;
  padding: 24rpx;
  margin-bottom: 32rpx;
  border: 1rpx solid $app-border;
}
.form-item {
  padding: 12rpx 0;
}
.form-item + .form-item {
  border-top: 1rpx solid $app-divider;
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
.register-btn {
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
.register-btn[disabled] {
  opacity: 0.5;
}
</style>
