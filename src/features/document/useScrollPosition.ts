import { useCallback, useEffect, useRef } from 'react'
import { savePosition } from './api'

// Remembers how far the reader scrolled `documentId` and saves it when the
// document changes, the view closes or the window goes away (specs/07,
// "Posição"). Nothing is saved for a document that was never scrolled.
export function useScrollPosition(documentId: string | null) {
  const fraction = useRef<number | null>(null)
  useEffect(() => {
    if (!documentId) return
    fraction.current = null
    const save = () => {
      if (fraction.current === null) return
      savePosition(documentId, { kind: 'Scroll', fraction: fraction.current }).catch(console.error)
    }
    window.addEventListener('pagehide', save)
    return () => {
      window.removeEventListener('pagehide', save)
      save()
    }
  }, [documentId])
  return useCallback((value: number) => {
    fraction.current = value
  }, [])
}
