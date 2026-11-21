// Page navigation keys (specs/07-reader.md, "Atalhos de teclado"). MD and
// TXT scroll a screen; PDF turns pages (paged/usePagedView.ts).

export type PageMove = 'next' | 'previous' | 'first' | 'last'

const KEY_MOVES: Partial<Record<string, PageMove>> = {
  PageDown: 'next',
  ArrowRight: 'next',
  PageUp: 'previous',
  ArrowLeft: 'previous',
  Home: 'first',
  End: 'last',
}

export function pageMove(key: string): PageMove | undefined {
  return KEY_MOVES[key]
}

// Where `move` takes a scrolling page: one screen down or up, or either end.
export function scrollTarget(page: HTMLElement, move: PageMove): number {
  const { scrollTop, clientHeight, scrollHeight } = page
  if (move === 'first') return 0
  if (move === 'last') return scrollHeight - clientHeight
  return scrollTop + (move === 'next' ? clientHeight : -clientHeight)
}

export function scrollPage(page: HTMLElement | null, move: PageMove) {
  if (!page) return
  page.scrollTop = scrollTarget(page, move)
}

// Heading ids carry a prefix so a title never takes one of the app's own ids
// (specs/07-reader.md, "Âncoras no MD"). A missing heading does nothing.
export function scrollToHeading(fragment: string) {
  document.getElementById(`folio-h-${fragment}`)?.scrollIntoView({ block: 'start' })
}
