import { describe, expect, test } from 'vitest'
import { formatDateTime, formatRelativeTime, formatSize } from './format'

const NOW = new Date(2026, 2, 20, 12, 0)
const ago = (seconds: number) => new Date(NOW.getTime() - seconds * 1000)

describe('formatSize', () => {
  test.each([
    [42 * 1024, '42 KB', '42 KB'],
    [310 * 1024, '310 KB', '310 KB'],
    [4.1 * 1024 ** 2, '4,1 MB', '4.1 MB'],
    [3.2 * 1024 ** 3, '3,2 GB', '3.2 GB'],
    [512, '512 B', '512 B'],
  ])('%d bytes', (bytes, ptText, enText) => {
    expect(formatSize(bytes, 'pt-BR')).toBe(ptText)
    expect(formatSize(bytes, 'en')).toBe(enText)
  })
})

describe('formatRelativeTime', () => {
  test.each([
    [2 * 86400, 'há 2 d', '2d ago'],
    [11 * 86400, 'há 11 d', '11d ago'],
    [3 * 3600, 'há 3 h', '3h ago'],
    [5 * 60, 'há 5 min', '5m ago'],
    [10, 'agora', 'now'],
    [70 * 86400, 'há 2 meses', '2mo ago'],
  ])('%d seconds ago', (seconds, ptText, enText) => {
    expect(formatRelativeTime(ago(seconds), NOW, 'pt-BR')).toBe(ptText)
    expect(formatRelativeTime(ago(seconds), NOW, 'en')).toBe(enText)
  })
})

describe('formatDateTime', () => {
  test('formats day, short month and 24h time', () => {
    const date = new Date(2026, 2, 18, 21, 30)

    expect(formatDateTime(date, 'pt-BR')).toBe('18 mar, 21:30')
    expect(formatDateTime(date, 'en')).toBe('Mar 18, 21:30')
  })
})
