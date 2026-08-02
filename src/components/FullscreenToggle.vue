<template>
  <el-button @click="toggleFullscreen">
    <el-icon><FullScreen v-if="!isFullscreen" /><Close v-else /></el-icon>
    {{ isFullscreen ? '退出全屏' : '全屏' }}
  </el-button>
</template>

<script setup lang="ts">
import { inject } from 'vue'
import type { Ref } from 'vue'
import { FullScreen, Close } from '@element-plus/icons-vue'

interface FullscreenContext {
  isFullscreen: Ref<boolean>
  toggleFullscreen: () => void
}

// 全屏能力由 App 布局层统一提供（useFullscreen），此处仅消费
const { isFullscreen, toggleFullscreen } = inject<FullscreenContext>('fullscreen')!
</script>
