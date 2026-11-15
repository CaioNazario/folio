import { useRef, useState } from 'react'
import type { DocumentActions, OpenDocument, StageContent } from '../types'
import type { ReaderState } from '../useReaderState'
import { useReaderShortcuts } from '../useReaderShortcuts'
import { useScrollPosition } from '../useScrollPosition'
import { DocumentHeader, type OpenMenu } from './DocumentHeader'
import { Stage } from './Stage'

type DocumentViewProps = {
  document: OpenDocument
  content: StageContent
  reader: ReaderState
  actions: DocumentActions
}

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
  useReaderShortcuts(withMenusClosed, openMenu !== null, page)
  const shown = content.status === 'text' || content.status === 'html'
  const onScrollFraction = useScrollPosition(shown ? document.id : null)
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <DocumentHeader document={document} actions={actions} openMenu={openMenu} onOpenMenu={setOpenMenu} />
      <Stage
        key={`${document.id}:${content.status}`}
        content={content}
        format={document.format}
        initialScroll={document.scroll}
        onScrollFraction={onScrollFraction}
        pageRef={page}
        reader={withMenusClosed}
        actions={actions}
      />
    </div>
  )
}
