import type { DeviceTarget } from '@synra/client'
import type { HotkeyHost } from './host'
import { computed, onScopeDispose, shallowRef } from 'vue'

export const TARGET_TIMEOUT_MS = 8000
const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
const platforms = new Set(['windows', 'macos', 'linux', 'desktop'])

export function decodeTargets(value: unknown): DeviceTarget[] {
  if (!Array.isArray(value) || value.length > 256)
    throw new Error('invalid_devices')
  const seen = new Set<string>()
  return value.map((item: unknown) => {
    if (typeof item !== 'object' || item === null || Array.isArray(item))
      throw new Error('invalid_devices')
    const row = item as Record<string, unknown>
    if (typeof row.deviceId !== 'string' || !uuid.test(row.deviceId) || seen.has(row.deviceId)
      || typeof row.displayName !== 'string' || !row.displayName.trim() || row.displayName.length > 256
      || typeof row.platform !== 'string' || !platforms.has(row.platform)
      || typeof row.online !== 'boolean') {
      throw new Error('invalid_devices')
    }
    seen.add(row.deviceId)
    return {
      deviceId: row.deviceId as DeviceTarget['deviceId'],
      displayName: row.displayName,
      platform: row.platform as DeviceTarget['platform'],
      online: row.online,
    }
  })
}

export function useTargets(host: HotkeyHost | undefined) {
  const devices = shallowRef<DeviceTarget[]>([])
  const selectedId = shallowRef<DeviceTarget['deviceId'] | ''>('')
  const selected = computed(() => devices.value.find(device => device.deviceId === selectedId.value))
  const pending = shallowRef(false)
  const error = shallowRef(host ? '' : 'bridge_unavailable')
  let disposed = false
  let active = false
  let generation = 0
  let timer: ReturnType<typeof setTimeout> | undefined

  function invalidate() {
    generation++
    clearTimeout(timer)
    devices.value = []
    selectedId.value = ''
  }

  async function refresh() {
    if (!host || active || disposed)
      return
    active = true
    pending.value = true
    error.value = ''
    const current = ++generation
    timer = setTimeout(() => {
      if (current === generation && !disposed) {
        invalidate()
        error.value = 'timeout'
      }
    }, TARGET_TIMEOUT_MS)
    try {
      const answer = await host.device.list()
      if (current === generation && !disposed) {
        devices.value = decodeTargets(answer)
        if (!devices.value.some(device => device.deviceId === selectedId.value))
          selectedId.value = ''
      }
    }
    catch (reason: unknown) {
      if (current === generation && !disposed)
        error.value = reason instanceof Error && reason.message === 'invalid_devices' ? 'invalid_devices' : 'unavailable'
    }
    finally {
      clearTimeout(timer)
      active = false
      if (!disposed)
        pending.value = false
    }
  }

  onScopeDispose(() => {
    disposed = true
    invalidate()
  })

  return { devices, selectedId, selected, pending, error, refresh }
}
