/**
 * 打印设置：应用级全局配置，跨页面沿用。
 * - 持久化到 localStorage（key: mold-management.print-preferences.v1）
 * - 勾选列按页面（pageKey）分别保存，互不影响
 * - 弹窗编辑本地草稿，确认后 applyPrintSettings 提交并持久化
 */
import { reactive } from 'vue'
import { PRINT_COLUMN_SETS, screwSpecPrintColumnFields } from '../config/printColumns'

export interface PrintSettings {
  enabledFieldsByPage: Record<string, string[]>
  portrait: boolean
  fontFamily: string
  fontSize: number
  styleId: string
  striped: boolean
  stripeColor: string
}

export const FONT_FAMILY_OPTIONS = ['微软雅黑', '宋体', '黑体', '仿宋', '楷体'] as const
export const FONT_SIZE_OPTIONS = [8, 9, 10, 11, 12] as const

export interface PrintStyleOption {
  id: string
  name: string
  headerCss: string
  color: string
}

export const PRINT_STYLE_OPTIONS: PrintStyleOption[] = [
  { id: 'blue', name: '蓝渐变', headerCss: 'linear-gradient(180deg,#5aa7ff,#2b7de9)', color: '#2b7de9' },
  { id: 'green', name: '绿渐变', headerCss: 'linear-gradient(180deg,#7fc97e,#3da33e)', color: '#3da33e' },
  { id: 'gray', name: '灰白简约', headerCss: 'linear-gradient(180deg,#f8fafc,#e6eaef)', color: '#d0d5da' },
]

/** 每个打印页面的默认勾选字段（由页面注册表自动推导，默认全选） */
export const DEFAULT_ENABLED_FIELDS: Record<string, string[]> = Object.fromEntries(
  Object.entries(PRINT_COLUMN_SETS).map(([key, columns]) => [key, columns.map(column => column.field)]),
)

export const DEFAULT_PRINT_SETTINGS: PrintSettings = {
  enabledFieldsByPage: Object.fromEntries(
    Object.entries(DEFAULT_ENABLED_FIELDS).map(([key, fields]) => [key, [...fields]]),
  ),
  portrait: true,
  fontFamily: '微软雅黑',
  fontSize: 9,
  styleId: 'blue',
  striped: false,
  stripeColor: '#eef6ff',
}

const STORAGE_KEY = 'mold-management.print-preferences.v1'

function isValidFontFamily(value: unknown): value is string {
  return typeof value === 'string' && (FONT_FAMILY_OPTIONS as readonly string[]).includes(value)
}

function isValidFontSize(value: unknown): value is number {
  return typeof value === 'number' && (FONT_SIZE_OPTIONS as readonly number[]).includes(value)
}

function isValidStyleId(value: unknown): value is string {
  return typeof value === 'string' && PRINT_STYLE_OPTIONS.some(option => option.id === value)
}

function isValidColor(value: unknown): value is string {
  return typeof value === 'string' && /^#[0-9a-f]{6}$/i.test(value)
}

function isRegisteredPageKey(key: string): key is keyof typeof DEFAULT_ENABLED_FIELDS {
  return Object.prototype.hasOwnProperty.call(DEFAULT_ENABLED_FIELDS, key)
}

function normalizeEnabledFields(value: unknown): Record<string, string[]> {
  const result: Record<string, string[]> = {}
  const source = value && typeof value === 'object' ? value as Record<string, unknown> : {}
  for (const pageKey of Object.keys(DEFAULT_ENABLED_FIELDS)) {
    const fields = source[pageKey]
    const validFields = Array.isArray(fields)
      ? fields.filter((field): field is string => typeof field === 'string' && DEFAULT_ENABLED_FIELDS[pageKey].includes(field))
      : [...DEFAULT_ENABLED_FIELDS[pageKey]]
    result[pageKey] = validFields.length ? validFields : [...DEFAULT_ENABLED_FIELDS[pageKey]]
  }
  return result
}

function cloneDefaultSettings(): PrintSettings {
  return {
    ...DEFAULT_PRINT_SETTINGS,
    enabledFieldsByPage: Object.fromEntries(
      Object.entries(DEFAULT_ENABLED_FIELDS).map(([key, fields]) => [key, [...fields]]),
    ),
  }
}

function loadSettings(): PrintSettings {
  if (typeof window === 'undefined') return cloneDefaultSettings()
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return cloneDefaultSettings()
    const parsed = JSON.parse(raw) as Partial<PrintSettings> & { enabledFields?: unknown }
    let enabledFieldsByPage: Record<string, string[]>
    if (parsed.enabledFieldsByPage !== undefined) {
      enabledFieldsByPage = normalizeEnabledFields(parsed.enabledFieldsByPage)
    } else if (Array.isArray(parsed.enabledFields)) {
      // 旧版：单个字段数组（螺丝规格），迁移到 screwSpec，其他页面默认全选
      const legacy = parsed.enabledFields.filter((field): field is string =>
        typeof field === 'string' && screwSpecPrintColumnFields.includes(field))
      enabledFieldsByPage = normalizeEnabledFields({
        ...DEFAULT_ENABLED_FIELDS,
        screwSpec: legacy.length ? legacy : [...DEFAULT_ENABLED_FIELDS.screwSpec],
      })
    } else {
      enabledFieldsByPage = cloneDefaultSettings().enabledFieldsByPage
    }
    return {
      enabledFieldsByPage,
      portrait: parsed.portrait !== false,
      fontFamily: isValidFontFamily(parsed.fontFamily) ? parsed.fontFamily : DEFAULT_PRINT_SETTINGS.fontFamily,
      fontSize: isValidFontSize(parsed.fontSize) ? parsed.fontSize : DEFAULT_PRINT_SETTINGS.fontSize,
      styleId: isValidStyleId(parsed.styleId) ? parsed.styleId : DEFAULT_PRINT_SETTINGS.styleId,
      striped: parsed.striped === true,
      stripeColor: isValidColor(parsed.stripeColor) ? parsed.stripeColor : DEFAULT_PRINT_SETTINGS.stripeColor,
    }
  } catch {
    return structuredClone(DEFAULT_PRINT_SETTINGS)
  }
}

const settings = reactive<PrintSettings>(loadSettings())

function persist() {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}

function getEnabledFields(pageKey: string): string[] {
  if (!isRegisteredPageKey(pageKey)) return []
  return settings.enabledFieldsByPage[pageKey] ?? [...DEFAULT_ENABLED_FIELDS[pageKey]]
}

function setEnabledFields(pageKey: string, fields: string[]) {
  if (!isRegisteredPageKey(pageKey)) return
  const validFields = fields.filter(field => DEFAULT_ENABLED_FIELDS[pageKey].includes(field))
  settings.enabledFieldsByPage = {
    ...settings.enabledFieldsByPage,
    [pageKey]: validFields.length ? validFields : [...DEFAULT_ENABLED_FIELDS[pageKey]],
  }
  persist()
}

function applyPrintSettings(patch: Partial<PrintSettings>) {
  if (patch.enabledFieldsByPage !== undefined) {
    patch.enabledFieldsByPage = normalizeEnabledFields(patch.enabledFieldsByPage)
  }
  Object.assign(settings, patch)
  persist()
}

function resetPrintSettings() {
  Object.assign(settings, cloneDefaultSettings())
  persist()
}

export function usePrintSettings() {
  return { settings, getEnabledFields, setEnabledFields, applyPrintSettings, resetPrintSettings }
}
