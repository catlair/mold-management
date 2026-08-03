import { ref, onMounted } from 'vue'
import { allowDeleteApi } from '../api'
import { showDetailedError } from '../utils/errorFeedback'

const allowDelete = ref(false)

export function useAllowDelete() {
  onMounted(async () => {
    try {
      allowDelete.value = await allowDeleteApi.get()
    } catch (error) {
      showDetailedError('加载删除功能设置', error)
    }
  })

  async function setAllowDelete(val: boolean) {
    allowDelete.value = val
    await allowDeleteApi.set(val)
  }

  return { allowDelete, setAllowDelete }
}
