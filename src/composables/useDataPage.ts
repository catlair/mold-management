import { ref, computed, onMounted, type Ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function useDataPage() {
  const isFullscreen = ref(false)
  const activeTab = ref('info')
  const loading = ref(true)

  async function toggleFullscreen() {
    const next = !isFullscreen.value
    isFullscreen.value = next
    try {
      await getCurrentWindow().setFullscreen(next)
    } catch {}
  }

  onMounted(async () => {
    try {
      isFullscreen.value = await getCurrentWindow().isFullscreen()
    } catch {}
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
