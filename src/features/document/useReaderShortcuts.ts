import { useEffect } from 'react'
import type { ReaderState } from './useReaderState'

function inTextField(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)
}

function shortcut(event: KeyboardEvent, reader: ReaderState): (() => void) | undefined {
  if (event.key === 'F11') return reader.toggleFullscreen
  if (event.key === 'Escape') return reader.exitFullscreen
  if (!event.ctrlKey) return undefined
  if (event.key === '+' || event.key === '=') return reader.zoomIn
  if (event.key === '-') return reader.zoomOut
  if (event.key === '0') return reader.resetZoom
  return undefined
}

// Reader shortcuts (specs/07-reader.md, "Atalhos de teclado"). Ignored inside
// text fields; Esc is left to an open menu, which Radix handles first.
export function useReaderShortcuts(reader: ReaderState, menuOpen: boolean) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (inTextField(event.target) || (menuOpen && event.key === 'Escape')) return
      const action = shortcut(event, reader)
      if (!action) return
      event.preventDefault()
      action()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [reader, menuOpen])
}
