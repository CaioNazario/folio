import { useCallback, useEffect, useState } from 'react'
import type { DocumentRef } from '@/features/sidebar/types'
import { getTextContent, recordOpened } from './api'
import { failedContent } from './stageContent'
import type { OpenDocument, StageContent } from './types'

type Loaded = { key: string; content: StageContent }

function toOpenDocument(ref: DocumentRef): OpenDocument {
  return { ...ref, derived: false, conversions: [] }
}

async function loadText(id: string): Promise<StageContent> {
  try {
    return { status: 'text', ...(await getTextContent(id)) }
  } catch (error) {
    return failedContent(error)
  }
}

// What the stage shows for `ref`: loading until the content for this exact
// document and attempt arrives. Only TXT has a content command so far.
function useStageContent(ref: DocumentRef | null, attempt: number, onShown: (id: string) => void): StageContent {
  const key = ref ? `${ref.id}:${attempt}` : ''
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  useEffect(() => {
    if (!ref || ref.format !== 'txt') return
    let current = true
    void loadText(ref.id).then((content) => {
      if (!current) return
      setLoaded({ key, content })
      if (content.status === 'text') onShown(ref.id)
    })
    return () => {
      current = false
    }
  }, [ref, key, onShown])
  if (ref && ref.format !== 'txt') return { status: 'unsupported' }
  return loaded?.key === key ? loaded.content : { status: 'loading' }
}

// The open document and its stage content; records the opening once shown.
export function useDocument(ref: DocumentRef | null, onRecorded: () => void) {
  const [attempt, setAttempt] = useState(0)
  const onShown = useCallback(
    (id: string) => void recordOpened(id).then(onRecorded).catch(console.error),
    [onRecorded],
  )
  const content = useStageContent(ref, attempt, onShown)
  const retry = useCallback(() => setAttempt((value) => value + 1), [])
  return { document: ref && toOpenDocument(ref), content, retry }
}
