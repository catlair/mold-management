<template>
  <div class="spec-input">
    <el-input
      ref="inputRef"
      v-model="inputValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :clearable="clearable"
      @focus="openKeyboard"
      @click="openKeyboard"
      @keyup="captureSelection"
      @blur="handleBlur"
    />
    <!-- 信息区域固定高度，确保多个 SpecInput 字段之间高度对齐 -->
    <div class="spec-input-info">
      <div v-if="parsed.warning" class="spec-input-warning">
        <el-icon><WarningFilled /></el-icon>
        <span>{{ parsed.warning }}</span>
      </div>
      <div v-else-if="parsed.kind === 'text' && inputValue.trim()" class="spec-input-hint">
        将以原文保存（未识别为尺寸/公差格式）
      </div>
      <div v-else-if="parsed.display && inputValue.trim() !== parsed.display" class="spec-input-preview">
        <span class="spec-input-preview-label">将保存为</span>
        <span class="spec-input-preview-value">{{ parsed.display }}</span>
      </div>
    </div>

    <ToleranceKeyboard
      :visible="keyboardVisible"
      :quick-keys="keyboardSettings.quickKeys"
      :anchor-rect="anchorRect"
      @insert="insertAtCursor"
      @backspace="backspace"
      @clear="clearInput"
      @done="finishInput"
      @close="closeKeyboard"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onBeforeUnmount } from 'vue'
import { WarningFilled } from '@element-plus/icons-vue'
import { parseSpecText } from '../utils/specNormalize'
import { useToleranceKeyboard } from '../composables/useToleranceKeyboard'
import ToleranceKeyboard from './ToleranceKeyboard.vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  clearable?: boolean
  /** 是否在聚焦时启用公差虚拟键盘 */
  virtualKeyboard?: boolean
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  clearable: false,
  virtualKeyboard: false,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const inputValue = ref(props.modelValue ?? '')
const inputRef = ref<any>(null)
const keyboardVisible = ref(false)
const anchorRect = ref<{ left: number; top: number; bottom: number; width: number } | null>(null)
const selectionStart = ref(0)
const selectionEnd = ref(0)
const { settings: keyboardSettings } = useToleranceKeyboard()

function nativeInput(): HTMLInputElement | null {
  const root = inputRef.value?.$el as HTMLElement | undefined
  return root?.querySelector('input') ?? null
}

watch(
  () => props.modelValue,
  (value) => {
    if (value !== inputValue.value) inputValue.value = value ?? ''
  },
)

watch(inputValue, (value) => emit('update:modelValue', value))
watch(() => keyboardSettings.enabled, (enabled) => {
  if (!enabled) keyboardVisible.value = false
})

const parsed = computed(() => parseSpecText(inputValue.value))

function captureSelection() {
  const input = nativeInput()
  if (!input) return
  selectionStart.value = input.selectionStart ?? inputValue.value.length
  selectionEnd.value = input.selectionEnd ?? selectionStart.value
}

function openKeyboard() {
  captureSelection()
  if (!props.virtualKeyboard || !keyboardSettings.enabled || props.disabled) return
  const input = nativeInput()
  if (!input) return
  // focus 与 click 都调用：即使输入框已保持焦点，关闭后再次点击也能重新唤起。
  if (!keyboardVisible.value) {
    window.dispatchEvent(new CustomEvent('tolerance-keyboard-open', { detail: input }))
  }
  const rect = input.getBoundingClientRect()
  anchorRect.value = { left: rect.left, top: rect.top, bottom: rect.bottom, width: rect.width }
  keyboardVisible.value = true
}

async function restoreCursor(position: number) {
  await nextTick()
  const input = nativeInput()
  if (!input) return
  input.focus({ preventScroll: true })
  input.setSelectionRange(position, position)
  selectionStart.value = position
  selectionEnd.value = position
}

function insertAtCursor(text: string) {
  const start = Math.min(selectionStart.value, inputValue.value.length)
  const end = Math.min(selectionEnd.value, inputValue.value.length)
  inputValue.value = `${inputValue.value.slice(0, start)}${text}${inputValue.value.slice(end)}`
  restoreCursor(start + text.length)
}

function backspace() {
  let start = Math.min(selectionStart.value, inputValue.value.length)
  const end = Math.min(selectionEnd.value, inputValue.value.length)
  if (start !== end) {
    inputValue.value = `${inputValue.value.slice(0, start)}${inputValue.value.slice(end)}`
  } else if (start > 0) {
    inputValue.value = `${inputValue.value.slice(0, start - 1)}${inputValue.value.slice(start)}`
    start -= 1
  }
  restoreCursor(start)
}

function clearInput() {
  inputValue.value = ''
  restoreCursor(0)
}

function closeKeyboard() {
  keyboardVisible.value = false
}

function finishInput() {
  normalizeValue()
  closeKeyboard()
}

/** 失焦确认：把输入统一为规范显示值。点击虚拟键盘会 preventDefault，不会触发失焦。 */
function handleBlur() {
  window.setTimeout(() => {
    if (!keyboardVisible.value) normalizeValue()
  }, 0)
}

function normalizeValue() {
  if (!inputValue.value.trim()) return
  const normalized = parseSpecText(inputValue.value).display
  if (normalized && normalized !== inputValue.value) inputValue.value = normalized
}

function handleWindowChange() {
  if (keyboardVisible.value) closeKeyboard()
}
function handleOtherKeyboard(event: Event) {
  const target = (event as CustomEvent).detail
  if (target !== nativeInput() && keyboardVisible.value) {
    normalizeValue()
    closeKeyboard()
  }
}
function handleDocumentMouseDown(event: MouseEvent) {
  if (!keyboardVisible.value) return
  const root = inputRef.value?.$el as HTMLElement | undefined
  if (root?.contains(event.target as Node)) return
  // 虚拟键盘自身通过 mousedown.stop 阻止事件冒泡；到达此处表示点击了键盘和输入框之外。
  normalizeValue()
  closeKeyboard()
}
window.addEventListener('resize', handleWindowChange)
window.addEventListener('scroll', handleWindowChange, true)
window.addEventListener('tolerance-keyboard-open', handleOtherKeyboard)
document.addEventListener('mousedown', handleDocumentMouseDown)
onBeforeUnmount(() => {
  window.removeEventListener('resize', handleWindowChange)
  window.removeEventListener('scroll', handleWindowChange, true)
  window.removeEventListener('tolerance-keyboard-open', handleOtherKeyboard)
  document.removeEventListener('mousedown', handleDocumentMouseDown)
})
</script>

<style scoped>
/* 根容器占满父容器，保证所有 SpecInput 输入框宽度一致 */
.spec-input { width: 100%; }
/* 无提示时缩小占位；出现规范提示时恢复一行高度，避免文字被裁切 */
.spec-input-info { margin-top: 2px; height: 8px; line-height: 8px; font-size: 12px; overflow: hidden; }
.spec-input-info:has(> *) { height: 22px; line-height: 22px; }
.spec-input-preview { display: flex; align-items: center; gap: 6px; font-size: 12px; line-height: 22px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.spec-input-preview-label { color: var(--text-muted); flex-shrink: 0; }
.spec-input-preview-value { color: var(--primary); font-weight: 600; font-variant-numeric: tabular-nums; overflow: hidden; text-overflow: ellipsis; }
.spec-input-warning { display: flex; align-items: center; gap: 5px; font-size: 12px; line-height: 22px; color: var(--warning-color, #e6a23c); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.spec-input-hint { font-size: 12px; line-height: 22px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
