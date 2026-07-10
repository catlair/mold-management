import { useState, useCallback, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function useFullscreen() {
  const [isFullscreen, setIsFullscreen] = useState(false)

  const toggle = useCallback(async () => {
    const next = !isFullscreen
    setIsFullscreen(next)
    try {
      await getCurrentWindow().setFullscreen(next)
    } catch {}
  }, [isFullscreen])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isFullscreen) toggle()
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [isFullscreen, toggle])

  useEffect(() => {
    getCurrentWindow().isFullscreen().then(setIsFullscreen).catch(() => {})
  }, [])

  return { isFullscreen, toggle }
}
