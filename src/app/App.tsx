import { ToastProvider, ToastViewport } from '@/components/ui/toast'
import { Sidebar } from '@/features/sidebar/components/Sidebar'
import { useSidebar } from '@/features/sidebar/useSidebar'

// Window shell (Design/specs/02-layout.md): sidebar + main area, which anchors
// the toast over every view.
export function App() {
  const sidebar = useSidebar()
  return (
    <ToastProvider>
      <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
        <Sidebar {...sidebar} now={new Date()} />
        <main className="relative flex min-w-0 flex-1 flex-col">
          <ToastViewport />
        </main>
      </div>
    </ToastProvider>
  )
}
