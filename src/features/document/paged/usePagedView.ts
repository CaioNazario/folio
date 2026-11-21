import { type Dispatch, type RefObject, type SetStateAction, useEffect, useRef, useState } from 'react'
import type { OpenPaged, Pager, Pages, PagedEvents, PagedSession } from './types'

export type PagedPhase =
  | { kind: 'loading' }
  | { kind: 'password'; wrong: boolean; submit: (password: string) => void }
  | { kind: 'ready' }
  | { kind: 'failed' }

export type PagedViewOptions = {
  zoom: number
  /** Page to open on, where the reader stopped. */
  initialPage: number | null
  onPages: (pages: Pages) => void
  onLink: (href: string) => void
  pagerRef: RefObject<Pager | null>
}

type SetPhase = Dispatch<SetStateAction<PagedPhase>>

const LOADING: PagedPhase = { kind: 'loading' }
const READY: PagedPhase = { kind: 'ready' }
const FAILED: PagedPhase = { kind: 'failed' }

// StrictMode opens twice in development, and an engine may take the buffer.
const ownCopy = (bytes: ArrayBuffer) => (import.meta.env.DEV ? bytes.slice(0) : bytes)

// What the engine reports becomes the phase and reaches the stage.
function sessionEvents(latest: RefObject<PagedViewOptions>, setPhase: SetPhase): PagedEvents {
  return {
    onPassword: (wrong, submit) => setPhase({ kind: 'password', wrong, submit }),
    onPages: (pages) => {
      setPhase((phase) => (phase.kind === 'ready' ? phase : READY))
      latest.current.onPages(pages)
    },
    onFailed: (error) => {
      console.error('the document could not be shown', error)
      setPhase(FAILED)
    },
    onLink: (href) => latest.current.onLink(href),
  }
}

// A closed session may still report (pdf.js aborting its loading): drop that.
function whileActive(events: PagedEvents, active: () => boolean): PagedEvents {
  const guard =
    <A extends unknown[]>(handler: (...args: A) => void) =>
    (...args: A) => {
      if (active()) handler(...args)
    }
  return { onPassword: guard(events.onPassword), onPages: guard(events.onPages), onFailed: guard(events.onFailed), onLink: guard(events.onLink) }
}

// One engine session per byte buffer, closed when the display goes away.
function useSession(open: OpenPaged, container: RefObject<HTMLDivElement | null>, bytes: ArrayBuffer, latest: RefObject<PagedViewOptions>, setPhase: SetPhase) {
  const session = useRef<PagedSession | null>(null)
  useEffect(() => {
    const element = container.current
    if (!element) return
    let active = true
    const { zoom, initialPage, pagerRef } = latest.current
    const opened = open(element, ownCopy(bytes), { page: initialPage, zoom }, whileActive(sessionEvents(latest, setPhase), () => active))
    session.current = opened
    pagerRef.current = opened.turn
    return () => {
      active = false
      opened.close()
      session.current = null
      pagerRef.current = null
    }
  }, [open, container, bytes, latest, setPhase])
  return session
}

// A paged document on the stage: engine session, phase and password answer
// (specs/07-reader.md, "Exibição"). `open` must be stable.
export function usePagedView(open: OpenPaged, bytes: ArrayBuffer, options: PagedViewOptions) {
  const container = useRef<HTMLDivElement>(null)
  const latest = useRef(options)
  useEffect(() => {
    latest.current = options
  })
  const [phase, setPhase] = useState<PagedPhase>(LOADING)
  const session = useSession(open, container, bytes, latest, setPhase)
  useEffect(() => session.current?.setZoom(options.zoom), [session, options.zoom])
  const submitPassword = (password: string) => {
    if (phase.kind !== 'password') return
    setPhase(LOADING)
    phase.submit(password)
  }
  return { container, phase, submitPassword }
}
