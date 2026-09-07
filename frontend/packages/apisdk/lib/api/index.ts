import { getResponseData, toRequestInfo, toResponseInfo } from '@hoshiyomi/alova/adapter/fetch'
import { createAlovaHandlers, createEventSystem, multipartBeforeRequest } from '@hoshiyomi/alova/lib'
import { createAlova } from 'alova'
import fetchAdapter from 'alova/fetch'
import VueHook from 'alova/vue'
import { createApis, mountApis, withConfigType } from './createApis'

export const eventSystem = createEventSystem()

const handlers = createAlovaHandlers(eventSystem.emit)

export const alovaInstance = createAlova({
  statesHook: VueHook,
  requestAdapter: fetchAdapter(),
  beforeRequest(method) {
    // 会话认证:同源请求自动携带 session Cookie(开发走 Vite 代理,生产同源反代),无需手动附头
    const req = toRequestInfo(method)
    multipartBeforeRequest(req)
    handlers.beforeRequest(req)
  },
  cacheFor: null,
  responded: {
    async onSuccess(response, method) {
      const respData = await getResponseData(response)
      // hoshiyomi 信封:code 为数字 0 表示成功(后端为 Rust i32,与参考 Go 后端的字符串 '0' 不同)
      const unwrapped = respData?.code === 0 && respData?.data !== undefined ? respData.data : respData
      return handlers.onSuccess(toResponseInfo(response), toRequestInfo(method), unwrapped)
    },
    async onError(error, method) {
      return handlers.onError(error, toRequestInfo(method), await getResponseData(error.response))
    },
    onComplete(method) {
      return handlers.onComplete(toRequestInfo(method))
    },
  },
})

export const $$userConfigMap = withConfigType({})

const Apis = createApis(alovaInstance, $$userConfigMap)

mountApis(Apis)

export default Apis
