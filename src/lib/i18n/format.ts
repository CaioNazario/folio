import type { Language } from './language'

// Number, size and date formatting for the active language
// (Design/specs/11-i18n.md, "O que é traduzido").

const SIZE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB']

export function formatSize(bytes: number, language: Language): string {
  const exponent = bytes < 1024 ? 0 : Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), SIZE_UNITS.length - 1)
  const value = bytes / 1024 ** exponent
  const digits = exponent > 0 && value < 10 ? 1 : 0
  const number = new Intl.NumberFormat(language, { maximumFractionDigits: digits }).format(value)
  return `${number} ${SIZE_UNITS[exponent]}`
}

const MINUTE = 60
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR
const MONTH = 30 * DAY
const YEAR = 365 * DAY

const RELATIVE_STEPS: [limit: number, unit: Intl.RelativeTimeFormatUnit, size: number][] = [
  [HOUR, 'minute', MINUTE],
  [DAY, 'hour', HOUR],
  [MONTH, 'day', DAY],
  [YEAR, 'month', MONTH],
  [Infinity, 'year', YEAR],
]

export function formatRelativeTime(date: Date, now: Date, language: Language): string {
  const seconds = Math.max(0, (now.getTime() - date.getTime()) / 1000)
  if (seconds < MINUTE) {
    return new Intl.RelativeTimeFormat(language, { style: 'narrow', numeric: 'auto' }).format(0, 'second')
  }
  const [, unit, size] = RELATIVE_STEPS.find(([limit]) => seconds < limit) ?? RELATIVE_STEPS[4]
  const text = new Intl.RelativeTimeFormat(language, { style: 'narrow' }).format(-Math.floor(seconds / size), unit)
  return language === 'pt-BR' ? adjustPortugueseRelative(text) : text
}

// The design writes "há 2 d" where ICU gives "há 2 dias", and drops the
// abbreviation dot ("há 5 min.").
function adjustPortugueseRelative(text: string): string {
  return text.replace(/ dias?$/, ' d').replace(/\.$/, '')
}

const DATE_TIME_OPTIONS: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
  hour: '2-digit',
  minute: '2-digit',
  hourCycle: 'h23',
}

export function formatDateTime(date: Date, language: Language): string {
  const formatter = new Intl.DateTimeFormat(language, DATE_TIME_OPTIONS)
  if (language === 'en') return formatter.format(date)
  // ICU gives "18 de mar., 21:30"; the design wants "18 mar, 21:30".
  const parts = Object.fromEntries(formatter.formatToParts(date).map((part) => [part.type, part.value]))
  return `${parts.day} ${parts.month.replace('.', '')}, ${parts.hour}:${parts.minute}`
}
