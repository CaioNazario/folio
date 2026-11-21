import { useCallback, useEffect, useRef } from 'react'
import type { ReadingPosition } from '@/lib/bindings/ReadingPosition'
import { savePosition } from './api'

// Remembers where the reader is in `documentId` (a page or a scroll fraction)
// and saves it when the document changes, the view closes or the window goes
// away (specs/07, "Posição"). Nothing is saved for a document never moved.
export function useReadingPosition(documentId: string | null) {
  const position = useRef<ReadingPosition | null>(null)
  useEffect(() => {
    if (!documentId) return
    position.current = null
    const save = () => {
      if (position.current === null) return
      savePosition(documentId, position.current).catch(console.error)
    }
    window.addEventListener('pagehide', save)
    return () => {
      window.removeEventListener('pagehide', save)
      save()
    }
  }, [documentId])
  return useCallback((value: ReadingPosition) => {
    position.current = value
  }, [])
}
