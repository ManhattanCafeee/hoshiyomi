import { createPinia } from 'pinia'
import { createSSRApp } from 'vue'
import App from './App.vue'
import { routeInterceptor } from './router/interceptor'
import '@/api/generated'
import 'uno.css'

export function createApp() {
  const app = createSSRApp(App)
  const pinia = createPinia()
  app.use(pinia)
  routeInterceptor.install()
  return {
    app,
  }
}
