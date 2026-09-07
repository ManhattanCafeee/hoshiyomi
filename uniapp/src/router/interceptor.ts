import { useAuthStore } from '@/stores/auth'
import { PP, TAB_PATHS } from './config'

export const routeInterceptor = {
  install() {
    uni.addInterceptor('navigateTo', { invoke })
    uni.addInterceptor('redirectTo', { invoke })
    uni.addInterceptor('reLaunch', { invoke })
    uni.addInterceptor('switchTab', { invoke })
  },
}

function invoke({ url }: { url: string }) {
  const path = url?.split('?')[0] ?? ''

  const authStore = useAuthStore()

  if (authStore.isLoggedIn) {
    // 已登录访问登录/注册页 → 回工作台
    if (path === PP.LOGIN || path === PP.REGISTER) {
      uni.reLaunch({ url: PP.HOME })
      return false
    }
    return true
  }

  // 未登录访问 tab 页 → 回登录
  if (TAB_PATHS.includes(path)) {
    uni.reLaunch({ url: PP.LOGIN })
    return false
  }

  return true
}
