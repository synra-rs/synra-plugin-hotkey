import { describe, expect, it } from 'vitest'
import { defaultOrder, moveItem, normalizeOrder } from './shortcuts'

describe('shortcut ordering', () => {
  it('keeps built-ins and appends missing ids after a saved partial order', () => {
    expect(normalizeOrder(['paste', 'paste', 'unknown', 'copy'])).toEqual([
      'paste',
      'copy',
      'cut',
      'undo',
      'redo',
      'select-all',
      'save',
      'find',
      'open-folder',
    ])
  })

  it('moves one item without mutating the saved order', () => {
    const order = defaultOrder()
    expect(moveItem(order, 0, 2).slice(0, 3)).toEqual(['cut', 'paste', 'copy'])
    expect(order.slice(0, 3)).toEqual(['copy', 'cut', 'paste'])
  })
})
