<script setup lang="ts">
import type { HotkeyHost } from './host'
import type { ColumnCount, Shortcut } from './shortcuts'
import { computed, onMounted, onScopeDispose, ref, watch } from 'vue'
import { windowHost } from './host'
import { failureMessage, waitForRemoteResponse } from './remote'
import { COLUMN_OPTIONS, loadPreferences, moveItem, savePreferences, SHORTCUTS } from './shortcuts'
import { useTargets } from './targets'

const props = defineProps<{ host?: HotkeyHost }>()
const host = props.host ?? windowHost()
const isDesktop = host?.env.hostKind() === 'desktop'
const targets = useTargets(host)
const { devices, selectedId, selected, pending: targetsPending, error: targetsError, refresh } = targets
const preferences = loadPreferences()
const columns = ref<ColumnCount>(preferences.columns)
const order = ref(preferences.order)
const editing = ref(false)
const pressedId = ref('')
const pendingIds = ref(new Set<string>())
const toast = ref('')
const drag = ref<{ id: string, pointerId: number, moved: boolean }>()
const suppressClick = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | undefined
let pressedTimer: ReturnType<typeof setTimeout> | undefined

const shortcutById = new Map(SHORTCUTS.map(shortcut => [shortcut.id, shortcut]))
const orderedShortcuts = computed(() => order.value.flatMap((id) => {
  const shortcut = shortcutById.get(id)
  return shortcut ? [shortcut] : []
}))
const targetOnline = computed(() => Boolean(selected.value?.online))
const gridStyle = computed(() => ({ '--columns': String(columns.value) }))

function showToast(message: string) {
  toast.value = message
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toast.value = ''
  }, 2200)
}

function setPending(id: string, active: boolean) {
  const next = new Set(pendingIds.value)
  if (active)
    next.add(id)
  else
    next.delete(id)
  pendingIds.value = next
}

async function sendShortcut(shortcut: Shortcut) {
  if (editing.value || pendingIds.value.has(shortcut.id))
    return
  if (!host) {
    showToast('请从 Synra 打开此插件')
    return
  }
  pressedId.value = shortcut.id
  clearTimeout(pressedTimer)
  pressedTimer = setTimeout(() => {
    pressedId.value = ''
  }, 180)
  if (!selected.value?.online) {
    showToast('请先选择一台在线电脑')
    return
  }
  const requestId = host.requestId()
  setPending(shortcut.id, true)
  try {
    const queued = await host.self.invoke('hotkey.execute', {
      target: selected.value.deviceId,
      requestId,
      shortcutId: shortcut.id,
      platform: selected.value.platform,
    })
    if (typeof queued !== 'object' || queued === null || !('requestId' in queued) || queued.requestId !== requestId)
      throw new Error('invalid_queue_response')
    const response = await waitForRemoteResponse(host, requestId)
    if (!response) {
      showToast('电脑端响应超时')
    }
    else if (response.status === 'ok') {
      showToast(`已发送：${shortcut.title}`)
    }
    else {
      showToast(response.message ?? failureMessage({ code: response.code ?? 'internal' }))
    }
  }
  catch (reason: unknown) {
    showToast(failureMessage(reason))
  }
  finally {
    setPending(shortcut.id, false)
  }
}

function handleShortcutClick(shortcut: Shortcut) {
  if (suppressClick.value) {
    suppressClick.value = false
    return
  }
  void sendShortcut(shortcut)
}

function beginDrag(event: PointerEvent, index: number) {
  if (!editing.value)
    return
  const shortcut = orderedShortcuts.value[index]
  if (!shortcut)
    return
  event.preventDefault()
  const current = event.currentTarget as HTMLElement | null
  current?.setPointerCapture?.(event.pointerId)
  drag.value = { id: shortcut.id, pointerId: event.pointerId, moved: false }
}

function moveDrag(event: PointerEvent) {
  const state = drag.value
  if (!state || state.pointerId !== event.pointerId)
    return
  event.preventDefault()
  const target = (document.elementFromPoint(event.clientX, event.clientY) as HTMLElement | null)?.closest<HTMLElement>('[data-shortcut-id]')
  const targetId = target?.dataset.shortcutId
  if (!targetId || targetId === state.id)
    return
  const from = order.value.indexOf(state.id)
  const to = order.value.indexOf(targetId)
  if (from < 0 || to < 0 || from === to)
    return
  order.value = moveItem(order.value, from, to)
  state.moved = true
}

function endDrag(event: PointerEvent) {
  const state = drag.value
  if (!state || state.pointerId !== event.pointerId)
    return
  const current = event.currentTarget as HTMLElement | null
  if (current?.hasPointerCapture?.(event.pointerId))
    current.releasePointerCapture(event.pointerId)
  if (state.moved) {
    suppressClick.value = true
    setTimeout(() => {
      suppressClick.value = false
    }, 220)
  }
  drag.value = undefined
}

function finishEditing() {
  editing.value = false
  drag.value = undefined
}

watch([columns, order], () => savePreferences({ columns: columns.value, order: order.value }))

onMounted(() => {
  document.documentElement.dataset.theme = host?.env.darkMode() ? 'dark' : 'light'
  if (!isDesktop)
    void refresh()
})

onScopeDispose(() => {
  clearTimeout(toastTimer)
  clearTimeout(pressedTimer)
})
</script>

<template>
  <main :class="{ receiver: isDesktop, editing }">
    <header class="heading">
      <div>
        <p class="eyebrow">
          SYNRA HOTKEY
        </p>
        <h1>热键</h1>
      </div>
      <button v-if="!isDesktop" type="button" class="edit-button" @click="editing ? finishEditing() : editing = true">
        {{ editing ? '完成' : '编辑' }}
      </button>
    </header>

    <section v-if="isDesktop" class="receiver-panel" aria-labelledby="receiver-title">
      <span class="receiver-icon i-carbon-keyboard" aria-hidden="true" />
      <h2 id="receiver-title">
        电脑端已就绪
      </h2>
      <p>请在手机端打开热键，选择这台电脑后点击快捷键。</p>
      <p class="muted">
        执行快捷键需要在电脑端允许“模拟按键”权限。
      </p>
    </section>

    <template v-else>
      <section class="control-panel" aria-label="热键设置">
        <div class="target-row">
          <label for="target-computer">电脑</label>
          <select id="target-computer" v-model="selectedId" :disabled="!host || targetsPending">
            <option value="">
              {{ targetsPending ? '正在读取…' : '选择电脑' }}
            </option>
            <option v-for="device in devices" :key="device.deviceId" :value="device.deviceId" :disabled="!device.online">
              {{ device.displayName }}{{ device.online ? '' : ' · 离线' }}
            </option>
          </select>
          <button type="button" class="quiet compact" :disabled="!host || targetsPending" @click="refresh">
            刷新
          </button>
        </div>
        <div class="toolbar">
          <div class="column-picker" role="group" aria-label="每行列数">
            <span>每行</span>
            <button v-for="option in COLUMN_OPTIONS" :key="option" type="button" :class="{ selected: columns === option }" :aria-pressed="columns === option" @click="columns = option">
              {{ option }}
            </button>
            <span>列</span>
          </div>
          <span v-if="editing" class="edit-hint" role="status">拖拽方块调整顺序</span>
        </div>
        <p v-if="targetsError === 'bridge_unavailable'" class="notice" role="alert">
          请从 Synra 打开此插件。
        </p>
        <p v-else-if="targetsError" class="notice" role="alert">
          暂时无法读取电脑列表，请稍后刷新。
        </p>
        <p v-else-if="!selected" class="muted" role="status">
          选择一台在线电脑后即可发送快捷键。
        </p>
        <p v-else-if="!targetOnline" class="notice" role="status">
          当前电脑已离线，请选择在线设备。
        </p>
      </section>

      <section class="shortcut-grid" :data-columns="columns" :style="gridStyle" aria-label="系统快捷键">
        <button
          v-for="(shortcut, index) in orderedShortcuts"
          :key="shortcut.id"
          :data-shortcut-id="shortcut.id"
          :data-testid="`shortcut-${shortcut.id}`"
          type="button"
          class="shortcut-card"
          :class="{ pressed: pressedId === shortcut.id, pending: pendingIds.has(shortcut.id), dragging: drag?.id === shortcut.id }"
          :aria-label="`${shortcut.title}，${shortcut.chord}`"
          :aria-busy="pendingIds.has(shortcut.id)"
          :disabled="pendingIds.has(shortcut.id)"
          @click="handleShortcutClick(shortcut)"
          @pointerdown="beginDrag($event, index)"
          @pointermove="moveDrag"
          @pointerup="endDrag"
          @pointercancel="endDrag"
          @lostpointercapture="endDrag"
        >
          <span class="shortcut-icon" :class="shortcut.icon" aria-hidden="true" />
          <span class="shortcut-title">{{ shortcut.title }}</span>
          <span class="shortcut-chord">{{ shortcut.chord }}</span>
        </button>
      </section>

      <p v-if="toast" class="toast" role="status">
        {{ toast }}
      </p>
    </template>
  </main>
</template>
