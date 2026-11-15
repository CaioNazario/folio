import { useState } from 'react'
import { applyLanguagePreference } from '@/lib/i18n'
import type { LanguagePreference } from '@/lib/i18n/language'

// Holds the language preference and switches the interface live.
// Persistence comes with the preferences command.
export function useLanguagePreference() {
  const [preference, setPreference] = useState<LanguagePreference>('system')

  const choose = (next: LanguagePreference) => {
    setPreference(next)
    void applyLanguagePreference(next)
  }

  return { preference, choose }
}
