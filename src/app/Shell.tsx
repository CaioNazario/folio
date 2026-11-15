import { useState } from 'react'
import { ToastViewport } from '@/components/ui/toast'
import { DocumentView } from '@/features/document/components/DocumentView'
import type { DocumentActions } from '@/features/document/types'
import { useDocument } from '@/features/document/useDocument'
import { useReaderState } from '@/features/document/useReaderState'
import { Sidebar } from '@/features/sidebar/components/Sidebar'
import type { DocumentRef } from '@/features/sidebar/types'
import { useSidebar } from '@/features/sidebar/useSidebar'

const noop = () => undefined

// Menu actions arrive with their commands (specs 04, 08 and 10).
const PENDING_ACTIONS: Omit<DocumentActions, 'onRetry'> = {
  onConvert: noop,
  onRename: noop,
  onShowInFolder: noop,
  onCopyFile: noop,
  onProperties: noop,
  onTrash: noop,
  onLink: noop,
}

// Window shell (Design/specs/02-layout.md): sidebar + main area, which
// anchors the toast over every view.
export function Shell() {
  const sidebar = useSidebar()
  const [openRef, setOpenRef] = useState<DocumentRef | null>(null)
  const { document, content, retry } = useDocument(openRef, sidebar.reloadRecents)
  const reader = useReaderState()
  const actions = { onImport: noop, onOpenDocument: setOpenRef, onTogglePin: sidebar.togglePin }
  return (
    <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
      <Sidebar data={sidebar.data} tree={sidebar.tree} actions={actions} selectedId={openRef?.id ?? null} now={new Date()} />
      <main className="relative flex min-w-0 flex-1 flex-col">
        {document && (
          <DocumentView document={document} content={content} reader={reader} actions={{ ...PENDING_ACTIONS, onRetry: retry }} />
        )}
        <ToastViewport />
      </main>
    </div>
  )
}
