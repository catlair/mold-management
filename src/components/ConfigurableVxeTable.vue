<template>
  <component
    :is="VxeTable"
    v-bind="$attrs"
    :id="tableId"
    :data="data"
    :row-config="rowConfig"
    :column-config="columnConfig"
    :custom-config="customConfig"
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
import {
  createVxeCustomConfig,
  resolveColumnResize,
  useTablePreferences,
  type ColumnResizeEventParams,
} from '../composables/useTablePreferences'

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
const customConfig = createVxeCustomConfig()
const { setColumnPreference } = useTablePreferences()

function handleColumnResizableChange(params: ColumnResizeEventParams) {
  if (!props.tableId) return
  const resized = resolveColumnResize(params)
  if (!resized) return

  setColumnPreference(props.tableId, resized.columnId, { width: resized.width })
  nextTick(() => window.dispatchEvent(new Event('resize')))
}

provide('tablePreferenceContext', {
  tableId: toRef(props, 'tableId'),
  data: toRef(props, 'data'),
})
</script>
