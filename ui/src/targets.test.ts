import { describe, expect, it } from 'vitest'
import { decodeTargets } from './targets'

describe('target decoding', () => {
  it('keeps only the bounded device shape', () => {
    expect(decodeTargets([
      {
        deviceId: '11111111-1111-4111-8111-111111111111',
        displayName: '我的电脑',
        platform: 'windows',
        online: true,
      },
    ])).toHaveLength(1)
  })

  it('rejects duplicate or malformed devices', () => {
    expect(() => decodeTargets([
      {
        deviceId: 'bad',
        displayName: '电脑',
        platform: 'windows',
        online: true,
      },
    ])).toThrow('invalid_devices')
  })
})
