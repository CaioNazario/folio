import { useCallback, useEffect, useState } from 'react'
import type { DocumentRef } from '@/features/sidebar/types'
import { openDocument, recordOpened } from './api'
import { failedContent, loadContent, toOpenDocument } from './documentData'
import type { OpenDocument, StageContent } from './types'

type Loaded = { key: string; document: OpenDocument | null; content: StageContent }

export type DocumentEvents = {
  /** The opening was recorded: the recents changed. */
  onRecorded: () => void
  /** The file is gone from disk: the sidebar no longer shows it (specs/07-reader.md). */
  onNotFound: () => void
}

const SHOWN: StageContent['status'][] = ['text', 'html', 'pdf', 'docx']

async function load(id: string, password: string | null): Promise<Omit<Loaded, 'key'>> {
  try {
    const document = toOpenDocument(await openDocument(id))
    return { document, content: await loadContent(document, password) }
  } catch (error) {
    return { document: null, content: failedContent(error) }
  }
}

// Until open_document answers (or when it fails), the header shows what the
// sidebar knows.
function placeholder(ref: DocumentRef): OpenDocument {
  return { ...ref, format: 'txt', folders: [], derived: false, conversions: [], valid: true, protected: false, position: null }
}

// Records the opening once something is shown; reports a missing file.
function afterLoad(id: string, content: StageContent, events: DocumentEvents) {
  if (SHOWN.includes(content.status)) recordOpened(id).then(events.onRecorded).catch(console.error)
  if (content.status === 'notFound') events.onNotFound()
}

// The result of loading `ref` for this attempt, or null while it loads.
function useLoaded(ref: DocumentRef | null, key: string, password: string | null, events: DocumentEvents): Loaded | null {
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  useEffect(() => {
    if (!ref) return
    let current = true
    void load(ref.id, password).then((result) => {
      if (!current) return
      setLoaded({ key, ...result })
      afterLoad(ref.id, result.content, events)
    })
    return () => {
      current = false
    }
  }, [ref, key, password, events])
  return loaded?.key === key ? loaded : null
}

/** Each retry or password is a new attempt; the password belongs to one document. */
type Attempt = { count: number; id: string | null; password: string | null }

// Opens `ref`: document info, then the stage content. Loading lasts until
// this exact document and attempt arrive. A password lives only here, for
// the open document (specs/07-reader.md, "DOCX").
export function useDocument(ref: DocumentRef | null, events: DocumentEvents) {
  const [attempt, setAttempt] = useState<Attempt>({ count: 0, id: null, password: null })
  const key = ref ? `${ref.id}:${attempt.count}` : ''
  const password = ref && attempt.id === ref.id ? attempt.password : null
  const fresh = useLoaded(ref, key, password, events)
  const retry = useCallback(() => setAttempt((value) => ({ ...value, count: value.count + 1 })), [])
  const id = ref?.id ?? null
  const submitPassword = useCallback((value: string) => setAttempt(({ count }) => ({ count: count + 1, id, password: value })), [id])
  const document = fresh?.document ?? (ref && placeholder(ref))
  return { document, content: fresh?.content ?? ({ status: 'loading' } as StageContent), retry, submitPassword, password }
}
