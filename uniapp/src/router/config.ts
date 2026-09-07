export const PP = {
  LOGIN: '/pages/login/index',
  REGISTER: '/pages/register/index',
  HOME: '/pages/home/index',
  PROFILE: '/pages/profile/index',
} as const

export const LoginPath = PP.LOGIN

export const TAB_PATHS: string[] = [PP.HOME, PP.PROFILE]
