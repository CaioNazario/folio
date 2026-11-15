import { useTranslation } from 'react-i18next'
import { formatDateTime, formatRelativeTime, formatSize } from './format'

// Formatters bound to the active language; re-render on language change.
export function useFormat() {
  const { i18n } = useTranslation()
  const language = i18n.language === 'en' ? 'en' : 'pt-BR'
  return {
    size: (bytes: number) => formatSize(bytes, language),
    relativeTime: (date: Date, now: Date) => formatRelativeTime(date, now, language),
    dateTime: (date: Date) => formatDateTime(date, language),
  }
}
