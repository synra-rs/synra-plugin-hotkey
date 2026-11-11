import type { HotkeyHost } from './host'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it } from 'vitest'
import App from './App.vue'

const computer = {
  deviceId: '11111111-1111-4111-8111-111111111111',
  displayName: '我的电脑',
  platform: 'windows',
  online: true,
} as const

function hostWith(calls: Array<{ action: string, args: unknown }>): HotkeyHost {
  return {
    env: {
      hostKind: () => 'mobile',
      platform: () => 'android',
      locale: () => 'zh-CN',
      darkMode: () => false,
      appVersion: () => '0.1.0',
      sdkVersion: () => '1.0.0',
      platformSupports: () => false,
      hostSupports: () => false,
      can: () => true,
    },
    device: {
      list: async () => [computer],
      send: async () => undefined,
      broadcast: async () => undefined,
      onChange: () => () => undefined,
      pick: async () => undefined,
      supported: [],
    },
    self: {
      async invoke(action: string, args?: unknown) {
        calls.push({ action, args })
        if (action === 'hotkey.execute')
          return { status: 'queued', requestId: (args as { requestId: string }).requestId }
        return { status: 'ok', requestId: (args as { requestId: string }).requestId }
      },
      async actions() { return [] },
    },
    requestId: () => 'request-1',
  } as unknown as HotkeyHost
}

describe('hotkey panel', () => {
  beforeEach(() => localStorage.clear())

  it('opens with Chinese shortcut labels and dispatches a fixed shortcut id', async () => {
    const calls: Array<{ action: string, args: unknown }> = []
    const wrapper = mount(App, { props: { host: hostWith(calls) } })
    await flushPromises()
    expect(wrapper.text()).toContain('复制')
    expect(wrapper.get('[data-testid="shortcut-copy"] .shortcut-icon').classes()).toContain('i-carbon-copy')
    await wrapper.get('#target-computer').setValue(computer.deviceId)
    await wrapper.get('[data-testid="shortcut-copy"]').trigger('click')
    await flushPromises()
    expect(calls.map(call => call.action)).toEqual(['hotkey.execute', 'hotkey.response-poll'])
    expect(calls[0]?.args).toMatchObject({ target: computer.deviceId, shortcutId: 'copy', platform: 'windows' })
  })

  it('switches between three and five columns without changing the shortcut list', async () => {
    const wrapper = mount(App, { props: { host: hostWith([]) } })
    await wrapper.get('.column-picker button:nth-of-type(3)').trigger('click')
    expect(wrapper.get('.shortcut-grid').attributes('data-columns')).toBe('5')
    expect(wrapper.text()).toContain('打开文件夹')
  })

  it('does not dispatch a shortcut while editing', async () => {
    const calls: Array<{ action: string, args: unknown }> = []
    const wrapper = mount(App, { props: { host: hostWith(calls) } })
    await flushPromises()
    await wrapper.get('#target-computer').setValue(computer.deviceId)
    await wrapper.get('.edit-button').trigger('click')
    await wrapper.get('[data-testid="shortcut-copy"]').trigger('click')
    await flushPromises()
    expect(calls).toEqual([])
  })
})
