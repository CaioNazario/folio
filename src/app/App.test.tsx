import { render, screen } from '@testing-library/react'
import { beforeEach, expect, test } from 'vitest'
import { mockCommands } from '@/test-utils'
import { pt } from '@/lib/i18n/pt'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import { App } from './App'

beforeEach(() => {
  mockCommands()
})

test('renders the sidebar and the main area', () => {
  render(<App preferences={DEFAULT_PREFERENCES} />)

  expect(screen.getByRole('complementary', { name: pt.shell.sidebar })).toBeInTheDocument()
  expect(screen.getByRole('main')).toBeInTheDocument()
})

test('shows the logo in the sidebar header', () => {
  render(<App preferences={DEFAULT_PREFERENCES} />)

  expect(screen.getByRole('img', { name: pt.app.name })).toBeInTheDocument()
})
