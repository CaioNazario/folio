import { ToastProvider } from '@/components/ui/toast'
import { PreferencesProvider } from '@/features/preferences/components/PreferencesProvider'
import { WorkspaceGate } from '@/features/workspace/components/WorkspaceGate'
import type { Preferences } from '@/lib/bindings/Preferences'
import { Shell } from './Shell'

// Providers, then the workspace gate: the shell only shows with a workspace open.
export function App({ preferences }: { preferences: Preferences }) {
  return (
    <ToastProvider>
      <PreferencesProvider initial={preferences}>
        <WorkspaceGate>
          <Shell />
        </WorkspaceGate>
      </PreferencesProvider>
    </ToastProvider>
  )
}
