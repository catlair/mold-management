<template>
  <el-dropdown trigger="click" placement="top-start" @command="handleCommand">
    <button
      class="theme-toggle"
      type="button"
      :aria-label="`当前主题：${currentLabel}，点击切换`"
      :title="`主题：${currentLabel}`"
    >
      <el-icon><component :is="currentIcon" /></el-icon>
      <span v-if="!collapsed" class="theme-label">{{ currentLabel }}</span>
      <el-icon v-if="!collapsed" class="theme-arrow"><ArrowUp /></el-icon>
    </button>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="light" :class="{ 'is-theme-active': themeMode === 'light' }">
          <el-icon><Sunny /></el-icon>浅色
        </el-dropdown-item>
        <el-dropdown-item command="dark" :class="{ 'is-theme-active': themeMode === 'dark' }">
          <el-icon><Moon /></el-icon>深色
        </el-dropdown-item>
        <el-dropdown-item command="system" :class="{ 'is-theme-active': themeMode === 'system' }">
          <el-icon><Monitor /></el-icon>跟随系统
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ArrowUp, Monitor, Moon, Sunny } from '@element-plus/icons-vue'
import { useTheme, type ThemeMode } from '../composables/useTheme'

withDefaults(defineProps<{
  collapsed?: boolean
}>(), {
  collapsed: false,
})

const { themeMode, setTheme } = useTheme()

const currentLabel = computed(() => ({
  light: '浅色',
  dark: '深色',
  system: '跟随系统',
})[themeMode.value])

const currentIcon = computed(() => ({
  light: Sunny,
  dark: Moon,
  system: Monitor,
})[themeMode.value])

function handleCommand(command: ThemeMode) {
  setTheme(command)
}
</script>

<style scoped>
.theme-toggle {
  width: calc(100% - 16px);
  min-height: 38px;
  margin: 0 8px 8px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  color: var(--sidebar-text);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease, transform 0.2s ease;
}

.theme-toggle:hover {
  color: var(--primary);
  background: var(--sidebar-hover);
  border-color: var(--border);
}

.theme-toggle:active {
  transform: scale(0.98);
}

.theme-toggle:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.theme-label {
  flex: 1;
  text-align: left;
  font-size: 13px;
  font-weight: 500;
}

.theme-arrow {
  font-size: 12px;
  opacity: 0.6;
}

:global(.el-dropdown-menu__item.is-theme-active) {
  color: var(--el-color-primary);
  font-weight: 600;
  background: var(--sidebar-active);
}

@media (prefers-reduced-motion: reduce) {
  .theme-toggle {
    transition: none;
  }
}
</style>
