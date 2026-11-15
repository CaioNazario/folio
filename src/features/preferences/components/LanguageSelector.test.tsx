import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import i18n from 'i18next'
import { afterEach, expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { LanguageSelector } from './LanguageSelector'

afterEach(() => i18n.changeLanguage('pt-BR'))

test('choosing English switches the interface live', async () => {
  const user = userEvent.setup({ pointerEventsCheck: 0 })
  render(<LanguageSelector />)
  await user.click(screen.getByRole('button', { name: pt.preferences.language.label }))
  await user.click(screen.getByRole('menuitem', { name: pt.preferences.language.en }))

  expect(i18n.language).toBe('en')
})
