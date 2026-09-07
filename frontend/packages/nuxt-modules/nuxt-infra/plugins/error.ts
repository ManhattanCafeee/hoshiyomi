function isChunkLoadError(err: unknown): boolean {
  return err instanceof TypeError && err.message.includes('error loading dynamically imported module')
}

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.config.errorHandler = (_err: unknown) => {}

  nuxtApp.hook('vue:error', (err) => {
    if (err instanceof AbortError) {
      if (err.message) console.log(err.message)
      return
    }
    if (isChunkLoadError(err)) {
      window.location.reload()
      return
    }
    console.error('vue:error', err)
  })

  nuxtApp.hook('app:error', (err) => {
    if (isChunkLoadError(err)) {
      window.location.reload()
      return
    }
    console.error('app:error', err)
  })
})
