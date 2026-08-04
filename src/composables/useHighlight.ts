import { ref, watch, nextTick, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export function useHighlight(tableData: any) {
  const route = useRoute()
  const router = useRouter()
  const highlightId = ref<string>('')

  function checkHighlight() {
    const id = route.query.highlight as string
    if (!id || !tableData.value) return
    highlightId.value = id
    nextTick(() => {
      setTimeout(() => {
        // vxe-table 行选择器：优先 .vxe-body--row，兼容旧 .el-table__body tr
        const rows = document.querySelectorAll<HTMLElement>('.vxe-body--row, .el-table__body tr')
        // 找到目标行索引（基于 rowid 属性或数据索引）
        let targetRow: HTMLElement | null = null
        for (let i = 0; i < tableData.value.length; i++) {
          if (tableData.value[i].id === id) {
            // 优先通过 vxe-table 的 rowid 定位
            targetRow = document.querySelector<HTMLElement>(`[rowid="${id}"]`)
            if (!targetRow) targetRow = rows[i] as HTMLElement
            break
          }
        }
        if (targetRow) {
          targetRow.scrollIntoView({ behavior: 'smooth', block: 'center' })
          targetRow.classList.add('highlight-flash')
          setTimeout(() => targetRow?.classList.remove('highlight-flash'), 2000)
        }
        if (route.query.highlight) {
          router.replace({ path: route.path })
        }
      }, 500)
    })
  }

  watch(() => route.query.highlight, (val) => {
    if (val) checkHighlight()
  })

  onMounted(() => {
    if (route.query.highlight) checkHighlight()
  })

  return { highlightId, checkHighlight }
}
