import { ref, onMounted } from 'vue'
import { allowDeleteApi } from '../api'

const allowDelete = ref(false)

export function useAllowDelete() {
  onMounted(async () => {
    try {
      allowDelete.value = await allowDeleteApi.get()
    } catch {}
  })

  async function setAllowDelete(val: boolean) {
    allowDelete.value = val
    await allowDeleteApi.set(val)
  }

  return { allowDelete, setAllowDelete }
}
