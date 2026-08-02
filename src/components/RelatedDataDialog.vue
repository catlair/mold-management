<template>
  <el-dialog
    :model-value="modelValue"
    :width="width"
    class="related-data-dialog"
    align-center
    append-to-body
    destroy-on-close
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #header>
      <div class="related-dialog-header">
        <div class="related-dialog-icon" aria-hidden="true">
          <el-icon><Connection /></el-icon>
        </div>
        <div class="related-dialog-heading">
          <h2>{{ title }}</h2>
          <p v-if="description">{{ description }}</p>
        </div>
      </div>
    </template>

    <div class="related-dialog-body">
      <div class="related-dialog-surface">
        <slot />
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { Connection } from '@element-plus/icons-vue'

withDefaults(defineProps<{
  modelValue: boolean
  title: string
  description?: string
  width?: string
}>(), {
  description: '',
  width: 'min(860px, calc(100vw - 48px))',
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<style>
.related-data-dialog.el-dialog {
  max-width: calc(100vw - 32px);
  margin: 16px auto;
  padding: 0;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card-bg);
  box-shadow: 0 24px 64px rgba(15, 23, 42, 0.2), 0 6px 18px rgba(15, 23, 42, 0.08);
}

.dark .related-data-dialog.el-dialog {
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.48), 0 6px 18px rgba(0, 0, 0, 0.24);
}

.related-data-dialog .el-dialog__header {
  margin: 0;
  padding: 22px 24px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--card-bg);
}

.related-data-dialog .el-dialog__headerbtn {
  top: 18px;
  right: 18px;
  width: 34px;
  height: 34px;
  border-radius: 9px;
  transition: background-color 0.18s ease, color 0.18s ease, transform 0.18s ease;
}

.related-data-dialog .el-dialog__headerbtn:hover {
  background: var(--surface-hover);
}

.related-data-dialog .el-dialog__headerbtn:active {
  transform: scale(0.96);
}

.related-data-dialog .el-dialog__body {
  padding: 0;
  background: var(--surface-muted);
}

.related-dialog-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-right: 42px;
}

.related-dialog-icon {
  width: 38px;
  height: 38px;
  flex: 0 0 38px;
  display: grid;
  place-items: center;
  color: var(--primary);
  background: var(--sidebar-active);
  border: 1px solid color-mix(in srgb, var(--primary) 20%, transparent);
  border-radius: 11px;
  font-size: 18px;
}

.related-dialog-heading {
  min-width: 0;
}

.related-dialog-heading h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 18px;
  line-height: 1.35;
  font-weight: 650;
  letter-spacing: 0.01em;
}

.related-dialog-heading p {
  margin: 4px 0 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.related-dialog-body {
  padding: 22px 24px 24px;
}

.related-dialog-surface {
  min-width: 0;
  padding: 12px;
  overflow: auto;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
}

.related-dialog-surface .vxe-table {
  border-radius: 8px;
}

.related-dialog-surface .vxe-table--render-default {
  min-width: max-content;
}

.related-dialog-surface::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

.related-dialog-surface::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
  border-radius: 5px;
}

.related-dialog-surface::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border: 2px solid var(--scrollbar-track);
  border-radius: 5px;
}

.related-dialog-empty {
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}

@media (max-width: 720px) {
  .related-data-dialog .el-dialog__header {
    padding: 18px 18px 15px;
  }

  .related-dialog-body {
    padding: 16px 18px 18px;
  }

  .related-dialog-surface {
    padding: 10px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .related-data-dialog .el-dialog__headerbtn {
    transition: none;
  }
}
</style>
