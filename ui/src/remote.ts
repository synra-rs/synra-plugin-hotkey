import type { HotkeyHost } from './host'
import { errorCode, messageFor } from './host'

export interface RemoteResponse {
  requestId: string
  status: 'pending' | 'ok' | 'error'
  code?: string
  message?: string
}

export function decodeRemoteResponse(value: unknown): RemoteResponse | undefined {
  if (typeof value !== 'object' || value === null || Array.isArray(value))
    return undefined
  const row = value as Record<string, unknown>
  if (typeof row.requestId !== 'string' || !['pending', 'ok', 'error'].includes(String(row.status)))
    return undefined
  return {
    requestId: row.requestId,
    status: row.status as RemoteResponse['status'],
    code: typeof row.code === 'string' ? row.code : undefined,
    message: typeof row.message === 'string' ? row.message : undefined,
  }
}

export async function waitForRemoteResponse(host: HotkeyHost, requestId: string, timeoutMs = 5000): Promise<RemoteResponse | undefined> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const answer = decodeRemoteResponse(await host.self.invoke('hotkey.response-poll', { requestId }))
      if (answer && answer.status !== 'pending')
        return answer
    }
    catch {
      return undefined
    }
    await new Promise(resolve => setTimeout(resolve, 120))
  }
  return undefined
}

export function failureMessage(reason: unknown): string {
  const message = typeof reason === 'string'
    ? reason
    : typeof reason === 'object' && reason !== null && 'message' in reason && typeof reason.message === 'string'
      ? reason.message
      : ''
  if (/simulating keys is not enabled/i.test(message))
    return '请在电脑端 Synra 设置中开启“允许插件模拟按键”，再重试。'
  return messageFor(errorCode(reason))
}
