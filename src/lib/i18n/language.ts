export type Language = 'pt-BR' | 'en'

export type LanguagePreference = 'system' | Language

const LANGUAGES: readonly string[] = ['pt-BR', 'en']

// Unknown or missing stored values fall back to following the system.
export function parsePreference(value: string | null | undefined): LanguagePreference {
  if (value && LANGUAGES.includes(value)) return value as Language
  return 'system'
}

// Any Portuguese variant maps to pt-BR; everything else to English.
export function languageFromLocale(locale: string | null): Language {
  if (locale?.toLowerCase().split(/[-_]/)[0] === 'pt') return 'pt-BR'
  return 'en'
}

export function resolveLanguage(preference: LanguagePreference, systemLocale: string | null): Language {
  if (preference !== 'system') return preference
  return languageFromLocale(systemLocale)
}
