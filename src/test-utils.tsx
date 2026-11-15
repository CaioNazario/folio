import { mockIPC } from '@tauri-apps/api/mocks'
import { render } from '@testing-library/react'
import type { ReactElement } from 'react'
import { ToastProvider, ToastViewport } from '@/components/ui/toast'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import { PreferencesProvider } from '@/features/preferences/components/PreferencesProvider'

type Commands = Record<string, (args: unknown) => unknown>

// Records every invoke() and answers it from `commands` (null when absent).
export function mockCommands(commands: Commands = {}) {
  const calls: { cmd: string; args: unknown }[] = []
  mockIPC((cmd, args) => {
    calls.push({ cmd, args })
    return commands[cmd] ? commands[cmd](args) : null
  })
  return calls
}

// Renders inside the app providers, with the default preferences.
export function renderWithProviders(ui: ReactElement) {
  return render(
    <ToastProvider>
      <PreferencesProvider initial={DEFAULT_PREFERENCES}>
        {ui}
        <ToastViewport />
      </PreferencesProvider>
    </ToastProvider>,
  )
}
