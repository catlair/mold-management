import { ref, computed, onMounted, type Ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { showDetailedError } from '../utils/errorFeedback'

export function useDataPage() {
  const isFullscreen = ref(false)
  const activeTab = ref('info')
  const loading = ref(true)

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
  })

  function usePagination<T>(list: Ref<T[]>) {
    const currentPage = ref(1)
    const pageSize = ref(10)
    const paginated = computed(() => {
      const start = (currentPage.value - 1) * pageSize.value
      return list.value.slice(start, start + pageSize.value)
    })
    function resetPage() { currentPage.value = 1 }
    return { currentPage, pageSize, paginated, resetPage }
  }

  const maxHeight = computed(() =>
    isFullscreen.value ? 'calc(100vh - 26px)' : 'calc(100vh - 170px)'
  )

  return { isFullscreen, activeTab, loading, toggleFullscreen, usePagination, maxHeight }
}
