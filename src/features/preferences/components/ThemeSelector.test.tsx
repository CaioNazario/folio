import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { mockCommands, renderWithProviders } from '@/test-utils'
import { ThemeSelector } from './ThemeSelector'

test('starts with the stored theme', () => {
  mockCommands()
  renderWithProviders(<ThemeSelector />)

  expect(document.documentElement.dataset.theme).toBe('dark')
})

test('choosing light applies it to the root', async () => {
  mockCommands()
  renderWithProviders(<ThemeSelector />)
  await userEvent.click(screen.getByRole('radio', { name: pt.preferences.theme.light }))

  expect(document.documentElement.dataset.theme).toBe('light')
  expect(screen.getByRole('radio', { name: pt.preferences.theme.light })).toHaveAttribute('aria-checked', 'true')
})

test('choosing a theme saves it', async () => {
  const calls = mockCommands()
  renderWithProviders(<ThemeSelector />)
  await userEvent.click(screen.getByRole('radio', { name: pt.preferences.theme.system }))

  expect(calls).toContainEqual({ cmd: 'set_theme', args: { theme: 'system' } })
})

test('a failed save shows the backend error', async () => {
  mockCommands({
    set_theme: () => {
      throw { kind: 'Database', path: '/home/u/.local/share/folio/folio.db' }
    },
  })
  renderWithProviders(<ThemeSelector />)
  await userEvent.click(screen.getByRole('radio', { name: pt.preferences.theme.light }))

  expect(await screen.findByText(/Não foi possível acessar o banco de dados/)).toBeInTheDocument()
})
