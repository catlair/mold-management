<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="tolerance-keyboard"
      :style="panelStyle"
      role="dialog"
      aria-label="公差输入虚拟键盘"
      @mousedown.prevent.stop
    >
      <div class="keyboard-head">
        <span>公差输入</span>
        <el-button text size="small" aria-label="关闭虚拟键盘" @click="$emit('close')">
          <el-icon><Close /></el-icon>
        </el-button>
      </div>

      <div class="keyboard-main">
        <div class="number-grid">
          <button v-for="key in numberKeys" :key="key" type="button" class="vk-key vk-number" @click="$emit('insert', key)">{{ key }}</button>
        </div>
        <div class="quick-grid">
          <button v-for="key in quickKeys" :key="key" type="button" class="vk-key vk-quick" @click="$emit('insert', key)">{{ key }}</button>
        </div>
      </div>

      <div class="keyboard-actions">
        <button type="button" class="vk-key vk-action" @click="$emit('backspace')"><el-icon><Back /></el-icon> 退格</button>
        <button type="button" class="vk-key vk-action" @click="$emit('clear')">清空</button>
        <button type="button" class="vk-key vk-action vk-done" @click="$emit('done')">完成</button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  visible: boolean
  quickKeys: string[]
  anchorRect?: { left: number; top: number; bottom: number; width: number } | null
}>(), {
  anchorRect: null,
})

defineEmits<{
  insert: [value: string]
  backspace: []
  clear: []
  done: []
  close: []
}>()

const numberKeys = ['7', '8', '9', '4', '5', '6', '1', '2', '3', '0', '00', '.']

const panelStyle = computed(() => {
  const rect = props.anchorRect
  if (!rect || typeof window === 'undefined') return {}
  const panelWidth = 420
  const left = Math.min(Math.max(8, rect.left), Math.max(8, window.innerWidth - panelWidth - 8))
  // 优先显示在输入框下方；下方空间不足时翻到上方。
  const estimatedHeight = 245
  const below = rect.bottom + 8
  const top = below + estimatedHeight <= window.innerHeight
    ? below
    : Math.max(8, rect.top - estimatedHeight - 8)
  return { left: `${left}px`, top: `${top}px` }
})
</script>

<style scoped>
.tolerance-keyboard {
  position: fixed;
  z-index: 5000;
  width: 420px;
  max-width: calc(100vw - 16px);
  padding: 10px;
  box-sizing: border-box;
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  background: var(--card-bg);
  color: var(--text-primary);
  box-shadow: 0 14px 38px rgba(15, 23, 42, 0.22);
  user-select: none;
}
.keyboard-head {
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 2px 7px 6px;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
  font-weight: 600;
}
.keyboard-main { display: grid; grid-template-columns: 142px 1fr; gap: 9px; padding-top: 9px; }
.number-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.quick-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 6px; align-content: start; max-height: 170px; overflow-y: auto; }
.vk-key {
  min-height: 36px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface-muted);
  color: var(--text-primary);
  font-family: inherit;
  font-size: 14px;
  cursor: pointer;
  transition: background-color .14s ease, border-color .14s ease, transform .08s ease;
}
.vk-key:hover { background: var(--surface-hover); border-color: var(--primary); }
.vk-key:active { transform: scale(.96); }
.vk-number { font-size: 16px; font-variant-numeric: tabular-nums; }
.vk-quick { padding: 4px 6px; overflow-wrap: anywhere; }
.keyboard-actions { display: grid; grid-template-columns: 1fr 1fr 1.25fr; gap: 7px; padding-top: 9px; }
.vk-action { display: inline-flex; align-items: center; justify-content: center; gap: 5px; }
.vk-done { border-color: var(--primary); background: var(--primary); color: #fff; }
.vk-done:hover { background: var(--primary-light); color: #fff; }
</style>
