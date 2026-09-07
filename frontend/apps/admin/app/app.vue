<script setup lang="ts">
import { ConfigProvider } from 'reka-ui'

const locale = useBrowserLocale()
</script>

<template>
  <ConfigProvider :locale="locale">
    <div>
      <NuxtLayout>
        <NuxtPage v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" />
          </Transition>
        </NuxtPage>
      </NuxtLayout>
      <Toaster />
    </div>
  </ConfigProvider>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: opacity 0.15s ease;
}
.page-enter-from,
.page-leave-to {
  opacity: 0;
}
</style>
