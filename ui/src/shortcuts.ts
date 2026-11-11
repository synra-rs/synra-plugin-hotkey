export interface Shortcut {
  id: string
  title: string
  chord: string
  icon: string
}

export const SHORTCUTS: readonly Shortcut[] = [
  { id: 'copy', title: '复制', chord: 'Ctrl C', icon: '⧉' },
  { id: 'cut', title: '剪切', chord: 'Ctrl X', icon: '✂' },
  { id: 'paste', title: '粘贴', chord: 'Ctrl V', icon: '▣' },
  { id: 'undo', title: '撤销', chord: 'Ctrl Z', icon: '↶' },
  { id: 'redo', title: '重做', chord: 'Ctrl Y', icon: '↷' },
  { id: 'select-all', title: '全选', chord: 'Ctrl A', icon: '☷' },
  { id: 'save', title: '保存', chord: 'Ctrl S', icon: '▤' },
  { id: 'find', title: '查找', chord: 'Ctrl F', icon: '⌕' },
  { id: 'open-folder', title: '打开文件夹', chord: 'Win E', icon: '▰' },
]

export type ColumnCount = 3 | 4 | 5
export const COLUMN_OPTIONS: readonly ColumnCount[] = [3, 4, 5]
export const PREFERENCES_KEY = 'synra-plugin-hotkey.preferences'

const shortcutIds = new Set(SHORTCUTS.map(shortcut => shortcut.id))

export function defaultOrder(): string[] {
  return SHORTCUTS.map(shortcut => shortcut.id)
}

export function normalizeOrder(value: unknown): string[] {
  const order: string[] = []
  if (Array.isArray(value)) {
    for (const item of value) {
      if (typeof item === 'string' && shortcutIds.has(item) && !order.includes(item))
        order.push(item)
    }
  }
  for (const shortcut of SHORTCUTS) {
    if (!order.includes(shortcut.id))
      order.push(shortcut.id)
  }
  return order
}

export function moveItem<T>(items: readonly T[], from: number, to: number): T[] {
  if (from < 0 || from >= items.length || to < 0 || to >= items.length || from === to)
    return [...items]
  const next = [...items]
  const [item] = next.splice(from, 1)
  if (item !== undefined)
    next.splice(to, 0, item)
  return next
}

export interface Preferences {
  columns: ColumnCount
  order: string[]
}

export function loadPreferences(storage: Storage | undefined = typeof localStorage === 'undefined' ? undefined : localStorage): Preferences {
  const fallback: Preferences = { columns: 3, order: defaultOrder() }
  if (!storage)
    return fallback
  try {
    const raw = storage.getItem(PREFERENCES_KEY)
    if (!raw)
      return fallback
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed))
      return fallback
    const row = parsed as Record<string, unknown>
    const columns = row.columns === 4 || row.columns === 5 ? row.columns : 3
    return { columns, order: normalizeOrder(row.order) }
  }
  catch {
    return fallback
  }
}

export function savePreferences(preferences: Preferences, storage: Storage | undefined = typeof localStorage === 'undefined' ? undefined : localStorage): void {
  if (!storage)
    return
  try {
    storage.setItem(PREFERENCES_KEY, JSON.stringify({ columns: preferences.columns, order: normalizeOrder(preferences.order) }))
  }
  catch {
    // Storage is a preference only; an unavailable store must not block shortcuts.
  }
}
