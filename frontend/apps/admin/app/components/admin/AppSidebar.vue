<script setup lang="ts">
import { LogOut } from '@lucide/vue'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { MENU_REGISTRY, DEFAULT_GROUP_ORDER, type MenuItemDef } from '~/config/menu-registry'

const route = useRoute()
const { hasPermission } = usePermissions()

const menuGroups = computed(() => {
  const groups = new Map<string, MenuItemDef[]>()
  for (const item of MENU_REGISTRY) {
    if (!groups.has(item.group)) groups.set(item.group, [])
    groups.get(item.group)!.push(item)
  }
  return DEFAULT_GROUP_ORDER.filter((g) => groups.has(g)).map((label) => ({
    label,
    items: groups.get(label)!,
  }))
})

function visibleItems(items: MenuItemDef[]) {
  return items.filter((i) => !i.perm || hasPermission(i.perm))
}

defineEmits<{ logout: [] }>()
</script>

<template>
  <Sidebar collapsible="icon">
    <SidebarHeader>
      <div class="flex items-center gap-2 px-2 py-1">
        <div
          class="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground text-sm font-bold"
        >
          H
        </div>
        <span class="truncate text-sm font-semibold">hoshiyomi 管理后台</span>
      </div>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup v-for="group in menuGroups" :key="group.label">
        <SidebarGroupLabel>{{ group.label }}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in visibleItems(group.items)" :key="item.to">
              <SidebarMenuButton as-child :tooltip="item.label" :data-active="route.path === item.to || undefined">
                <NuxtLink :to="item.to">
                  <component :is="item.icon" />
                  <span>{{ item.label }}</span>
                </NuxtLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton tooltip="退出登录" @click="$emit('logout')">
            <LogOut />
            <span>退出登录</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
  </Sidebar>
</template>
