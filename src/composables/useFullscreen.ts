import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { showDetailedError } from '../utils/errorFeedback'

/**
 * 统一全屏逻辑：状态、切换、ESC 退出、退出按钮
 * 供各管理页面共用，避免重复实现
 */
export function useFullscreen() {
  const isFullscreen = ref(false)

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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isFullscreen.value) {
      toggleFullscreen()
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

  onUnmounted(() => {
    document.removeEventListener('keydown', onKeydown)
  })

  return { isFullscreen, toggleFullscreen }
}
