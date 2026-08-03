<template>
  <component
    :is="VxeTable"
    v-bind="$attrs"
    :data="data"
    :row-config="rowConfig"
    :column-config="columnConfig"
    @column-resizable-change="handleColumnResizableChange"
  >
    <slot />
    <template v-if="$slots.empty" #empty>
      <slot name="empty" />
    </template>
  </component>
</template>

<script setup lang="ts">
import { computed, nextTick, provide, resolveComponent, toRef } from 'vue'
import { useTablePreferences } from '../composables/useTablePreferences'

const VxeTable = resolveComponent('vxe-table')

const props = withDefaults(defineProps<{
  tableId?: string
  data: any[]
  rowConfig?: Record<string, unknown>
  columnConfig?: Record<string, unknown>
}>(), {
  tableId: '',
})

const rowConfig = computed(() => ({
  ...(props.rowConfig || {}),
  isHover: true,
}))
const columnConfig = computed(() => ({
  resizable: true,
  ...(props.columnConfig || {}),
}))
const { setColumnPreference } = useTablePreferences()

function handleColumnResizableChange({ column, resizeWidth }: any) {
  const columnId = String(column?.field || column?.title || '')
  if (!props.tableId || !columnId || columnId === '操作' || !Number.isFinite(resizeWidth)) return
  setColumnPreference(props.tableId, columnId, { width: Math.max(60, Math.round(resizeWidth)) })
  nextTick(() => window.dispatchEvent(new Event('resize')))
}

provide('tablePreferenceContext', {
  tableId: toRef(props, 'tableId'),
  data: toRef(props, 'data'),
})
</script>
