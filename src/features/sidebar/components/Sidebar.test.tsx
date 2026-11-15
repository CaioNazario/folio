import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test, vi } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { renderWithProviders } from '@/test-utils'
import type { SidebarData } from '../types'
import { Sidebar } from './Sidebar'

const NOW = new Date(2026, 2, 20, 12, 0)
const DAY = 86_400_000

const DATA: SidebarData = {
  favorites: [
    { id: 'f-v3', name: 'Trabalho v3.docx', kind: 'file' },
    { id: 'p-fac', name: 'Faculdade', kind: 'folder' },
  ],
  projects: [
    {
      id: 'p-fac',
      name: 'Faculdade',
      count: 2,
      files: [
        { id: 'f-v3', name: 'Trabalho v3.docx' },
        { id: 'f-ref', name: 'Referências.md' },
      ],
    },
    { id: 'p-tra', name: 'Trabalho', count: 1, files: [{ id: 'f-rel', name: 'Relatório Q1.pdf' }] },
  ],
  recents: [{ id: 'f-pdf', name: 'Trabalho v2.pdf', openedAt: new Date(NOW.getTime() - 2 * DAY) }],
  workspace: { path: '~/Documentos', userName: 'Caio', fileCount: 1284, totalBytes: 3.2 * 1024 ** 3 },
}

function renderSidebar(data: SidebarData = DATA, selectedId: string | null = null) {
  const actions = { onImport: vi.fn(), onOpenDocument: vi.fn(), onTogglePin: vi.fn() }
  renderWithProviders(<Sidebar data={data} actions={actions} selectedId={selectedId} now={NOW} />)
  return { actions, user: userEvent.setup() }
}

const section = (name: string) => screen.getByRole('region', { name })

test('shows the empty favorites text', () => {
  renderSidebar({ ...DATA, favorites: [] })

  expect(screen.getByText(pt.sidebar.favorites.empty)).toBeInTheDocument()
})

test('opening a favorite file opens the document', async () => {
  const { actions, user } = renderSidebar()
  await user.click(within(section(pt.sidebar.favorites.title)).getByRole('button', { name: 'Trabalho v3.docx' }))

  expect(actions.onOpenDocument).toHaveBeenCalledWith('f-v3')
})

test('projects start closed and expand on click', async () => {
  const { user } = renderSidebar()
  const projects = section(pt.sidebar.projects)
  expect(within(projects).queryByRole('button', { name: 'Referências.md' })).not.toBeInTheDocument()

  await user.click(within(projects).getByRole('button', { name: 'Faculdade' }))

  expect(within(projects).getByRole('button', { name: 'Referências.md' })).toBeInTheDocument()
})

test('a pinned folder toggles its project in the tree', async () => {
  const { user } = renderSidebar()
  await user.click(within(section(pt.sidebar.favorites.title)).getByRole('button', { name: 'Faculdade' }))

  expect(within(section(pt.sidebar.projects)).getByRole('button', { name: 'Faculdade' })).toHaveAttribute('aria-expanded', 'true')
})

test('the pin toggles the favorite without opening the row', async () => {
  const { actions, user } = renderSidebar()
  const recents = section(pt.sidebar.recents)
  await user.click(within(recents).getByRole('button', { name: pt.sidebar.pin.add }))

  expect(actions.onTogglePin).toHaveBeenCalledWith('f-pdf', 'file')
  expect(actions.onOpenDocument).not.toHaveBeenCalled()
})

test('marks the selected file in the tree', async () => {
  const { user } = renderSidebar(DATA, 'f-ref')
  await user.click(within(section(pt.sidebar.projects)).getByRole('button', { name: 'Faculdade' }))

  expect(screen.getByRole('button', { name: 'Referências.md' })).toHaveAttribute('aria-current', 'page')
})

test('shows the relative time of recents', () => {
  renderSidebar()

  expect(within(section(pt.sidebar.recents)).getByText('há 2 d')).toBeInTheDocument()
})

test('shows the workspace summary in the footer', () => {
  renderSidebar()

  expect(screen.getByText('1.284 arquivos · 3,2 GB')).toBeInTheDocument()
})
