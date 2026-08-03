import { computed, reactive } from 'vue'
import { tableDefinitionMap, type TableColumnDefinition } from '../config/tableCatalog'

export interface ColumnPreference {
  visible: boolean
  sortable: boolean
  filterable: boolean
  width?: number
}

export type TablePreferenceState = Record<string, Record<string, ColumnPreference>>

const STORAGE_KEY = 'mold-management.table-preferences.v1'

function defaultPreference(column: TableColumnDefinition): ColumnPreference {
  return {
    visible: column.defaultVisible !== false,
    sortable: column.canSort === false ? false : column.defaultSortable === true,
    filterable: column.canFilter === false ? false : column.defaultFilterable === true,
  }
}

function loadPreferences(): TablePreferenceState {
  if (typeof window === 'undefined') return {}
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    return raw ? JSON.parse(raw) as TablePreferenceState : {}
  } catch {
    return {}
  }
}

const preferences = reactive<TablePreferenceState>(loadPreferences())

function persist() {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences))
}

function getColumnPreference(tableId: string, columnId: string): ColumnPreference {
  const definition = tableDefinitionMap.get(tableId)?.columns.find(item => item.id === columnId)
  const fallback: ColumnPreference = definition
    ? defaultPreference(definition)
    : { visible: true, sortable: false, filterable: false }
  const stored = preferences[tableId]?.[columnId]
  return stored ? { ...fallback, ...stored } : fallback
}

function ensureTablePreference(tableId: string) {
  const table = tableDefinitionMap.get(tableId)
  if (!table) return
  if (!preferences[tableId]) preferences[tableId] = {}
  for (const column of table.columns) {
    if (!preferences[tableId][column.id]) {
      preferences[tableId][column.id] = defaultPreference(column)
    }
  }
}

function setColumnPreference(tableId: string, columnId: string, patch: Partial<ColumnPreference>) {
  ensureTablePreference(tableId)
  const table = tableDefinitionMap.get(tableId)
  const definition = table?.columns.find(item => item.id === columnId)
  const next = { ...getColumnPreference(tableId, columnId), ...patch }
  if (definition?.canSort === false) next.sortable = false
  if (definition?.canFilter === false) next.filterable = false
  preferences[tableId][columnId] = next
  persist()
}

function resetTablePreference(tableId: string) {
  delete preferences[tableId]
  persist()
}

function resetAllPreferences() {
  for (const tableId of Object.keys(preferences)) delete preferences[tableId]
  persist()
}

export function useTablePreferences(tableId?: string) {
  const currentTablePreferences = computed(() => {
    if (!tableId) return {}
    const table = tableDefinitionMap.get(tableId)
    if (!table) return {}
    return Object.fromEntries(table.columns.map(column => [column.id, getColumnPreference(tableId, column.id)]))
  })

  return {
    preferences,
    currentTablePreferences,
    getColumnPreference,
    setColumnPreference,
    resetTablePreference,
    resetAllPreferences,
  }
}
