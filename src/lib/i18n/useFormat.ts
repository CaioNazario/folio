import { useTranslation } from 'react-i18next'
import { formatDateTime, formatNumber, formatPercent, formatRelativeTime, formatSize } from './format'

// Formatters bound to the active language; re-render on language change.
export function useFormat() {
  const { i18n } = useTranslation()
  const language = i18n.language === 'en' ? 'en' : 'pt-BR'
  return {
    number: (value: number) => formatNumber(value, language),
    percent: (ratio: number) => formatPercent(ratio, language),
    size: (bytes: number) => formatSize(bytes, language),
    relativeTime: (date: Date, now: Date) => formatRelativeTime(date, now, language),
    dateTime: (date: Date) => formatDateTime(date, language),
  }
}
