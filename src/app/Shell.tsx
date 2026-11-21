import { useMemo, useState } from 'react'
import { ToastViewport } from '@/components/ui/toast'
import { DocumentView } from '@/features/document/components/DocumentView'
import type { DocumentActions } from '@/features/document/types'
import { useDocument } from '@/features/document/useDocument'
import { useLinkHandler } from '@/features/document/useLinkHandler'
import { useReaderState } from '@/features/document/useReaderState'
import { Home } from '@/features/home/components/Home'
import type { HomeActions } from '@/features/home/types'
import { Sidebar } from '@/features/sidebar/components/Sidebar'
import type { DocumentRef } from '@/features/sidebar/types'
import { useSidebar } from '@/features/sidebar/useSidebar'

const noop = () => undefined

// Menu actions arrive with their commands (specs 04, 08 and 10).
const PENDING_ACTIONS: Omit<DocumentActions, 'onRetry' | 'onLink' | 'onPassword'> = {
  onConvert: noop,
  onRename: noop,
  onShowInFolder: noop,
  onCopyFile: noop,
  onProperties: noop,
  onTrash: noop,
}

// Import, conversion, new project and the full recents list arrive with
// their commands (specs 03, 06, 08 and 09).
const PENDING_HOME_ACTIONS: Omit<HomeActions, 'onOpenDocument' | 'onToggleProject'> = {
  onImport: noop,
  onConvert: noop,
  onNewProject: noop,
  onSeeAllRecents: noop,
}

// Window shell (Design/specs/02-layout.md): sidebar + main area, which
// anchors the toast over every view. The main area shows the open document,
// or the Home when there is none.
export function Shell() {
  const sidebar = useSidebar()
  const [openRef, setOpenRef] = useState<DocumentRef | null>(null)
  const { reload, reloadRecents } = sidebar
  const events = useMemo(() => ({ onRecorded: reloadRecents, onNotFound: reload }), [reloadRecents, reload])
  const { document, content, retry, submitPassword } = useDocument(openRef, events)
  const reader = useReaderState()
  const onLink = useLinkHandler(openRef?.id ?? null, setOpenRef)
  const now = new Date()
  const actions = { onHome: () => setOpenRef(null), onImport: noop, onOpenDocument: setOpenRef, onTogglePin: sidebar.togglePin }
  const homeActions = { ...PENDING_HOME_ACTIONS, onOpenDocument: setOpenRef, onToggleProject: sidebar.tree.toggle }
  const homeData = { recents: sidebar.data.recents, projects: sidebar.tree.root.folders }
  return (
    <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
      <Sidebar data={sidebar.data} tree={sidebar.tree} actions={actions} selectedId={openRef?.id ?? null} now={now} />
      <main className="relative flex min-w-0 flex-1 flex-col">
        {document ? (
          <DocumentView document={document} content={content} reader={reader} actions={{ ...PENDING_ACTIONS, onRetry: retry, onLink, onPassword: submitPassword }} />
        ) : (
          <Home data={homeData} actions={homeActions} now={now} />
        )}
        <ToastViewport />
      </main>
    </div>
  )
}
