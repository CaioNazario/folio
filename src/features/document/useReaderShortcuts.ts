import { useEffect } from 'react'
import { type PageMove, pageMove } from './pageScroll'
import type { ReaderState } from './useReaderState'

function inTextField(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)
}

const hasModifier = (event: KeyboardEvent) => event.ctrlKey || event.altKey || event.metaKey || event.shiftKey

function zoomShortcut(event: KeyboardEvent, reader: ReaderState): (() => void) | undefined {
  if (event.key === '+' || event.key === '=') return reader.zoomIn
  if (event.key === '-') return reader.zoomOut
  if (event.key === '0') return reader.resetZoom
  return undefined
}

type TurnPage = (move: PageMove) => void

function pageShortcut(event: KeyboardEvent, turnPage: TurnPage | null): (() => void) | undefined {
  const move = turnPage && !hasModifier(event) ? pageMove(event.key) : undefined
  return move && (() => turnPage?.(move))
}

// `turnPage` is null while a menu is open: Radix uses Esc and the arrows there.
function shortcut(event: KeyboardEvent, reader: ReaderState, turnPage: TurnPage | null) {
  if (event.key === 'F11') return reader.toggleFullscreen
  if (event.key === 'Escape') return reader.exitFullscreen
  if (event.ctrlKey) return zoomShortcut(event, reader)
  return pageShortcut(event, turnPage)
}

// Reader shortcuts (specs/07-reader.md, "Atalhos de teclado"). Ignored inside
// text fields; an open menu keeps Esc and the page keys for itself.
export function useReaderShortcuts(reader: ReaderState, menuOpen: boolean, turnPage: TurnPage) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (inTextField(event.target) || (menuOpen && event.key === 'Escape')) return
      const action = shortcut(event, reader, menuOpen ? null : turnPage)
      if (!action) return
      event.preventDefault()
      action()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [reader, menuOpen, turnPage])
}
