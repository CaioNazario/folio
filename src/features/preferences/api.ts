import { invoke } from '@tauri-apps/api/core'
import type { Language } from '@/lib/bindings/Language'
import type { Preferences } from '@/lib/bindings/Preferences'
import type { Theme } from '@/lib/bindings/Theme'

export const DEFAULT_PREFERENCES: Preferences = { language: 'system', theme: 'dark' }

export function getPreferences(): Promise<Preferences> {
  return invoke<Preferences>('get_preferences')
}

// Outside Tauri (plain browser) there is no backend: use the defaults.
export async function loadPreferences(): Promise<Preferences> {
  try {
    return await getPreferences()
  } catch {
    return DEFAULT_PREFERENCES
  }
}

export function saveLanguage(language: Language): Promise<void> {
  return invoke('set_language', { language })
}

export function saveTheme(theme: Theme): Promise<void> {
  return invoke('set_theme', { theme })
}
