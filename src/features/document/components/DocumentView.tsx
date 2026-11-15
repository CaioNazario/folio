import { useState } from 'react'
import type { DocumentActions, OpenDocument, StageContent } from '../types'
import type { ReaderState } from '../useReaderState'
import { useReaderShortcuts } from '../useReaderShortcuts'
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
  useReaderShortcuts(withMenusClosed, openMenu !== null)
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <DocumentHeader document={document} actions={actions} openMenu={openMenu} onOpenMenu={setOpenMenu} />
      <Stage content={content} format={document.format} reader={withMenusClosed} actions={actions} />
    </div>
  )
}
