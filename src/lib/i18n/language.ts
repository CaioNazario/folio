import type { Language } from '@/lib/bindings/Language'

// The stored preference (ts-rs binding) and the language actually shown.
export type LanguagePreference = Language

export type ActiveLanguage = 'pt-BR' | 'en'

// Any Portuguese variant maps to pt-BR; everything else to English.
export function languageFromLocale(locale: string | null): ActiveLanguage {
  if (locale?.toLowerCase().split(/[-_]/)[0] === 'pt') return 'pt-BR'
  return 'en'
}

export function resolveLanguage(preference: LanguagePreference, systemLocale: string | null): ActiveLanguage {
  if (preference !== 'system') return preference
  return languageFromLocale(systemLocale)
}
