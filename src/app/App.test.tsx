import { emit } from '@tauri-apps/api/event'
import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import { App } from './App'
import { workspaceCommands } from './fixtures'

async function renderApp(overrides: Parameters<typeof workspaceCommands>[0] = {}) {
  const calls = mockCommands(workspaceCommands(overrides))
  render(<App preferences={DEFAULT_PREFERENCES} />)
  await screen.findByText('1.284 arquivos · 3,2 GB')
  return { calls, user: userEvent.setup() }
}

const section = (name: string) => screen.getByRole('region', { name })
const projects = () => section(pt.sidebar.projects)

test('loads the tree, favorites, recents and footer of the open workspace', async () => {
  await renderApp()

  expect(within(projects()).getByRole('button', { name: 'Faculdade' })).toBeInTheDocument()
  expect(within(projects()).getByRole('button', { name: 'Leia-me.txt' })).toBeInTheDocument()
  expect(within(section(pt.sidebar.favorites.title)).getByRole('button', { name: 'Faculdade' })).toBeInTheDocument()
  expect(within(section(pt.sidebar.recents)).getByText('há 2 d')).toBeInTheDocument()
  expect(screen.getByText('~/Documentos/Folio')).toBeInTheDocument()
})

test('a folder loads its documents when opened', async () => {
  const { calls, user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: 'Faculdade' }))

  expect(await within(projects()).findByRole('button', { name: 'Referências.md' })).toBeInTheDocument()
  expect(calls).toContainEqual({ cmd: 'list_dir', args: { parent: [70] } })
})

test('a pinned folder opens its project in the tree', async () => {
  const { user } = await renderApp()
  await user.click(within(section(pt.sidebar.favorites.title)).getByRole('button', { name: 'Faculdade' }))

  expect(within(projects()).getByRole('button', { name: 'Faculdade' })).toHaveAttribute('aria-expanded', 'true')
})

test('pinning a document toggles it and reloads the favorites', async () => {
  const { calls, user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: pt.sidebar.pin.add }))

  expect(calls).toContainEqual({ cmd: 'toggle_favorite_document', args: { id: 'd-readme' } })
  expect(calls.filter((call) => call.cmd === 'get_favorites')).toHaveLength(2)
})

test('opening a text document shows it and records the opening', async () => {
  const { calls, user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: 'Leia-me.txt' }))

  expect(await screen.findByText('Conteúdo do trabalho')).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Leia-me.txt' })).toBeInTheDocument()
  expect(calls).toContainEqual({ cmd: 'record_opened', args: { id: 'd-readme' } })
})

test('a document gone from disk shows the not found state', async () => {
  const { user } = await renderApp({
    get_text_content: () => {
      throw { kind: 'DocumentNotFound', id: 'd-readme' }
    },
  })
  await user.click(within(projects()).getByRole('button', { name: 'Leia-me.txt' }))

  expect(await screen.findByText(pt.errors.DocumentNotFound)).toBeInTheDocument()
})

test('a format without a view yet says so', async () => {
  const { user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: 'Faculdade' }))
  await user.click(await screen.findByRole('button', { name: 'Referências.md' }))

  expect(screen.getByText('A exibição de MD ainda não está disponível')).toBeInTheDocument()
})

test('reloads the sidebar when a scan finishes', async () => {
  const { calls } = await renderApp()
  const before = calls.filter((call) => call.cmd === 'get_workspace_summary').length
  await act(() => emit('scan-finished'))

  expect(calls.filter((call) => call.cmd === 'get_workspace_summary')).toHaveLength(before + 1)
})
