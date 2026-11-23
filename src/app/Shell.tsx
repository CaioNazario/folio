import { useCallback, useMemo, useState } from 'react'
import { ToastViewport } from '@/components/ui/toast'
import { ProtectedConvertDialog } from '@/features/conversion/components/ProtectedConvertDialog'
import { useConversions } from '@/features/conversion/useConversions'
import { useConvertAction } from '@/features/conversion/useConvertAction'
import { DocumentView } from '@/features/document/components/DocumentView'
import { PropertiesDialog } from '@/features/document/components/PropertiesDialog'
import { useProperties } from '@/features/document/properties/useProperties'
import type { DocumentActions } from '@/features/document/types'
import { useDocument } from '@/features/document/useDocument'
import { useLinkHandler } from '@/features/document/useLinkHandler'
import { useReaderState } from '@/features/document/useReaderState'
import { Home } from '@/features/home/components/Home'
import type { HomeActions } from '@/features/home/types'
import { Sidebar } from '@/features/sidebar/components/Sidebar'
import type { DocumentRef } from '@/features/sidebar/types'
import type { ConversionLink } from '@/lib/bindings/ConversionLink'
import type { Document } from '@/lib/bindings/Document'
import { useSidebar } from '@/features/sidebar/useSidebar'

const noop = () => undefined

// Menu actions arrive with their commands (specs 04 and 10).
const PENDING_ACTIONS: Omit<DocumentActions, 'onRetry' | 'onLink' | 'onPassword' | 'onProperties' | 'onConvert'> = {
  onRename: noop,
  onShowInFolder: noop,
  onCopyFile: noop,
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
  const { document, content, retry, submitPassword, password } = useDocument(openRef, events)
  // A finished conversion opens the derivative, which the sidebar now lists.
  const openDerivative = useCallback(
    ({ id, name }: Document) => {
      setOpenRef({ id, name })
      reload()
    },
    [reload],
  )
  const conversion = useConvertAction(document, useConversions(openDerivative))
  const reader = useReaderState()
  const onLink = useLinkHandler(openRef?.id ?? null, setOpenRef)
  const properties = useProperties(openRef?.id ?? null)
  const openLinked = ({ document: linked }: ConversionLink) => {
    properties.close()
    setOpenRef({ id: linked.id, name: linked.name })
  }
  const now = new Date()
  const actions = { onHome: () => setOpenRef(null), onImport: noop, onOpenDocument: setOpenRef, onTogglePin: sidebar.togglePin }
  const homeActions = { ...PENDING_HOME_ACTIONS, onOpenDocument: setOpenRef, onToggleProject: sidebar.tree.toggle }
  const homeData = { recents: sidebar.data.recents, projects: sidebar.tree.root.folders }
  return (
    <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
      <Sidebar data={sidebar.data} tree={sidebar.tree} actions={actions} selectedId={openRef?.id ?? null} now={now} />
      <main className="relative flex min-w-0 flex-1 flex-col">
        {document ? (
          <DocumentView document={document} content={content} reader={reader} actions={{ ...PENDING_ACTIONS, onConvert: conversion.onConvert, onRetry: retry, onLink, onPassword: submitPassword, onProperties: properties.open }} />
        ) : (
          <Home data={homeData} actions={homeActions} now={now} />
        )}
        <ProtectedConvertDialog open={conversion.dialogOpen} knownPassword={password} onConfirm={conversion.confirm} onClose={conversion.close} />
        <PropertiesDialog properties={properties.properties} onClose={properties.close} onOpenDocument={openLinked} />
        <ToastViewport />
      </main>
    </div>
  )
}
