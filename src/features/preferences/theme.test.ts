import { expect, test } from 'vitest'
import { resolveTheme } from './theme'

test('a fixed theme ignores the system', () => {
  expect(resolveTheme('light', true)).toBe('light')
})

test.each([
  [true, 'dark'],
  [false, 'light'],
] as const)('system with dark=%s resolves to %s', (systemDark, theme) => {
  expect(resolveTheme('system', systemDark)).toBe(theme)
})
