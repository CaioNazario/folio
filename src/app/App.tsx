import { Sidebar } from '@/features/sidebar/components/Sidebar'

// Window shell (Design/specs/02-layout.md): sidebar + main area.
export function App() {
  return (
    <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
      <Sidebar />
      <main className="relative flex min-w-0 flex-1 flex-col" />
    </div>
  )
}
