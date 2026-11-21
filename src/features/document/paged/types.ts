import type { PageMove } from '../pageScroll'

// What a paged display (PDF, DOCX) and its engine say to each other
// (specs/07-reader.md, "Exibição"). The display only talks to the engine
// through these, so tests can replace the engine.

export type Pages = { current: number; total: number }

export type PagedEvents = {
  /** The engine needs a password (pdf.js); `wrong` after one was rejected. */
  onPassword: (wrong: boolean, submit: (password: string) => void) => void
  onPages: (pages: Pages) => void
  onFailed: (error: unknown) => void
  /** A link that does not point inside the document. */
  onLink: (href: string) => void
}

export type PagedStart = { page: number | null; zoom: number }

export type PagedSession = {
  turn: (move: PageMove) => void
  setZoom: (zoom: number) => void
  close: () => void
}

/** Shows `bytes` in `container`; the engine may take the buffer. */
export type OpenPaged = (container: HTMLDivElement, bytes: ArrayBuffer, start: PagedStart, events: PagedEvents) => PagedSession

/** Turns the pages of the document on the stage (the page keys). */
export type Pager = (move: PageMove) => void
