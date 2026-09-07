import type { Component } from 'vue'
import { markRaw } from 'vue'
import { LayoutDashboard, Users, KeyRound } from '@lucide/vue'

export interface MenuItemDef {
  id: string
  to: string
  label: string
  desc: string
  icon: Component
  perm?: string
  group: string
}

export const MENU_REGISTRY: MenuItemDef[] = [
  {
    id: 'dashboard',
    to: '/',
    label: '仪表盘',
    desc: '概览与快捷入口',
    icon: markRaw(LayoutDashboard),
    group: '首页',
  },
  {
    id: 'users',
    to: '/users',
    label: '用户管理',
    desc: '用户列表、创建与编辑',
    icon: markRaw(Users),
    perm: 'user:read',
    group: '系统',
  },
  {
    id: 'profile',
    to: '/profile',
    label: '个人资料',
    desc: '账户信息与密码修改',
    icon: markRaw(KeyRound),
    group: '系统',
  },
]

export const DEFAULT_GROUP_ORDER = ['首页', '系统']
