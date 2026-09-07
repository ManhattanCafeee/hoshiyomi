import { toast } from '~/components/ui/toast'
import { BizError } from '@hoshiyomi/alova/lib'

export function useNotify() {
  function success(title: string, description?: string) {
    toast({ title, description })
  }

  function error(titleOrError: string | Error, fallback?: string) {
    const title = titleOrError instanceof Error ? titleOrError.message : titleOrError
    toast({ title: title || fallback || '操作失败', variant: 'destructive' })
  }

  function apiError(err: unknown, fallback?: string) {
    let message = fallback || '操作失败'
    if (err instanceof BizError) {
      message = (err.data as { message?: string })?.message ?? err.message
    } else if (err instanceof Error) {
      message = err.message
    } else if (err && typeof err === 'object') {
      const e = err as Record<string, unknown>
      message = (e.message ?? e.msg ?? message) as string
    }
    toast({ title: message, variant: 'destructive' })
  }

  return { success, error, apiError }
}
