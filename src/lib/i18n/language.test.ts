import { expect, test } from 'vitest'
import { languageFromLocale, resolveLanguage } from './language'

test.each(['pt-BR', 'pt-PT', 'pt', 'pt_BR.UTF-8'])('maps %s to Portuguese', (locale) => {
  expect(languageFromLocale(locale)).toBe('pt-BR')
})

test.each(['en-US', 'es-ES', 'fr', null])('maps %s to English', (locale) => {
  expect(languageFromLocale(locale)).toBe('en')
})

test('a fixed preference wins over the system locale', () => {
  expect(resolveLanguage('en', 'pt-BR')).toBe('en')
})

test('the system preference follows the locale', () => {
  expect(resolveLanguage('system', 'pt-BR')).toBe('pt-BR')
})
