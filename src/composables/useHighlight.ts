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
        const rows = document.querySelectorAll('.el-table__body tr')
        for (let i = 0; i < tableData.value.length; i++) {
          if (tableData.value[i].id === id) {
            rows[i]?.scrollIntoView({ behavior: 'smooth', block: 'center' })
            rows[i]?.classList.add('highlight-flash')
            setTimeout(() => rows[i]?.classList.remove('highlight-flash'), 2000)
            break
          }
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
