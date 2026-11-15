import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import { en } from './en'
import { type Language, type LanguagePreference, resolveLanguage } from './language'
import { pt } from './pt'
import { systemLocale } from './systemLocale'

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation'
    resources: { translation: typeof pt }
  }
}

export const resources = { 'pt-BR': { translation: pt }, en: { translation: en } }

// Starts i18next synchronously in the given language and keeps <html lang>
// in sync with every later change.
export function initI18n(language: Language) {
  i18n.on('languageChanged', (lng) => document.documentElement.setAttribute('lang', lng))
  void i18n.use(initReactI18next).init({
    resources,
    lng: language,
    fallbackLng: 'pt-BR',
    initAsync: false,
    interpolation: { escapeValue: false },
  })
  return i18n
}

export async function languageForPreference(preference: LanguagePreference): Promise<Language> {
  if (preference !== 'system') return preference
  return resolveLanguage('system', await systemLocale())
}

export async function applyLanguagePreference(preference: LanguagePreference) {
  await i18n.changeLanguage(await languageForPreference(preference))
}

export function activeLanguage(): Language {
  return i18n.language === 'en' ? 'en' : 'pt-BR'
}
