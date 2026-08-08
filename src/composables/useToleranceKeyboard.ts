import { reactive } from 'vue'

export interface ToleranceKeyboardSettings {
  enabled: boolean
  quickKeys: string[]
}

export const DEFAULT_TOLERANCE_QUICK_KEYS = [
  '+', '-', '±', '/', '~', '×', '=', 'W', 'Φ', '介', '介厚', '垫', '束', '割尾',
]

/** 新增的系统快捷键：自动补入旧配置，不覆盖用户已有顺序与自定义字符。 */
const REQUIRED_QUICK_KEYS = ['=', 'W']

const STORAGE_KEY = 'mold-management.tolerance-keyboard.v1'

function loadSettings(): ToleranceKeyboardSettings {
  const defaults: ToleranceKeyboardSettings = {
    enabled: true,
    quickKeys: [...DEFAULT_TOLERANCE_QUICK_KEYS],
  }
  if (typeof localStorage === 'undefined') return defaults
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaults
    const parsed = JSON.parse(raw) as Partial<ToleranceKeyboardSettings>
    const storedKeys = Array.isArray(parsed.quickKeys)
      ? parsed.quickKeys.map(String).map(value => value.trim()).filter(value => value && value !== '.')
      : defaults.quickKeys
    const quickKeys = [...new Set([...storedKeys, ...REQUIRED_QUICK_KEYS])].slice(0, 30)
    return {
      enabled: typeof parsed.enabled === 'boolean' ? parsed.enabled : defaults.enabled,
      quickKeys,
    }
  } catch {
    return defaults
  }
}

const settings = reactive<ToleranceKeyboardSettings>(loadSettings())

function persist() {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}

export function updateToleranceKeyboardSettings(patch: Partial<ToleranceKeyboardSettings>) {
  if (typeof patch.enabled === 'boolean') settings.enabled = patch.enabled
  if (Array.isArray(patch.quickKeys)) {
    settings.quickKeys = [...new Set(
      patch.quickKeys.map(String).map(value => value.trim()).filter(value => value && value !== '.'),
    )].slice(0, 30)
  }
  persist()
}

export function resetToleranceKeyboardSettings() {
  settings.enabled = true
  settings.quickKeys = [...DEFAULT_TOLERANCE_QUICK_KEYS]
  persist()
}

export function useToleranceKeyboard() {
  return { settings, updateToleranceKeyboardSettings, resetToleranceKeyboardSettings }
}
