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

const SHOWN: StageContent['status'][] = ['text', 'html']

async function load(id: string): Promise<Omit<Loaded, 'key'>> {
  try {
    const document = toOpenDocument(await openDocument(id))
    return { document, content: await loadContent(document) }
  } catch (error) {
    return { document: null, content: failedContent(error) }
  }
}

// Until open_document answers (or when it fails), the header shows what the
// sidebar knows.
function placeholder(ref: DocumentRef): OpenDocument {
  return { ...ref, format: 'txt', folders: [], derived: false, conversions: [], valid: true, scroll: null }
}

// Records the opening once something is shown; reports a missing file.
function afterLoad(id: string, content: StageContent, events: DocumentEvents) {
  if (SHOWN.includes(content.status)) recordOpened(id).then(events.onRecorded).catch(console.error)
  if (content.status === 'notFound') events.onNotFound()
}

// The result of loading `ref` for this attempt, or null while it loads.
function useLoaded(ref: DocumentRef | null, key: string, events: DocumentEvents): Loaded | null {
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  useEffect(() => {
    if (!ref) return
    let current = true
    void load(ref.id).then((result) => {
      if (!current) return
      setLoaded({ key, ...result })
      afterLoad(ref.id, result.content, events)
    })
    return () => {
      current = false
    }
  }, [ref, key, events])
  return loaded?.key === key ? loaded : null
}

// Opens `ref`: document info, then the stage content. Loading lasts until
// this exact document and attempt arrive.
export function useDocument(ref: DocumentRef | null, events: DocumentEvents) {
  const [attempt, setAttempt] = useState(0)
  const key = ref ? `${ref.id}:${attempt}` : ''
  const fresh = useLoaded(ref, key, events)
  const retry = useCallback(() => setAttempt((value) => value + 1), [])
  const document = fresh?.document ?? (ref && placeholder(ref))
  return { document, content: fresh?.content ?? ({ status: 'loading' } as StageContent), retry }
}
