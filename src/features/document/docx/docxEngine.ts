import { renderAsync } from 'docx-preview'
import type { PageMove } from '../pageScroll'
import type { OpenPaged, PagedEvents } from '../paged/types'
import { watchWidth } from '../paged/watchWidth'
import { declaredWidth, dropEmptyPages, pageAt, scrollToPage } from './docxPages'
import { fixUndrawableImages } from './undrawableImages'

// docx-preview behind the DOCX display (specs/07-reader.md, "DOCX").

const OPTIONS = { inWrapper: false, breakPages: true, ignoreLastRenderedPageBreak: false, renderComments: false }

/** Space around the pages, in px; the display styles the host with it (p-3). */
export const DOCX_GUTTER = 12

type View = { container: HTMLDivElement; host: HTMLElement; pages: HTMLElement[]; zoom: number }

async function renderPages(host: HTMLElement, bytes: ArrayBuffer): Promise<HTMLElement[]> {
  await renderAsync(bytes, host, host, OPTIONS)
  const pages = dropEmptyPages(host)
  await fixUndrawableImages(host)
  return pages
}

// 100% is the page filling the stage width; the reader stays on its page.
function applyZoom(view: View) {
  const [first] = view.pages
  if (!first) return
  const page = view.pages[pageAt(view.container, view.pages)]
  const fit = (view.container.clientWidth - 2 * DOCX_GUTTER) / declaredWidth(first)
  view.host.style.zoom = String((fit * view.zoom) / 100)
  scrollToPage(view.container, page, DOCX_GUTTER)
}

function goTo(view: View, index: number) {
  const page = view.pages[Math.min(Math.max(index, 0), view.pages.length - 1)]
  if (page) scrollToPage(view.container, page, DOCX_GUTTER)
}

function turnPage(view: View, move: PageMove) {
  const current = pageAt(view.container, view.pages)
  const targets = { next: current + 1, previous: current - 1, first: 0, last: view.pages.length - 1 }
  goTo(view, targets[move])
}

function reportPages(view: View, events: PagedEvents) {
  events.onPages({ current: pageAt(view.container, view.pages) + 1, total: view.pages.length })
}

function watchScroll(view: View, events: PagedEvents) {
  let frame = 0
  const onScroll = () => {
    cancelAnimationFrame(frame)
    frame = requestAnimationFrame(() => reportPages(view, events))
  }
  view.container.addEventListener('scroll', onScroll, { passive: true })
  return () => {
    cancelAnimationFrame(frame)
    view.container.removeEventListener('scroll', onScroll)
  }
}

// '#bookmark' (table of contents, cross references) scrolls to the bookmark
// in the document; any other link goes to the core, never to the webview.
function interceptLinks(view: View, onLink: (href: string) => void) {
  const onClick = (event: MouseEvent) => {
    const link = event.target instanceof Element ? event.target.closest('a') : null
    if (!link) return
    event.preventDefault()
    const href = link.getAttribute('href') ?? ''
    if (href.startsWith('#')) view.host.querySelector(`[id="${CSS.escape(href.slice(1))}"]`)?.scrollIntoView({ block: 'start' })
    else if (href) onLink(href)
  }
  view.container.addEventListener('click', onClick, true)
  return () => view.container.removeEventListener('click', onClick, true)
}

// Shows the rendered pages: zoom, saved page, then the watchers.
function show(view: View, page: number | null, events: PagedEvents) {
  applyZoom(view)
  goTo(view, (page ?? 1) - 1)
  reportPages(view, events)
  return [watchScroll(view, events), interceptLinks(view, events.onLink), watchWidth(view.container, () => applyZoom(view))]
}

function closeSession(session: { closed: boolean; stops: (() => void)[] }, host: HTMLElement) {
  session.closed = true
  session.stops.forEach((stop) => stop())
  host.remove()
}

// Each session renders into its own host, so a closed one that is still
// rendering never mixes its pages with the next one's.
export const openDocx: OpenPaged = (container, bytes, start, events) => {
  const host = container.appendChild(document.createElement('div'))
  const view: View = { container, host, pages: [], zoom: start.zoom }
  const session = { closed: false, stops: [] as (() => void)[] }
  renderPages(host, bytes).then((pages) => {
    if (session.closed) return
    view.pages = pages
    session.stops = show(view, start.page, events)
  }, events.onFailed)
  const setZoom = (zoom: number) => {
    view.zoom = zoom
    applyZoom(view)
  }
  return { turn: (move) => turnPage(view, move), setZoom, close: () => closeSession(session, host) }
}
