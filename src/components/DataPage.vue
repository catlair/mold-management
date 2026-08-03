<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ title }}</span>
          <div class="header-right">
            <slot name="header-extra" />
            <el-button @click="toggleFullscreen">
              <el-icon><FullScreen v-if="!isFullscreen" /><Close v-else /></el-icon>
              {{ isFullscreen ? '退出全屏' : '全屏' }}
            </el-button>
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <slot />
      </el-tabs>
    </el-card>

    <el-button v-if="isFullscreen" class="fullscreen-exit-btn" type="danger" circle @click="toggleFullscreen">
      <el-icon :size="20"><Close /></el-icon>
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, provide, onMounted, onUnmounted } from 'vue'
import { FullScreen, Close } from '@element-plus/icons-vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { showDetailedError } from '../utils/errorFeedback'

defineProps<{ title: string }>()

const isFullscreen = ref(false)
const activeTab = ref('info')

provide('isFullscreen', isFullscreen)

async function toggleFullscreen() {
  const next = !isFullscreen.value
  isFullscreen.value = next
  try {
    await getCurrentWindow().setFullscreen(next)
  } catch (error) {
    isFullscreen.value = !next
    showDetailedError(next ? '进入全屏' : '退出全屏', error)
  }
}

onMounted(async () => {
  try {
    isFullscreen.value = await getCurrentWindow().isFullscreen()
  } catch (error) {
    showDetailedError('读取全屏状态', error)
  }
  document.addEventListener('keydown', onKeydown)
})
onUnmounted(() => { document.removeEventListener('keydown', onKeydown) })

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && isFullscreen.value) toggleFullscreen()
}

defineExpose({ isFullscreen, activeTab })
</script>

<style scoped>
.fullscreen-exit-btn {
  position: fixed;
  top: 12px;
  right: 12px;
  z-index: 2001;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
}
</style>
