import i18n from 'i18next'
import { afterEach, expect, test } from 'vitest'
import { en } from './en'
import { pt } from './pt'

function leaves(node: object): string[] {
  return Object.values(node).flatMap((value) => (typeof value === 'string' ? [value] : leaves(value)))
}

afterEach(() => i18n.changeLanguage('pt-BR'))

test.each([['pt', pt], ['en', en]])('%s has no empty value', (_, resources) => {
  expect(leaves(resources).every((value) => value.trim() !== '')).toBe(true)
})

test('pluralizes and formats counts in Portuguese', () => {
  expect(i18n.t('common.files', { count: 1284 })).toBe('1.284 arquivos')
  expect(i18n.t('common.files', { count: 1 })).toBe('1 arquivo')
})

test('pluralizes and formats counts in English', async () => {
  await i18n.changeLanguage('en')

  expect(i18n.t('common.files', { count: 1284 })).toBe('1,284 files')
})

test('keeps the html lang attribute in sync with the language', async () => {
  await i18n.changeLanguage('en')

  expect(document.documentElement.lang).toBe('en')
})
