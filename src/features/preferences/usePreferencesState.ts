import { useEffect, useState } from 'react'
import { useToast } from '@/components/ui/toast-context'
import type { Preferences } from '@/lib/bindings/Preferences'
import { applyLanguagePreference } from '@/lib/i18n'
import { errorMessage, isBridgeError } from '@/lib/i18n/errors'
import { saveLanguage, saveTheme } from './api'
import type { PreferencesContextValue } from './preferences-context'
import { applyTheme, resolveTheme } from './theme'
import { useSystemDark } from './useSystemDark'

// Applies a change at once and saves it; a failed save shows the backend
// error in a toast and keeps the choice for this session.
export function usePreferencesState(initial: Preferences): PreferencesContextValue {
  const [preferences, setPreferences] = useState(initial)
  const { show } = useToast()
  const resolved = resolveTheme(preferences.theme, useSystemDark())
  useEffect(() => applyTheme(resolved), [resolved])

  const report = (error: unknown) => (isBridgeError(error) ? show(errorMessage(error)) : console.error(error))
  const setTheme = (theme: Preferences['theme']) => {
    setPreferences((current) => ({ ...current, theme }))
    saveTheme(theme).catch(report)
  }
  const setLanguage = (language: Preferences['language']) => {
    setPreferences((current) => ({ ...current, language }))
    void applyLanguagePreference(language)
    saveLanguage(language).catch(report)
  }
  return { ...preferences, setTheme, setLanguage }
}
