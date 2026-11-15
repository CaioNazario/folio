import { useEffect, useState } from 'react'
import { resolveTheme, type ThemePreference } from './theme'
import { useSystemDark } from './useSystemDark'

// Holds the theme preference and applies the resolved theme to <html
// data-theme>. Persistence comes with the preferences command.
export function useThemePreference() {
  const [preference, setPreference] = useState<ThemePreference>('dark')
  const theme = resolveTheme(preference, useSystemDark())

  useEffect(() => {
    document.documentElement.dataset.theme = theme
  }, [theme])

  return { preference, setPreference }
}
