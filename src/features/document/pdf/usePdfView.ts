import { type Dispatch, type RefObject, type SetStateAction, useEffect, useRef, useState } from 'react'
import type { PageMove } from '../pageScroll'
import { openPdf, type Pages, type PdfEvents, type PdfSession } from './pdfEngine'

export type PdfPhase =
  | { kind: 'loading' }
  | { kind: 'password'; wrong: boolean; submit: (password: string) => void }
  | { kind: 'ready' }
  | { kind: 'failed' }

/** Turns the pages of the document on the stage (the page keys). */
export type Pager = (move: PageMove) => void

export type PdfViewOptions = {
  zoom: number
  /** Page to open on, where the reader stopped. */
  initialPage: number | null
  onPages: (pages: Pages) => void
  onLink: (href: string) => void
  pagerRef: RefObject<Pager | null>
}

type SetPhase = Dispatch<SetStateAction<PdfPhase>>

const LOADING: PdfPhase = { kind: 'loading' }
const READY: PdfPhase = { kind: 'ready' }
const FAILED: PdfPhase = { kind: 'failed' }

// StrictMode opens twice in development, and pdf.js takes the buffer it gets.
const ownCopy = (bytes: ArrayBuffer) => (import.meta.env.DEV ? bytes.slice(0) : bytes)

// What pdf.js reports becomes the phase and reaches the stage.
function sessionEvents(latest: RefObject<PdfViewOptions>, setPhase: SetPhase): PdfEvents {
  return {
    onPassword: (wrong, submit) => setPhase({ kind: 'password', wrong, submit }),
    onPages: (pages) => {
      setPhase((phase) => (phase.kind === 'ready' ? phase : READY))
      latest.current.onPages(pages)
    },
    onFailed: (error) => {
      console.error('pdf.js could not open the document', error)
      setPhase(FAILED)
    },
    onLink: (href) => latest.current.onLink(href),
  }
}

// A closed session still hears pdf.js abort its loading: drop that.
function whileActive(events: PdfEvents, active: () => boolean): PdfEvents {
  const guard =
    <A extends unknown[]>(handler: (...args: A) => void) =>
    (...args: A) => {
      if (active()) handler(...args)
    }
  return { onPassword: guard(events.onPassword), onPages: guard(events.onPages), onFailed: guard(events.onFailed), onLink: guard(events.onLink) }
}

// One pdf.js session per byte buffer, closed when the display goes away.
function useSession(container: RefObject<HTMLDivElement | null>, bytes: ArrayBuffer, latest: RefObject<PdfViewOptions>, setPhase: SetPhase) {
  const session = useRef<PdfSession | null>(null)
  useEffect(() => {
    const element = container.current
    if (!element) return
    let active = true
    const { zoom, initialPage, pagerRef } = latest.current
    const opened = openPdf(element, ownCopy(bytes), { page: initialPage, zoom }, whileActive(sessionEvents(latest, setPhase), () => active))
    session.current = opened
    pagerRef.current = opened.turn
    return () => {
      active = false
      opened.close()
      session.current = null
      pagerRef.current = null
    }
  }, [container, bytes, latest, setPhase])
  return session
}

// The PDF on the stage: pdf.js session, phase and password answer
// (specs/07-reader.md, "PDF").
export function usePdfView(bytes: ArrayBuffer, options: PdfViewOptions) {
  const container = useRef<HTMLDivElement>(null)
  const latest = useRef(options)
  useEffect(() => {
    latest.current = options
  })
  const [phase, setPhase] = useState<PdfPhase>(LOADING)
  const session = useSession(container, bytes, latest, setPhase)
  useEffect(() => session.current?.setZoom(options.zoom), [session, options.zoom])
  const submitPassword = (password: string) => {
    if (phase.kind !== 'password') return
    setPhase(LOADING)
    phase.submit(password)
  }
  return { container, phase, submitPassword }
}
