import { computed, reactive } from 'vue'
import { tableDefinitionMap, type TableColumnDefinition } from '../config/tableCatalog'

export interface ColumnPreference {
  visible: boolean
  sortable: boolean
  filterable: boolean
  width?: number
}

export type TablePreferenceState = Record<string, Record<string, ColumnPreference>>

export interface TablePreferencesExportFile {
  kind: 'mold-management-table-preferences'
  version: 1
  exportedAt: string
  preferences: TablePreferenceState
}

export interface TablePreferencesImportResult {
  preferences: TablePreferenceState
  tableCount: number
  columnCount: number
  ignoredTableCount: number
  ignoredColumnCount: number
}

interface ResizableColumnLike {
  field?: unknown
  property?: unknown
  title?: unknown
  resizeWidth?: unknown
  renderWidth?: unknown
}

export interface ColumnResizeEventParams {
  column?: ResizableColumnLike
  resizeColumn?: ResizableColumnLike
  resizeWidth?: unknown
}

export interface ResolvedColumnResize {
  columnId: string
  width: number
}

interface VxeCustomStoreData {
  resizableData?: Record<string, number>
}

interface VxeCustomStoreParams {
  id: string
  storeData?: VxeCustomStoreData
}

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

function replacePreferences(next: TablePreferenceState) {
  if (typeof window === 'undefined') return
  const serialized = JSON.stringify(next)
  window.localStorage.setItem(STORAGE_KEY, serialized)
  for (const tableId of Object.keys(preferences)) delete preferences[tableId]
  for (const [tableId, columns] of Object.entries(next)) preferences[tableId] = columns
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}

function validateColumnPreference(value: unknown, tableId: string, columnId: string): ColumnPreference {
  if (!isRecord(value)) throw new Error(`表格 ${tableId} 的列 ${columnId} 配置格式无效`)
  for (const key of ['visible', 'sortable', 'filterable'] as const) {
    if (typeof value[key] !== 'boolean') throw new Error(`表格 ${tableId} 的列 ${columnId} 缺少有效的 ${key} 配置`)
  }
  if (value.width != null && (!Number.isFinite(value.width) || Number(value.width) < 60 || Number(value.width) > 4000)) {
    throw new Error(`表格 ${tableId} 的列 ${columnId} 宽度必须在 60 至 4000 之间`)
  }
  return {
    visible: value.visible as boolean,
    sortable: value.sortable as boolean,
    filterable: value.filterable as boolean,
    ...(value.width == null ? {} : { width: Math.round(Number(value.width)) }),
  }
}

export function createTablePreferencesExport(): TablePreferencesExportFile {
  const exportedPreferences: TablePreferenceState = {}
  for (const [tableId, table] of tableDefinitionMap) {
    exportedPreferences[tableId] = Object.fromEntries(
      table.columns.map(column => [column.id, getColumnPreference(tableId, column.id)]),
    )
  }
  return {
    kind: 'mold-management-table-preferences',
    version: 1,
    exportedAt: new Date().toISOString(),
    preferences: exportedPreferences,
  }
}

export function parseTablePreferencesImport(raw: string): TablePreferencesImportResult {
  let document: unknown
  try {
    document = JSON.parse(raw)
  } catch {
    throw new Error('文件不是有效的 JSON 配置文件')
  }
  if (!isRecord(document) || document.kind !== 'mold-management-table-preferences') {
    throw new Error('文件类型不正确，请选择由本系统导出的表格配置文件')
  }
  if (document.version !== 1) throw new Error(`不支持的配置版本：${String(document.version)}`)
  if (!isRecord(document.preferences)) throw new Error('配置文件缺少 preferences 数据')

  const imported: TablePreferenceState = {}
  let columnCount = 0
  let ignoredTableCount = 0
  let ignoredColumnCount = 0

  for (const [tableId, rawColumns] of Object.entries(document.preferences)) {
    const table = tableDefinitionMap.get(tableId)
    if (!table) {
      ignoredTableCount += 1
      continue
    }
    if (!isRecord(rawColumns)) throw new Error(`表格 ${tableId} 的配置格式无效`)
    const knownColumns = new Map(table.columns.map(column => [column.id, column]))
    const importedColumns: Record<string, ColumnPreference> = {}
    for (const [columnId, rawPreference] of Object.entries(rawColumns)) {
      const definition = knownColumns.get(columnId)
      if (!definition) {
        ignoredColumnCount += 1
        continue
      }
      const preference = validateColumnPreference(rawPreference, tableId, columnId)
      if (definition.canSort === false) preference.sortable = false
      if (definition.canFilter === false) preference.filterable = false
      importedColumns[columnId] = preference
      columnCount += 1
    }
    imported[tableId] = importedColumns
  }

  if (!Object.keys(imported).length || !columnCount) throw new Error('配置文件中没有当前版本可识别的表格列配置')
  return {
    preferences: imported,
    tableCount: Object.keys(imported).length,
    columnCount,
    ignoredTableCount,
    ignoredColumnCount,
  }
}

export function applyTablePreferencesImport(result: TablePreferencesImportResult) {
  replacePreferences(result.preferences)
}

export function resolveColumnResize(params: ColumnResizeEventParams): ResolvedColumnResize | null {
  const resizedColumn = params.resizeColumn || params.column
  const columnId = String(
    resizedColumn?.field ||
    resizedColumn?.property ||
    resizedColumn?.title ||
    '',
  )
  const width = Number(
    params.resizeWidth ??
    resizedColumn?.resizeWidth ??
    params.column?.resizeWidth ??
    resizedColumn?.renderWidth ??
    params.column?.renderWidth,
  )

  if (!columnId || columnId === '操作' || !Number.isFinite(width)) return null

  return {
    columnId,
    width: Math.max(60, Math.round(width)),
  }
}

export function createVxeCustomConfig() {
  return {
    enabled: true,
    storage: { resizable: true },
    restoreStore({ id }: VxeCustomStoreParams): VxeCustomStoreData {
      const table = tableDefinitionMap.get(id)
      if (!table) return {}

      const resizableData = Object.fromEntries(
        table.columns.flatMap(column => {
          const width = getColumnPreference(id, column.id).width
          return Number.isFinite(width) ? [[column.id, width as number]] : []
        }),
      )
      return Object.keys(resizableData).length ? { resizableData } : {}
    },
    updateStore({ id, storeData }: VxeCustomStoreParams) {
      const table = tableDefinitionMap.get(id)
      if (table) {
        const widths = storeData?.resizableData || {}
        for (const column of table.columns) {
          const width = Number(widths[column.id])
          if (column.id !== '操作' && Number.isFinite(width)) {
            setColumnPreference(id, column.id, {
              width: Math.max(60, Math.round(width)),
            })
          }
        }
      }
      return Promise.resolve()
    },
  }
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
