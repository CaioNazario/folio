import { createContext, useContext } from 'react'
import type { Preferences } from '@/lib/bindings/Preferences'

export type PreferencesContextValue = Preferences & {
  setTheme: (theme: Preferences['theme']) => void
  setLanguage: (language: Preferences['language']) => void
}

export const PreferencesContext = createContext<PreferencesContextValue | null>(null)

export function usePreferences(): PreferencesContextValue {
  const context = useContext(PreferencesContext)
  if (!context) throw new Error('usePreferences must be used inside <PreferencesProvider>')
  return context
}
