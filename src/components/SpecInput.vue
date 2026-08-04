<template>
  <div class="spec-input">
    <el-input
      v-model="inputValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :clearable="clearable"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { WarningFilled } from '@element-plus/icons-vue'
import { parseSpecText } from '../utils/specNormalize'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  clearable?: boolean
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  clearable: false,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const inputValue = ref(props.modelValue ?? '')

watch(
  () => props.modelValue,
  (value) => {
    if (value !== inputValue.value) {
      inputValue.value = value ?? ''
    }
  },
)

watch(inputValue, (value) => {
  emit('update:modelValue', value)
})

const parsed = computed(() => parseSpecText(inputValue.value))

/** 失焦确认：把输入统一为规范显示值 */
function handleBlur() {
  if (!inputValue.value.trim()) return
  const normalized = parsed.value.display
  if (normalized && normalized !== inputValue.value) {
    inputValue.value = normalized
  }
}
</script>

<style scoped>
/* 根容器占满父容器，保证所有 SpecInput 输入框宽度一致 */
.spec-input {
  width: 100%;
}
/* 信息区域固定 22px 高度（与一行预览文字一致），无论是否有内容都占位，
   保证多个 SpecInput 字段（左右两列）视觉高度整齐 */
.spec-input-info {
  margin-top: 4px;
  height: 22px;
  line-height: 22px;
  font-size: 12px;
  overflow: hidden;
}
.spec-input-preview {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  line-height: 22px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.spec-input-preview-label {
  color: var(--text-muted);
  flex-shrink: 0;
}
.spec-input-preview-value {
  color: var(--primary);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
}
.spec-input-warning {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  line-height: 22px;
  color: var(--warning-color, #e6a23c);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.spec-input-hint {
  font-size: 12px;
  line-height: 22px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
