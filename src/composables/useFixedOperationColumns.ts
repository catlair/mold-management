import { nextTick, onMounted, onUnmounted } from 'vue'

const SCROLL_CONTAINER_SELECTOR = '.dt-wrap, .record-table-scroll, .backup-table-scroll, .stock-table-shell'
const OPERATION_COLUMN_SELECTOR = '.operation-column'
const TABLE_HEADER_SELECTOR = '.vxe-table--header-wrapper'

function updateFixedTableRegions(container: HTMLElement) {
  const table = container.querySelector<HTMLElement>('.vxe-table')
  if (!table) return

  const header = table.querySelector<HTMLElement>(TABLE_HEADER_SELECTOR)
  if (header) {
    const maxScrollTop = Math.max(0, container.scrollHeight - container.clientHeight)
    const translateY = Math.min(container.scrollTop, maxScrollTop)
    header.style.setProperty('--table-header-offset', `${translateY}px`)
  }

  const columns = table.querySelectorAll<HTMLElement>(OPERATION_COLUMN_SELECTOR)
  if (!columns.length) return

  // 只有内容真正横向溢出时才固定操作列。
  // scrollbar-gutter 会让 offsetWidth 与 clientWidth 存在差值；若无溢出仍参与补偿，
  // 操作列会被错误左移一个滚动条宽度，造成与前一列重叠。
  const hasHorizontalOverflow = container.scrollWidth > container.clientWidth + 1
  container.classList.toggle('has-horizontal-overflow', hasHorizontalOverflow)

  const scrollbarGutter = hasHorizontalOverflow
    ? Math.max(0, container.offsetWidth - container.clientWidth)
    : 0
  const maxScrollLeft = hasHorizontalOverflow
    ? Math.max(0, container.scrollWidth - container.offsetWidth)
    : 0
  const translateX = hasHorizontalOverflow
    ? Math.min(0, container.scrollLeft - maxScrollLeft - scrollbarGutter)
    : 0

  columns.forEach((column) => {
    column.style.setProperty('--operation-column-offset', `${translateX}px`)
  })
}

export function useFixedOperationColumns() {
  let observer: ResizeObserver | null = null
  let mutationObserver: MutationObserver | null = null
  let frameId = 0
  let updateAll = false
  const containers = new Set<HTMLElement>()
  const pendingContainers = new Set<HTMLElement>()

  const pruneContainers = () => {
    containers.forEach((container) => {
      if (container.isConnected) return
      container.removeEventListener('scroll', handleScroll)
      observer?.unobserve(container)
      containers.delete(container)
      pendingContainers.delete(container)
    })
  }

  const scheduleUpdate = (container?: HTMLElement) => {
    if (container) pendingContainers.add(container)
    else updateAll = true
    if (frameId) return

    frameId = requestAnimationFrame(() => {
      frameId = 0
      pruneContainers()
      if (updateAll) {
        containers.forEach(updateFixedTableRegions)
      } else {
        pendingContainers.forEach(updateFixedTableRegions)
      }
      updateAll = false
      pendingContainers.clear()
    })
  }

  const handleScroll = (event: Event) => {
    // 滚动时同步更新，避免下一帧调度被 ResizeObserver 覆盖而产生表头或固定列漂移。
    updateFixedTableRegions(event.currentTarget as HTMLElement)
  }

  const registerContainers = () => {
    document.querySelectorAll<HTMLElement>(SCROLL_CONTAINER_SELECTOR).forEach((container) => {
      if (containers.has(container)) return
      containers.add(container)
      container.addEventListener('scroll', handleScroll, { passive: true })
      observer?.observe(container)
      scheduleUpdate(container)
    })
  }

  const handleViewportResize = () => {
    registerContainers()
    scheduleUpdate()
  }

  onMounted(async () => {
    await nextTick()
    observer = new ResizeObserver(() => scheduleUpdate())
    mutationObserver = new MutationObserver(() => {
      registerContainers()
      scheduleUpdate()
    })
    mutationObserver.observe(document.body, { childList: true, subtree: true })
    registerContainers()
    window.addEventListener('resize', handleViewportResize)
  })

  onUnmounted(() => {
    if (frameId) cancelAnimationFrame(frameId)
    observer?.disconnect()
    mutationObserver?.disconnect()
    window.removeEventListener('resize', handleViewportResize)
    containers.forEach((container) => container.removeEventListener('scroll', handleScroll))
    containers.clear()
  })
}
