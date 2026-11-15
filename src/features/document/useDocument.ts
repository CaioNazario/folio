import { useCallback, useEffect, useState } from 'react'
import type { DocumentRef } from '@/features/sidebar/types'
import { openDocument, recordOpened } from './api'
import { failedContent, loadContent, toOpenDocument } from './documentData'
import type { OpenDocument, StageContent } from './types'

type Loaded = { key: string; document: OpenDocument | null; content: StageContent }

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

// The result of loading `ref` for this attempt, or null while it loads.
// Records the opening once something is shown.
function useLoaded(ref: DocumentRef | null, key: string, onRecorded: () => void): Loaded | null {
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  useEffect(() => {
    if (!ref) return
    let current = true
    void load(ref.id).then((result) => {
      if (!current) return
      setLoaded({ key, ...result })
      if (SHOWN.includes(result.content.status)) recordOpened(ref.id).then(onRecorded).catch(console.error)
    })
    return () => {
      current = false
    }
  }, [ref, key, onRecorded])
  return loaded?.key === key ? loaded : null
}

// Opens `ref`: document info, then the stage content. Loading lasts until
// this exact document and attempt arrive.
export function useDocument(ref: DocumentRef | null, onRecorded: () => void) {
  const [attempt, setAttempt] = useState(0)
  const key = ref ? `${ref.id}:${attempt}` : ''
  const fresh = useLoaded(ref, key, onRecorded)
  const retry = useCallback(() => setAttempt((value) => value + 1), [])
  const document = fresh?.document ?? (ref && placeholder(ref))
  return { document, content: fresh?.content ?? ({ status: 'loading' } as StageContent), retry }
}
