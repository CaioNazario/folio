import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import i18n from 'i18next'
import { afterEach, expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { mockCommands, renderWithProviders } from '@/test-utils'
import { LanguageSelector } from './LanguageSelector'

afterEach(() => i18n.changeLanguage('pt-BR'))

async function chooseEnglish() {
  const user = userEvent.setup({ pointerEventsCheck: 0 })
  renderWithProviders(<LanguageSelector />)
  await user.click(screen.getByRole('button', { name: pt.preferences.language.label }))
  await user.click(screen.getByRole('menuitem', { name: pt.preferences.language.en }))
}

test('choosing English switches the interface live', async () => {
  mockCommands()
  await chooseEnglish()

  expect(i18n.language).toBe('en')
})

test('choosing a language saves it', async () => {
  const calls = mockCommands()
  await chooseEnglish()

  expect(calls).toContainEqual({ cmd: 'set_language', args: { language: 'en' } })
})
