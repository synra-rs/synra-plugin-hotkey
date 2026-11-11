import type { SynraClient } from '@synra/client'

export type HotkeyHost = Pick<SynraClient, 'env' | 'device' | 'self' | 'requestId'>

export function windowHost(): HotkeyHost | undefined {
  return typeof window === 'undefined' ? undefined : window.synra
}

export function errorCode(reason: unknown): string {
  if (typeof reason === 'object' && reason !== null && 'code' in reason && typeof reason.code === 'string')
    return reason.code
  if (reason instanceof Error && reason.message === 'timeout')
    return 'timeout'
  return 'internal'
}

const messages: Record<string, string> = {
  bridge_unavailable: '请从 Synra 打开此插件。',
  capability_denied: '插件权限不足，请在 Synra 中允许设备发送和电脑端按键权限。',
  device_offline: '目标电脑已离线，请刷新设备列表后重试。',
  timeout: '电脑端响应超时，请确认电脑在线后重试。',
  unsupported: '当前电脑不支持模拟快捷键。',
  badArgument: '快捷键请求参数无效。',
  internal: '快捷键请求未完成，请稍后重试。',
}

export function messageFor(code: string): string {
  return messages[code] ?? messages.internal
}
