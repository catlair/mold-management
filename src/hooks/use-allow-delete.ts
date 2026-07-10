import { useEffect } from 'react'
import { useAllowDelete as useAllowDeleteStore } from '@/stores/allow-delete'

export function useAllowDelete() {
  const store = useAllowDeleteStore()
  useEffect(() => { store.fetch() }, [])
  return store
}
