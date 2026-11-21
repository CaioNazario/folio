import { useCallback, useRef, useState } from 'react'
import { type PageMove, scrollPage } from '../pageScroll'
import type { Pager } from '../pdf/usePdfView'
import type { DocumentActions, OpenDocument, Reading, StageContent } from '../types'
import type { ReaderState } from '../useReaderState'
import { useReaderShortcuts } from '../useReaderShortcuts'
import { useReadingPosition } from '../useReadingPosition'
import { DocumentHeader, type OpenMenu } from './DocumentHeader'
import { Stage } from './Stage'

type DocumentViewProps = {
  document: OpenDocument
  content: StageContent
  reader: ReaderState
  actions: DocumentActions
}

const SHOWN: StageContent['status'][] = ['text', 'html', 'pdf']

// Document view (Design/specs/05-document-view.md): header + stage. Toggling
// fullscreen closes the menus.
export function DocumentView({ document, content, reader, actions }: DocumentViewProps) {
  const [openMenu, setOpenMenu] = useState<OpenMenu>(null)
  const withMenusClosed: ReaderState = {
    ...reader,
    toggleFullscreen: () => {
      setOpenMenu(null)
      reader.toggleFullscreen()
    },
  }
  const page = useRef<HTMLElement>(null)
  const pager = useRef<Pager>(null)
  // A PDF turns its pages; MD and TXT scroll a screen.
  const turnPage = useCallback((move: PageMove) => (pager.current ? pager.current(move) : scrollPage(page.current, move)), [])
  useReaderShortcuts(withMenusClosed, openMenu !== null, turnPage)
  const onPosition = useReadingPosition(SHOWN.includes(content.status) ? document.id : null)
  const reading: Reading = { start: document.position, onPosition, pageRef: page, pagerRef: pager }
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <DocumentHeader document={document} actions={actions} openMenu={openMenu} onOpenMenu={setOpenMenu} />
      <Stage
        key={`${document.id}:${content.status}`}
        content={content}
        format={document.format}
        reading={reading}
        reader={withMenusClosed}
        actions={actions}
      />
    </div>
  )
}
