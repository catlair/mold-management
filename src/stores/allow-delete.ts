import { create } from 'zustand'
import { allowDeleteApi } from '@/api'

interface AllowDeleteState {
  allowDelete: boolean
  fetch: () => Promise<void>
  set: (val: boolean) => Promise<void>
}

export const useAllowDelete = create<AllowDeleteState>((set) => ({
  allowDelete: false,
  fetch: async () => {
    try {
      const val = await allowDeleteApi.get()
      set({ allowDelete: val })
    } catch {}
  },
  set: async (val: boolean) => {
    await allowDeleteApi.set(val)
    set({ allowDelete: val })
  },
}))
