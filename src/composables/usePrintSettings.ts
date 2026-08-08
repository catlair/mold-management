/**
 * 打印设置：按业务页面独立保存。
 * - 持久化到 localStorage（key: mold-management.print-preferences.v1）
 * - 螺丝规格、冲头、牙板等页面的列、方向、字体、字号、风格和隔行色互不覆盖
 * - 兼容旧版的全局版式设置和按页面列设置，首次读取时自动迁移
 */
import { reactive } from 'vue'
import { PRINT_COLUMN_SETS, screwSpecPrintColumnFields } from '../config/printColumns'

export type PrintGroupMode = 'none' | 'customer'

export interface PrintPageSettings {
  enabledFields: string[]
  portrait: boolean
  fontFamily: string
  fontSize: number
  styleId: string
  striped: boolean
  stripeColor: string
  /** 分组方式：none=普通分页；customer=按客户分组（同客户尽量同页） */
  groupMode: PrintGroupMode
}

export interface PrintSettings {
  byPage: Record<string, PrintPageSettings>
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

export const DEFAULT_PAGE_PRINT_SETTINGS: PrintPageSettings = {
  enabledFields: [],
  portrait: true,
  fontFamily: '微软雅黑',
  fontSize: 9,
  styleId: 'blue',
  striped: false,
  stripeColor: '#eef6ff',
  groupMode: 'none',
}

/** 默认启用按客户分组的页面（打印设置里可随时切回普通分页） */
const DEFAULT_GROUP_MODE: Record<string, PrintGroupMode> = {
  screwSpec: 'customer',
}

const STORAGE_KEY = 'mold-management.print-preferences.v1'

type LegacyStorage = Partial<PrintPageSettings> & {
  enabledFieldsByPage?: unknown
  byPage?: unknown
  enabledFields?: unknown
}

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

function clonePageDefaults(pageKey: string, enabledFields?: string[]): PrintPageSettings {
  const defaults = DEFAULT_ENABLED_FIELDS[pageKey] ?? []
  const validFields = enabledFields?.filter(field => defaults.includes(field))
  return {
    ...DEFAULT_PAGE_PRINT_SETTINGS,
    enabledFields: validFields?.length ? [...validFields] : [...defaults],
    groupMode: DEFAULT_GROUP_MODE[pageKey] ?? 'none',
  }
}

function cloneDefaultSettings(): PrintSettings {
  return {
    byPage: Object.fromEntries(
      Object.keys(DEFAULT_ENABLED_FIELDS).map(pageKey => [pageKey, clonePageDefaults(pageKey)]),
    ),
  }
}

function normalizePageSettings(pageKey: string, value: unknown, fallback?: Partial<PrintPageSettings>): PrintPageSettings {
  const source = value && typeof value === 'object' ? value as Partial<PrintPageSettings> : {}
  const fallbackFields = fallback?.enabledFields
  const fields = Array.isArray(source.enabledFields) ? source.enabledFields : fallbackFields
  const result = clonePageDefaults(pageKey, Array.isArray(fields) ? fields : undefined)
  result.portrait = typeof source.portrait === 'boolean' ? source.portrait : fallback?.portrait ?? result.portrait
  result.fontFamily = isValidFontFamily(source.fontFamily) ? source.fontFamily : fallback?.fontFamily ?? result.fontFamily
  result.fontSize = isValidFontSize(source.fontSize) ? source.fontSize : fallback?.fontSize ?? result.fontSize
  result.styleId = isValidStyleId(source.styleId) ? source.styleId : fallback?.styleId ?? result.styleId
  result.striped = typeof source.striped === 'boolean' ? source.striped : fallback?.striped ?? result.striped
  result.stripeColor = isValidColor(source.stripeColor) ? source.stripeColor : fallback?.stripeColor ?? result.stripeColor
  result.groupMode = source.groupMode === 'none' || source.groupMode === 'customer'
    ? source.groupMode
    : fallback?.groupMode ?? result.groupMode
  return result
}

function normalizeEnabledFieldsByPage(value: unknown): Record<string, string[]> {
  const source = value && typeof value === 'object' ? value as Record<string, unknown> : {}
  return Object.fromEntries(
    Object.keys(DEFAULT_ENABLED_FIELDS).map(pageKey => {
      const fields = source[pageKey]
      return [pageKey, clonePageDefaults(pageKey, Array.isArray(fields) ? fields.filter((field): field is string => typeof field === 'string') : undefined).enabledFields]
    }),
  )
}

function loadSettings(): PrintSettings {
  const defaults = cloneDefaultSettings()
  if (typeof window === 'undefined') return defaults
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaults
    const parsed = JSON.parse(raw) as LegacyStorage

    // 新版结构：每个页面拥有完整的设置对象。
    if (parsed.byPage && typeof parsed.byPage === 'object') {
      const source = parsed.byPage as Record<string, unknown>
      for (const pageKey of Object.keys(DEFAULT_ENABLED_FIELDS)) {
        defaults.byPage[pageKey] = normalizePageSettings(pageKey, source[pageKey])
      }
      return defaults
    }

    // 兼容当前旧版：列已按页面保存，但版式仍是全局设置。
    const enabledFieldsByPage = parsed.enabledFieldsByPage
      ? normalizeEnabledFieldsByPage(parsed.enabledFieldsByPage)
      : (() => {
          const legacyFields = Array.isArray(parsed.enabledFields)
            ? parsed.enabledFields.filter((field): field is string => typeof field === 'string' && screwSpecPrintColumnFields.includes(field))
            : undefined
          return Object.fromEntries(
            Object.keys(DEFAULT_ENABLED_FIELDS).map(pageKey => [pageKey, pageKey === 'screwSpec' && legacyFields?.length ? legacyFields : DEFAULT_ENABLED_FIELDS[pageKey]]),
          )
        })()
    const legacyLayout: Partial<PrintPageSettings> = {
      portrait: parsed.portrait,
      fontFamily: parsed.fontFamily,
      fontSize: parsed.fontSize,
      styleId: parsed.styleId,
      striped: parsed.striped,
      stripeColor: parsed.stripeColor,
    }
    for (const pageKey of Object.keys(DEFAULT_ENABLED_FIELDS)) {
      defaults.byPage[pageKey] = normalizePageSettings(pageKey, { enabledFields: enabledFieldsByPage[pageKey] }, legacyLayout)
    }
    return defaults
  } catch {
    return cloneDefaultSettings()
  }
}

const settings = reactive<PrintSettings>(loadSettings())

function persist() {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}

function getPageSettings(pageKey: string): PrintPageSettings {
  if (!settings.byPage[pageKey]) {
    settings.byPage[pageKey] = clonePageDefaults(pageKey)
  }
  return settings.byPage[pageKey]
}

function setEnabledFields(pageKey: string, fields: string[]) {
  const pageSettings = getPageSettings(pageKey)
  const validFields = fields.filter(field => (DEFAULT_ENABLED_FIELDS[pageKey] ?? []).includes(field))
  pageSettings.enabledFields = validFields.length ? [...validFields] : [...(DEFAULT_ENABLED_FIELDS[pageKey] ?? [])]
  persist()
}

function applyPrintSettings(pageKey: string, patch: Partial<PrintPageSettings>) {
  const pageSettings = getPageSettings(pageKey)
  Object.assign(pageSettings, patch)
  persist()
}

function resetPrintSettings(pageKey: string) {
  const pageSettings = getPageSettings(pageKey)
  Object.assign(pageSettings, clonePageDefaults(pageKey))
  persist()
}

export function usePrintSettings() {
  return { settings, getPageSettings, setEnabledFields, applyPrintSettings, resetPrintSettings }
}
