import { describe, expect, it } from 'vitest'
import { failureMessage } from './remote'

describe('remote failure messages', () => {
  it('explains the separate host policy for key simulation', () => {
    expect(failureMessage({ code: 'badArgument', message: 'simulating keys is not enabled; the user has to turn it on in settings' })).toContain('允许插件模拟按键')
  })
})
