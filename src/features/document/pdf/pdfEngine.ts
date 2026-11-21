import { PDFJS_OPTIONS, pdfjs } from './pdfjs'
import 'pdfjs-dist/web/pdf_viewer.css'
import { EventBus, PDFLinkService, PDFViewer } from 'pdfjs-dist/web/pdf_viewer.mjs'
import type { PageMove } from '../pageScroll'
import type { OpenPaged, PagedEvents, PagedStart } from '../paged/types'
import { watchWidth } from '../paged/watchWidth'

// The pdf.js viewer behind the PDF display (specs/07-reader.md, "PDF").

type Viewer = { eventBus: EventBus; linkService: PDFLinkService; viewer: PDFViewer }

function createViewer(container: HTMLDivElement): Viewer {
  const eventBus = new EventBus()
  const linkService = new PDFLinkService({ eventBus })
  const viewer = new PDFViewer({ container, eventBus, linkService })
  linkService.setViewer(viewer)
  return { eventBus, linkService, viewer }
}

// 100% is the current page filling the stage width.
function applyZoom(viewer: PDFViewer, zoom: number) {
  if (viewer.pagesCount === 0) return
  viewer.currentScaleValue = 'page-width'
  viewer.currentScale = (viewer.currentScale * zoom) / 100
}

function turnPage(viewer: PDFViewer, move: PageMove) {
  if (move === 'next') viewer.nextPage()
  else if (move === 'previous') viewer.previousPage()
  else viewer.currentPageNumber = move === 'first' ? 1 : viewer.pagesCount
}

// Opens on the saved page, or on the last one if the PDF has fewer now.
function watchPages({ eventBus, viewer }: Viewer, start: PagedStart, zoom: () => number, events: PagedEvents) {
  const report = () => events.onPages({ current: viewer.currentPageNumber, total: viewer.pagesCount })
  eventBus.on('pagesinit', () => {
    applyZoom(viewer, zoom())
    if (start.page) viewer.currentPageNumber = Math.min(start.page, viewer.pagesCount)
    report()
  })
  eventBus.on('pagechanging', report)
}

// pdf.js follows in-document destinations ('#...') itself; any other link
// goes to the core, never to the webview.
function interceptLinks(container: HTMLElement, onLink: (href: string) => void) {
  const onClick = (event: MouseEvent) => {
    const link = event.target instanceof Element ? event.target.closest('a') : null
    const href = link?.getAttribute('href')
    if (!href || href.startsWith('#')) return
    event.preventDefault()
    event.stopPropagation()
    onLink(href)
  }
  container.addEventListener('click', onClick, true)
  return () => container.removeEventListener('click', onClick, true)
}

function loadDocument(bytes: ArrayBuffer, { linkService, viewer }: Viewer, events: PagedEvents) {
  const task = pdfjs.getDocument({ data: new Uint8Array(bytes), ...PDFJS_OPTIONS })
  task.onPassword = (submit: (password: string) => void, reason: number) =>
    events.onPassword(reason === pdfjs.PasswordResponses.INCORRECT_PASSWORD, submit)
  task.promise.then((document) => {
    viewer.setDocument(document)
    linkService.setDocument(document)
  }, events.onFailed)
  return task
}

// Shows `bytes` in `container`, which must be absolutely positioned and hold
// an empty `.pdfViewer` div. pdf.js takes the buffer: pass a copy to keep it.
export const openPdf: OpenPaged = (container, bytes, start, events) => {
  const parts = createViewer(container)
  let zoom = start.zoom
  watchPages(parts, start, () => zoom, events)
  const task = loadDocument(bytes, parts, events)
  const stops = [interceptLinks(container, events.onLink), watchWidth(container, () => applyZoom(parts.viewer, zoom))]
  const setZoom = (value: number) => {
    zoom = value
    applyZoom(parts.viewer, zoom)
  }
  const close = () => {
    stops.forEach((stop) => stop())
    void task.destroy()
  }
  return { turn: (move) => turnPage(parts.viewer, move), setZoom, close }
}
