import { ToastProvider, ToastViewport } from '@/components/ui/toast'
import { DocumentView } from '@/features/document/components/DocumentView'
import { useDocument } from '@/features/document/useDocument'
import { useReaderState } from '@/features/document/useReaderState'
import { PreferencesProvider } from '@/features/preferences/components/PreferencesProvider'
import { Sidebar } from '@/features/sidebar/components/Sidebar'
import { useSidebar } from '@/features/sidebar/useSidebar'
import type { Preferences } from '@/lib/bindings/Preferences'

// Window shell (Design/specs/02-layout.md): sidebar + main area, which anchors
// the toast over every view.
export function App({ preferences }: { preferences: Preferences }) {
  const sidebar = useSidebar()
  const { document, content, actions } = useDocument()
  const reader = useReaderState()
  return (
    <ToastProvider>
      <PreferencesProvider initial={preferences}>
        <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
          <Sidebar {...sidebar} now={new Date()} />
          <main className="relative flex min-w-0 flex-1 flex-col">
            {document && <DocumentView document={document} content={content} reader={reader} actions={actions} />}
            <ToastViewport />
          </main>
        </div>
      </PreferencesProvider>
    </ToastProvider>
  )
}
