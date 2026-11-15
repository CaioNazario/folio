import { render, screen } from '@testing-library/react'
import { expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { App } from './App'

test('renders the sidebar and the main area', () => {
  render(<App />)

  expect(screen.getByRole('complementary', { name: pt.shell.sidebar })).toBeInTheDocument()
  expect(screen.getByRole('main')).toBeInTheDocument()
})

test('shows the logo in the sidebar header', () => {
  render(<App />)

  expect(screen.getByRole('img', { name: pt.app.name })).toBeInTheDocument()
})
