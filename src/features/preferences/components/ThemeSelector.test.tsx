import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { ThemeSelector } from './ThemeSelector'

test('starts dark', () => {
  render(<ThemeSelector />)

  expect(document.documentElement.dataset.theme).toBe('dark')
})

test('choosing light applies it to the root', async () => {
  render(<ThemeSelector />)
  await userEvent.click(screen.getByRole('radio', { name: pt.preferences.theme.light }))

  expect(document.documentElement.dataset.theme).toBe('light')
  expect(screen.getByRole('radio', { name: pt.preferences.theme.light })).toHaveAttribute('aria-checked', 'true')
})
