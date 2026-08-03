<template>
  <component
    :is="component"
    v-if="visible"
    v-bind="forwardedProps"
    :field="resolvedField"
    :col-id="resolvedColumnId"
    :sortable="sortable"
    :width="resolvedWidth"
    :resizable="resizable"
    :filters="resolvedFilters"
    :filter-method="resolvedFilterMethod"
  >
    <template v-if="$slots.default" #default="slotProps">
      <slot v-bind="slotProps" />
    </template>
  </component>
</template>

<script setup lang="ts">
import { computed, inject, useAttrs, type Ref } from 'vue'
import { tableDefinitionMap } from '../config/tableCatalog'
import { useTablePreferences } from '../composables/useTablePreferences'

interface TablePreferenceContext {
  tableId: Ref<string | undefined>
  data: Ref<any[]>
}

const props = withDefaults(defineProps<{
  tableId?: string
  columnId?: string
  component?: string
  sortable?: boolean
  filters?: Array<{ label: string; value: unknown }>
  filterMethod?: (params: any) => boolean
  resizable?: boolean
}>(), {
  component: 'vxe-column',
  sortable: false,
  filters: () => [],
  filterMethod: undefined,
  resizable: undefined,
})

const attrs = useAttrs()
const context = inject<TablePreferenceContext | null>('tablePreferenceContext', null)
const resolvedTableId = computed(() => props.tableId || context?.tableId.value || '')
const resolvedColumnId = computed(() => props.columnId || String(attrs.field || attrs.title || ''))
const resolvedField = computed(() => String(attrs.field || resolvedColumnId.value))
const { getColumnPreference } = useTablePreferences()

const isConfigured = computed(() => Boolean(resolvedTableId.value && tableDefinitionExists(resolvedTableId.value)))
const preference = computed(() => getColumnPreference(resolvedTableId.value, resolvedColumnId.value))
const visible = computed(() => !isConfigured.value || preference.value.visible !== false)
const sortable = computed(() => isConfigured.value ? preference.value.sortable : props.sortable)
const filterable = computed(() => isConfigured.value ? Boolean(preference.value.filterable) : props.filters.length > 0)
const resolvedWidth = computed(() => preference.value.width ?? attrs.width)
const resizable = computed(() => {
  if (props.resizable !== undefined) return props.resizable
  return resolvedColumnId.value !== '操作'
})
const resolvedFilters = computed(() => {
  if (!filterable.value) return undefined
  if (props.filters.length) return props.filters
  const field = String(attrs.field || resolvedColumnId.value)
  const values = [...new Set((context?.data.value || []).map(row => row?.[field]).filter(value => value !== '' && value != null))]
  return values.map(value => ({ label: String(value), value }))
})
const resolvedFilterMethod = computed(() => filterable.value && resolvedFilters.value?.length
  ? (props.filterMethod || defaultFilterMethod)
  : undefined)
const forwardedProps = computed(() => attrs)

function tableDefinitionExists(tableId: string) {
  return tableDefinitionMap.has(tableId)
}

function defaultFilterMethod({ value, row, column }: any) {
  const field = column?.field || column?.property || resolvedColumnId.value
  return row?.[field] === value
}
</script>
