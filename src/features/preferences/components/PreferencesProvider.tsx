import type { ReactNode } from 'react'
import type { Preferences } from '@/lib/bindings/Preferences'
import { PreferencesContext } from '../preferences-context'
import { usePreferencesState } from '../usePreferencesState'

// Theme and language for the whole app, loaded at startup (src/main.tsx).
export function PreferencesProvider({ initial, children }: { initial: Preferences; children: ReactNode }) {
  const value = usePreferencesState(initial)
  return <PreferencesContext.Provider value={value}>{children}</PreferencesContext.Provider>
}
