import { computed, ref } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'mold-management-theme'
const themeMode = ref<ThemeMode>('system')
const systemPrefersDark = ref(false)
let initialized = false
let mediaQuery: MediaQueryList | null = null

function isThemeMode(value: string | null): value is ThemeMode {
  return value === 'light' || value === 'dark' || value === 'system'
}

function syncDocumentTheme() {
  if (typeof document === 'undefined') return
  const dark = themeMode.value === 'dark' || (themeMode.value === 'system' && systemPrefersDark.value)
  document.documentElement.classList.toggle('dark', dark)
  document.documentElement.dataset.theme = dark ? 'dark' : 'light'
  document.documentElement.style.colorScheme = dark ? 'dark' : 'light'
}

function handleSystemThemeChange(event: MediaQueryListEvent) {
  systemPrefersDark.value = event.matches
  if (themeMode.value === 'system') syncDocumentTheme()
}

export function initializeTheme() {
  if (initialized || typeof window === 'undefined') return

  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  systemPrefersDark.value = mediaQuery.matches

  const saved = window.localStorage.getItem(STORAGE_KEY)
  themeMode.value = isThemeMode(saved) ? saved : 'system'
  syncDocumentTheme()

  mediaQuery.addEventListener('change', handleSystemThemeChange)
  initialized = true
}

export function useTheme() {
  initializeTheme()

  const isDark = computed(() =>
    themeMode.value === 'dark' || (themeMode.value === 'system' && systemPrefersDark.value)
  )

  function setTheme(mode: ThemeMode) {
    themeMode.value = mode
    if (typeof window !== 'undefined') window.localStorage.setItem(STORAGE_KEY, mode)
    syncDocumentTheme()
  }

  return {
    themeMode,
    isDark,
    setTheme,
  }
}
