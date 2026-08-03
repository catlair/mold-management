import { allowDeleteApi, backupApi, dieMachineTypeApi, punchSpecApi } from '../api'
import { useTheme, type ThemeMode } from '../composables/useTheme'
import {
  applyTablePreferencesImport,
  createTablePreferencesExport,
  parseTablePreferencesImport,
  type TablePreferenceState,
  type TablePreferencesImportResult,
} from '../composables/useTablePreferences'

export const PORTABLE_CONFIGURATION_KIND = 'mold-management-portable-configuration'
export const PORTABLE_CONFIGURATION_VERSION = 1

export interface PortableConfigurationFile {
  kind: typeof PORTABLE_CONFIGURATION_KIND
  version: typeof PORTABLE_CONFIGURATION_VERSION
  exportedAt: string
  configurations: Record<string, unknown>
}

export interface PortableConfigurationPreviewItem {
  id: string
  label: string
  summary: string
}

export interface PortableConfigurationImportPlan {
  values: Record<string, unknown>
  items: PortableConfigurationPreviewItem[]
}

export interface PortableConfigurationDefinition<T> {
  id: string
  label: string
  description: string
  editor: 'option-list' | 'boolean' | 'number' | 'theme' | 'table-preferences'
  defaultValue?: T
  load: () => Promise<T>
  normalize: (value: unknown) => T
  save: (value: T) => Promise<void>
  summarize: (value: T) => string
}

function normalizeBoolean(value: unknown, label: string): boolean {
  if (typeof value !== 'boolean') throw new Error(`${label}必须是布尔值`)
  return value
}

function normalizeBackupCount(value: unknown): number {
  const count = Number(value)
  if (!Number.isInteger(count) || count < 1 || count > 100) {
    throw new Error('备份保留份数必须是 1 至 100 之间的整数')
  }
  return count
}

function normalizeTheme(value: unknown): ThemeMode {
  if (value !== 'light' && value !== 'dark' && value !== 'system') {
    throw new Error('主题模式必须是浅色、深色或跟随系统')
  }
  return value
}

function normalizeOptionList(value: unknown, label: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${label}配置必须是数组`)
  const normalized: string[] = []
  for (const item of value) {
    if (typeof item !== 'string') throw new Error(`${label}配置只能包含文本选项`)
    const text = item.trim()
    if (!text) continue
    if (text.length > 40) throw new Error(`${label}选项「${text}」不能超过 40 个字符`)
    if (!normalized.some(existing => existing.toLocaleLowerCase() === text.toLocaleLowerCase())) {
      normalized.push(text)
    }
  }
  if (!normalized.length) throw new Error(`${label}至少保留一个选项`)
  if (normalized.length > 100) throw new Error(`${label}最多允许 100 个选项`)
  return normalized
}

function normalizeTablePreferences(value: unknown): TablePreferencesImportResult {
  return parseTablePreferencesImport(JSON.stringify({
    kind: 'mold-management-table-preferences',
    version: 1,
    preferences: value,
  }))
}

const portableConfigurationRegistry: PortableConfigurationDefinition<any>[] = [
  {
    id: 'system.allow-delete',
    label: '允许删除',
    description: '控制各管理页面是否显示删除按钮。',
    editor: 'boolean',
    defaultValue: false,
    load: () => allowDeleteApi.get(),
    normalize: value => normalizeBoolean(value, '允许删除'),
    save: async value => { await allowDeleteApi.set(value) },
    summarize: value => value ? '已开启' : '已关闭',
  },
  {
    id: 'backup.retention-count',
    label: '备份保留份数',
    description: '超过该数量时自动清理最早且未锁定的备份。备份目录不会随配置迁移。',
    editor: 'number',
    defaultValue: 10,
    load: async () => (await backupApi.getConfig()).backupCount || 10,
    normalize: normalizeBackupCount,
    save: async value => {
      const config = await backupApi.getConfig()
      await backupApi.setConfig(value, config.backupPath || null)
    },
    summarize: value => `${value} 份`,
  },
  {
    id: 'appearance.theme',
    label: '界面主题',
    description: '应用的浅色、深色或跟随系统主题。',
    editor: 'theme',
    defaultValue: 'system',
    load: async () => useTheme().themeMode.value,
    normalize: normalizeTheme,
    save: async value => { useTheme().setTheme(value) },
    summarize: value => ({ light: '浅色', dark: '深色', system: '跟随系统' })[value as ThemeMode],
  },
  {
    id: 'options.die-machine-types',
    label: '牙板机型列表',
    description: '用于牙板新增、编辑表单的候选项，表单仍允许临时输入其他内容。',
    editor: 'option-list',
    defaultValue: ['003', '3/16', '1/4', '6R'],
    load: () => dieMachineTypeApi.get(),
    normalize: value => normalizeOptionList(value, '牙板机型列表'),
    save: async value => { await dieMachineTypeApi.set(value) },
    summarize: value => `${value.length} 个选项`,
  },
  {
    id: 'options.punch-specs',
    label: '冲头规格列表',
    description: '用于冲头新增、编辑表单的候选项，表单仍允许临时输入其他内容。',
    editor: 'option-list',
    defaultValue: ['12*15', '14*15', '18*18'],
    load: () => punchSpecApi.get(),
    normalize: value => normalizeOptionList(value, '冲头规格列表'),
    save: async value => { await punchSpecApi.set(value) },
    summarize: value => `${value.length} 个选项`,
  },
  {
    id: 'tables.preferences',
    label: '页面表格配置',
    description: '统一管理各业务页面表格的显示列、排序、筛选和列宽。',
    editor: 'table-preferences',
    load: async () => createTablePreferencesExport().preferences,
    normalize: normalizeTablePreferences,
    save: async value => { applyTablePreferencesImport(value) },
    summarize: value => `${value.tableCount} 个表格、${value.columnCount} 个列配置`,
  },
]

const registryMap = new Map(portableConfigurationRegistry.map(definition => [definition.id, definition]))

export async function createPortableConfigurationExport(): Promise<PortableConfigurationFile> {
  const configurations: Record<string, unknown> = {}
  for (const definition of portableConfigurationRegistry) {
    const value = await definition.load()
    configurations[definition.id] = definition.id === 'tables.preferences'
      ? value
      : definition.normalize(value)
  }
  return {
    kind: PORTABLE_CONFIGURATION_KIND,
    version: PORTABLE_CONFIGURATION_VERSION,
    exportedAt: new Date().toISOString(),
    configurations,
  }
}

export function parsePortableConfigurationImport(raw: string): PortableConfigurationImportPlan {
  let document: unknown
  try {
    document = JSON.parse(raw)
  } catch {
    throw new Error('文件不是有效的 JSON 配置文件')
  }
  if (!document || typeof document !== 'object' || Array.isArray(document)) {
    throw new Error('配置文件格式无效')
  }

  const record = document as Record<string, unknown>
  if (record.kind !== PORTABLE_CONFIGURATION_KIND) {
    throw new Error('文件类型不正确，请选择由本系统导出的统一配置文件')
  }
  if (record.version !== PORTABLE_CONFIGURATION_VERSION) {
    throw new Error(`不支持的配置版本：${String(record.version)}`)
  }
  if (!record.configurations || typeof record.configurations !== 'object' || Array.isArray(record.configurations)) {
    throw new Error('配置文件缺少 configurations 数据')
  }
  const configurations = record.configurations as Record<string, unknown>

  const fileIds = Object.keys(configurations)
  const unknownIds = fileIds.filter(id => !registryMap.has(id))
  if (unknownIds.length) {
    throw new Error(`配置文件包含当前版本不支持的配置：${unknownIds.join('、')}`)
  }
  const missingDefinitions = portableConfigurationRegistry.filter(definition => !(definition.id in configurations))
  if (missingDefinitions.length) {
    throw new Error(`配置文件不完整，缺少：${missingDefinitions.map(definition => definition.label).join('、')}`)
  }

  const values: Record<string, unknown> = {}
  const items: PortableConfigurationPreviewItem[] = []
  for (const definition of portableConfigurationRegistry) {
    const normalized = definition.normalize(configurations[definition.id])
    values[definition.id] = normalized
    items.push({ id: definition.id, label: definition.label, summary: definition.summarize(normalized) })
  }
  return { values, items }
}

export async function applyPortableConfigurationImport(plan: PortableConfigurationImportPlan): Promise<void> {
  const definitions = plan.items.map(item => registryMap.get(item.id)!).filter(Boolean)
  const previousValues = new Map<string, unknown>()
  const applied: PortableConfigurationDefinition<any>[] = []

  for (const definition of definitions) {
    previousValues.set(definition.id, await definition.load())
  }

  try {
    for (const definition of definitions) {
      await definition.save(plan.values[definition.id])
      applied.push(definition)
    }
  } catch (error) {
    const rollbackErrors: string[] = []
    for (const definition of applied.reverse()) {
      try {
        const previous = previousValues.get(definition.id)
        const rollbackValue = definition.id === 'tables.preferences'
          ? normalizeTablePreferences(previous as TablePreferenceState)
          : previous
        await definition.save(rollbackValue)
      } catch (rollbackError) {
        rollbackErrors.push(`${definition.label}：${String(rollbackError)}`)
      }
    }
    if (rollbackErrors.length) {
      throw new Error(`${String(error)}；部分配置回滚失败：${rollbackErrors.join('；')}`)
    }
    throw error
  }
}

export function getPortableConfigurationRegistry() {
  return portableConfigurationRegistry
}
