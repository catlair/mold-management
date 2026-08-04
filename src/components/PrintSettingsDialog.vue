<template>
  <el-dialog
    :model-value="modelValue"
    :title="mode === 'print' ? '打印设置' : '导出 PDF 设置'"
    width="760px"
    append-to-body
    class="print-settings-dialog"
    @update:model-value="emit('update:modelValue', $event)"
    @closed="resetDraft"
  >
    <div class="settings-layout">
      <section class="settings-columns">
        <div class="settings-section-title">
          <span>打印列</span>
          <div class="settings-actions">
            <el-button link type="primary" size="small" @click="selectAll">全选</el-button>
            <el-button link size="small" @click="clearAll">清空</el-button>
          </div>
        </div>
        <el-checkbox-group v-model="draft.enabledFields" class="settings-column-list">
          <el-checkbox v-for="column in columns" :key="column.field" :value="column.field">
            <span class="column-label">{{ column.label }}</span>
            <span class="column-short">打印为「{{ column.shortLabel }}」</span>
          </el-checkbox>
        </el-checkbox-group>
      </section>

      <section class="settings-options">
        <div class="settings-section-title">版式选项</div>

        <div class="option-row">
          <span class="option-label">方向</span>
          <el-radio-group v-model="draft.portrait">
            <el-radio :value="true">竖向 A4</el-radio>
            <el-radio :value="false">横向 A4</el-radio>
          </el-radio-group>
          <span class="option-hint">{{ draft.portrait ? '210 × 297mm' : '297 × 210mm' }}</span>
        </div>

        <div class="option-row">
          <span class="option-label">字体</span>
          <el-select v-model="draft.fontFamily" size="small" style="width: 132px">
            <el-option v-for="font in FONT_FAMILY_OPTIONS" :key="font" :label="font" :value="font" />
          </el-select>
        </div>

        <div class="option-row">
          <span class="option-label">字号</span>
          <el-select v-model="draft.fontSize" size="small" style="width: 132px">
            <el-option v-for="size in FONT_SIZE_OPTIONS" :key="size" :label="`${size} pt`" :value="size" />
          </el-select>
        </div>

        <div class="option-row option-stripe">
          <span class="option-label">隔行颜色</span>
          <div class="stripe-controls">
            <el-switch v-model="draft.striped" inline-prompt active-text="开" inactive-text="关" />
            <el-color-picker
              v-model="draft.stripeColor"
              size="small"
              :disabled="!draft.striped"
              :predefine="STRIPE_COLOR_OPTIONS"
            />
            <span class="stripe-preview" :class="{ 'is-disabled': !draft.striped }">
              <i />
              <i :style="{ backgroundColor: draft.striped ? draft.stripeColor : '#ffffff' }" />
              <i />
            </span>
          </div>
          <span class="option-hint">仅数据行交替着色，空白填充行保持白色</span>
        </div>

        <div class="option-row option-style">
          <span class="option-label">风格</span>
          <div class="style-picker">
            <button
              v-for="style in PRINT_STYLE_OPTIONS"
              :key="style.id"
              type="button"
              class="style-item"
              :class="{ 'is-active': draft.styleId === style.id }"
              @click="draft.styleId = style.id"
            >
              <span class="style-swatch" :style="{ background: style.headerCss }" />
              <span class="style-name">{{ style.name }}</span>
            </button>
          </div>
        </div>
      </section>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button link @click="resetAll">恢复默认</el-button>
        <span class="footer-spacer" />
        <el-button @click="emit('update:modelValue', false)">取消</el-button>
        <el-button
          type="primary"
          :disabled="!draft.enabledFields.length"
          @click="confirmSettings"
        >
          {{ mode === 'print' ? '打 印' : '导出 PDF' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { PrintColumn } from '../config/printColumns'
import {
  usePrintSettings,
  FONT_FAMILY_OPTIONS,
  FONT_SIZE_OPTIONS,
  PRINT_STYLE_OPTIONS,
} from '../composables/usePrintSettings'

const props = defineProps<{
  modelValue: boolean
  mode: 'print' | 'pdf'
  pageKey: string
  columns: PrintColumn[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  confirm: []
}>()

const { settings, getEnabledFields, setEnabledFields, applyPrintSettings, resetPrintSettings } = usePrintSettings()
const STRIPE_COLOR_OPTIONS = ['#eef6ff', '#eef9f0', '#f3f4f6', '#fff7e6', '#f6efff']

interface DraftSettings {
  enabledFields: string[]
  portrait: boolean
  fontFamily: string
  fontSize: number
  styleId: string
  striped: boolean
  stripeColor: string
}

function buildDraft(): DraftSettings {
  return {
    enabledFields: [...getEnabledFields(props.pageKey)],
    portrait: settings.portrait,
    fontFamily: settings.fontFamily,
    fontSize: settings.fontSize,
    styleId: settings.styleId,
    striped: settings.striped,
    stripeColor: settings.stripeColor,
  }
}

const draft = reactive<DraftSettings>(buildDraft())

function resetDraft() {
  Object.assign(draft, buildDraft())
}

watch(() => props.modelValue, (opened) => {
  if (opened) resetDraft()
})

function selectAll() {
  draft.enabledFields = props.columns.map(column => column.field)
}

function clearAll() {
  draft.enabledFields = []
}

function resetAll() {
  resetPrintSettings()
  resetDraft()
}

function confirmSettings() {
  setEnabledFields(props.pageKey, [...draft.enabledFields])
  applyPrintSettings({
    portrait: draft.portrait,
    fontFamily: draft.fontFamily,
    fontSize: draft.fontSize,
    styleId: draft.styleId,
    striped: draft.striped,
    stripeColor: draft.stripeColor,
  })
  emit('update:modelValue', false)
  emit('confirm')
}
</script>

<style scoped>
.print-settings-dialog :deep(.el-dialog__body) {
  padding-top: 14px;
}

.settings-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 240px;
  gap: 18px;
}

.settings-section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
}

.settings-actions {
  display: flex;
  gap: 4px;
}

.settings-column-list {
  max-height: 340px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.settings-column-list :deep(.el-checkbox) {
  height: auto;
  margin-right: 0;
  padding: 4px 6px;
  border-radius: 6px;
  transition: background-color 140ms ease;
}
.settings-column-list :deep(.el-checkbox:hover) {
  background: var(--surface-hover);
}

.column-label {
  color: var(--text-primary);
  font-size: 13px;
}

.column-short {
  margin-left: 6px;
  color: var(--text-muted);
  font-size: 11px;
}

.settings-options {
  padding-left: 16px;
  border-left: 1px solid var(--border);
}

.option-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 14px;
}

.option-label {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
}

.option-hint {
  color: var(--text-muted);
  font-size: 11px;
}

.stripe-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.stripe-preview {
  width: 82px;
  overflow: hidden;
  display: grid;
  grid-template-rows: repeat(3, 7px);
  border: 1px solid var(--border);
  border-radius: 4px;
  background: #fff;
}

.stripe-preview i {
  display: block;
  background: #fff;
}

.stripe-preview.is-disabled {
  opacity: 0.48;
}

.style-picker {
  display: flex;
  gap: 8px;
}

.style-item {
  flex: 1;
  padding: 6px 4px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--card-bg);
  cursor: pointer;
  transition: border-color 140ms ease, box-shadow 140ms ease;
}
.style-item:hover {
  border-color: var(--primary);
}
.style-item.is-active {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent);
}

.style-swatch {
  width: 100%;
  height: 26px;
  border-radius: 5px;
  border: 1px solid rgba(0, 0, 0, 0.08);
}

.style-name {
  color: var(--text-secondary);
  font-size: 11px;
}

.dialog-footer {
  display: flex;
  align-items: center;
  gap: 8px;
}

.footer-spacer {
  flex: 1;
}
</style>
