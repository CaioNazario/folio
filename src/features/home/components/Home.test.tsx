import { render, screen } from '@testing-library/react'
import { expect, test, vi } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import type { HomeActions } from '../types'
import { Home } from './Home'

function actions(): HomeActions {
  const names = ['onImport', 'onConvert', 'onNewProject', 'onSeeAllRecents', 'onOpenDocument', 'onToggleProject']
  return Object.fromEntries(names.map((name) => [name, vi.fn()])) as unknown as HomeActions
}

test('an empty workspace shows the home without recents or projects', () => {
  render(<Home data={{ recents: [], projects: [] }} actions={actions()} now={new Date()} />)

  expect(screen.getByRole('heading', { name: pt.home.title, level: 1 })).toBeInTheDocument()
  expect(screen.queryByRole('list')).not.toBeInTheDocument()
  expect(screen.queryByRole('region', { name: pt.home.projects })).not.toBeInTheDocument()
})

test('a recent document at the workspace root shows only its size', () => {
  const recent = { id: 'd1', name: 'Leia-me.txt', project: null, size: 2048, openedAt: new Date() }
  render(<Home data={{ recents: [recent], projects: [] }} actions={actions()} now={new Date()} />)

  expect(screen.getByRole('button', { name: /Leia-me/ })).toHaveTextContent(/^Leia-me\.txt2 KB/)
})
