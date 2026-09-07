import { defineStore } from 'pinia'
import Apis from '@/api/generated'

export interface UserInfo {
  id: number
  username: string
  email: string
  created_at: string
  updated_at: string
  permissions: string[]
}

function readPersisted(): UserInfo | null {
  try {
    const raw = uni.getStorageSync('user')
    if (!raw)
      return null
    return JSON.parse(raw) as UserInfo
  }
  catch {
    return null
  }
}

export const useUserStore = defineStore('user', () => {
  const info = ref<UserInfo | null>(readPersisted())

  function setUser(u: UserInfo) {
    info.value = u
    uni.setStorageSync('user', JSON.stringify(u))
  }

  async function fetchMe() {
    const data = (await Apis.auth.Auth__jwtMe({})) as unknown as {
      user: Omit<UserInfo, 'permissions'>
      permissions: string[]
    }
    setUser({ ...data.user, permissions: data.permissions ?? [] })
    return data
  }

  function clear() {
    info.value = null
    uni.removeStorageSync('user')
  }

  return { info, setUser, fetchMe, clear }
})
