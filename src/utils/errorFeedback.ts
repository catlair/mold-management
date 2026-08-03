import { ElNotification } from 'element-plus'

export interface NamedError {
  label: string
  error: unknown
}

export interface NamedRequest {
  label: string
  request: Promise<unknown>
}

export async function settleNamedRequests(requests: NamedRequest[]) {
  const results = await Promise.allSettled(requests.map(item => item.request))
  const failures: NamedError[] = []
  const values = results.map((result, index) => {
    if (result.status === 'fulfilled') return result.value
    failures.push({ label: requests[index].label, error: result.reason })
    return undefined
  })
  return { values, failures }
}

const recentNotifications = new Map<string, number>()

export function getErrorReason(error: unknown): string {
  if (typeof error === 'string') return error.trim() || '后端未返回具体原因'
  if (error instanceof Error) return error.message.trim() || error.name || '未知错误'
  if (error && typeof error === 'object') {
    const value = error as Record<string, unknown>
    for (const key of ['message', 'error', 'reason', 'cause']) {
      if (typeof value[key] === 'string' && value[key].trim()) return value[key].trim()
    }
    try {
      return JSON.stringify(error)
    } catch {
      return String(error)
    }
  }
  return String(error || '未知错误')
}

export function isUserCancellation(error: unknown): boolean {
  return error === 'cancel' || error === 'close'
}

export function getErrorSuggestion(reason: string): string {
  const normalized = reason.toLowerCase()
  if (/being used|占用|sharing violation|os error 32|另一个程序/.test(normalized)) {
    return '请关闭正在打开该数据文件的 Excel/WPS 窗口，然后重试。'
  }
  if (/permission denied|access is denied|拒绝访问|os error 5|权限/.test(normalized)) {
    return '请检查数据文件和所在目录是否可写，必要时将文件移到有读写权限的目录。'
  }
  if (/worksheet|sheet|工作表|range not found/.test(normalized)) {
    return '当前 Excel 缺少程序需要的工作表；请在“配置”中确认选择了正确的数据文件。'
  }
  if (/not found|no such file|找不到|不存在/.test(normalized)) {
    return '请在“配置”中检查数据文件路径，确认文件未被移动或删除。'
  }
  if (/zip|corrupt|invalid.*xlsx|损坏|格式/.test(normalized)) {
    return '数据文件可能损坏或不是有效的 XLSX 文件；请先从备份恢复后再重试。'
  }
  if (/unknown command|command .* not found|未找到命令/.test(normalized)) {
    return '前后端版本不一致；请完全退出并重新启动桌面应用。'
  }
  return '请重试；若仍失败，请保留此错误原因，并检查“配置”中的数据文件路径。'
}

function notifyOnce(title: string, message: string) {
  const key = `${title}|${message}`
  const now = Date.now()
  if (now - (recentNotifications.get(key) || 0) < 1500) return
  recentNotifications.set(key, now)
  for (const [storedKey, time] of recentNotifications) {
    if (now - time > 10_000) recentNotifications.delete(storedKey)
  }

  ElNotification({
    title,
    message,
    type: 'error',
    duration: 12_000,
    showClose: true,
    customClass: 'detailed-error-notification',
  })
}

export function showDetailedError(action: string, error: unknown, suggestion?: string) {
  const reason = getErrorReason(error)
  notifyOnce(
    `${action}失败`,
    `错误原因：${reason}\n处理建议：${suggestion || getErrorSuggestion(reason)}`,
  )
  console.error(`[${action}失败]`, error)
}

export function showBatchErrors(action: string, failures: NamedError[]) {
  if (!failures.length) return
  const details = failures.map(({ label, error }) => `${label}：${getErrorReason(error)}`).join('\n')
  const suggestions = [...new Set(failures.map(({ error }) => getErrorSuggestion(getErrorReason(error))))]
  notifyOnce(
    `${action}部分失败`,
    `失败项目：\n${details}\n处理建议：${suggestions.join('；')}`,
  )
  console.error(`[${action}部分失败]`, failures)
}
