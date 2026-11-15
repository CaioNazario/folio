import { emit } from '@tauri-apps/api/event'
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test, vi } from 'vitest'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import { App } from './App'
import { ROOT, workspaceCommands } from './fixtures'

async function renderApp(overrides: Parameters<typeof workspaceCommands>[0] = {}) {
  const calls = mockCommands(workspaceCommands(overrides))
  render(<App preferences={DEFAULT_PREFERENCES} />)
  await screen.findByText('1.284 arquivos · 3,2 GB')
  return { calls, user: userEvent.setup() }
}

const sidebar = () => screen.getByRole('complementary', { name: pt.shell.sidebar })
const section = (name: string) => within(sidebar()).getByRole('region', { name })
const projects = () => section(pt.sidebar.projects)
const home = () => within(screen.getByRole('main'))

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

test('a document gone from disk reloads the sidebar', async () => {
  const { calls, user } = await renderApp({
    get_text_content: () => {
      throw { kind: 'DocumentNotFound', id: 'd-readme' }
    },
  })
  const count = (cmd: string) => calls.filter((call) => call.cmd === cmd).length
  const before = ['list_dir', 'get_favorites', 'get_recent_documents', 'get_workspace_summary'].map(count)
  await user.click(within(projects()).getByRole('button', { name: 'Leia-me.txt' }))
  await screen.findByText(pt.errors.DocumentNotFound)

  await waitFor(() => expect(['list_dir', 'get_favorites', 'get_recent_documents', 'get_workspace_summary'].map(count)).toEqual(before.map((n) => n + 1)))
})

async function openReferences(user: ReturnType<typeof userEvent.setup>) {
  await user.click(within(projects()).getByRole('button', { name: 'Faculdade' }))
  await user.click(await screen.findByRole('button', { name: 'Referências.md' }))
  return screen.findByRole('heading', { name: 'Referências', level: 1 })
}

test('a markdown document shows its html with the folder in the breadcrumb', async () => {
  const { user } = await renderApp()
  await openReferences(user)

  expect(screen.getByRole('navigation', { name: pt.document.breadcrumb })).toHaveTextContent('Faculdade/Referências.md')
})

test('an image the core could not show gets a label in the active language', async () => {
  const { user } = await renderApp()
  await openReferences(user)

  expect(screen.getByText(/diagrama/)).toHaveAttribute('data-label', pt.document.image.remote)
})

test('an external link opens in the system browser', async () => {
  const { calls, user } = await renderApp()
  await openReferences(user)
  await user.click(screen.getByRole('link', { name: 'site' }))

  await waitFor(() => expect(calls).toContainEqual(expect.objectContaining({ cmd: 'plugin:opener|open_url' })))
})

test('a link to another document opens it', async () => {
  const { user } = await renderApp()
  await openReferences(user)
  await user.click(screen.getByRole('link', { name: 'leia' }))

  expect(await screen.findByText('Conteúdo do trabalho')).toBeInTheDocument()
})

test('a link to a file Folio does not open shows a toast', async () => {
  const { user } = await renderApp()
  await openReferences(user)
  await user.click(screen.getByRole('link', { name: 'planilha' }))

  expect(await screen.findByText(pt.document.linkUnsupported)).toBeInTheDocument()
})

test('a format without a view yet says so', async () => {
  const { user } = await renderApp({ list_dir: () => ({ folders: [], documents: [{ ...ROOT.documents[0], id: 'd-pdf', name: 'Artigo.pdf', format: 'pdf' }] }) })
  await user.click(within(projects()).getByRole('button', { name: 'Artigo.pdf' }))

  expect(await screen.findByText('A exibição de PDF ainda não está disponível')).toBeInTheDocument()
})

test('saves the scroll position when another document opens', async () => {
  const { calls, user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: 'Leia-me.txt' }))
  fireEvent.scroll(await screen.findByRole('article'))
  await openReferences(user)

  expect(calls).toContainEqual({ cmd: 'save_position', args: { id: 'd-readme', position: { kind: 'Scroll', fraction: 0 } } })
})

test('reloads the sidebar when a scan finishes', async () => {
  const { calls } = await renderApp()
  const before = calls.filter((call) => call.cmd === 'get_workspace_summary').length
  await act(() => emit('scan-finished'))

  expect(calls.filter((call) => call.cmd === 'get_workspace_summary')).toHaveLength(before + 1)
})

test('shows the home when no document is open', async () => {
  await renderApp()

  expect(home().getByRole('heading', { name: pt.home.title, level: 1 })).toBeInTheDocument()
  expect(home().getByRole('button', { name: /Importar arquivos/ })).toBeInTheDocument()
})

test('the home lists the recent documents with project, size and time', async () => {
  await renderApp()
  const recent = home().getByRole('button', { name: /Trabalho v3\.txt/ })

  expect(recent).toHaveTextContent('Faculdade · 42 KB')
  expect(recent).toHaveTextContent('há 2 d')
})

test('opening a recent document from the home shows it in the reader', async () => {
  const { calls, user } = await renderApp()
  await user.click(home().getByRole('button', { name: /Trabalho v3\.txt/ }))

  expect(await screen.findByText('Conteúdo do trabalho')).toBeInTheDocument()
  expect(calls).toContainEqual({ cmd: 'open_document', args: { id: 'd-v3' } })
})

test('the home lists the projects with their document count', async () => {
  await renderApp()

  expect(home().getByRole('button', { name: /^Faculdade/ })).toHaveTextContent('2 documentos')
})

test('a project card opens the project in the sidebar and stays on the home', async () => {
  const { user } = await renderApp()
  await user.click(home().getByRole('button', { name: /^Faculdade/ }))

  expect(within(projects()).getByRole('button', { name: 'Faculdade' })).toHaveAttribute('aria-expanded', 'true')
  expect(home().getByRole('heading', { name: pt.home.title, level: 1 })).toBeInTheDocument()
})

test('the logo goes back to the home', async () => {
  const { user } = await renderApp()
  await user.click(within(projects()).getByRole('button', { name: 'Leia-me.txt' }))
  await screen.findByText('Conteúdo do trabalho')
  await user.click(screen.getByRole('button', { name: pt.sidebar.home }))

  expect(home().getByRole('heading', { name: pt.home.title, level: 1 })).toBeInTheDocument()
})

test('an anchor link scrolls the page to its heading', async () => {
  const { user } = await renderApp()
  await openReferences(user)
  const heading = screen.getByRole('heading', { name: 'Notas' })
  heading.scrollIntoView = vi.fn()
  await user.click(screen.getByRole('link', { name: 'notas' }))

  await waitFor(() => expect(heading.scrollIntoView).toHaveBeenCalled())
})

test('an anchor to a missing heading does nothing', async () => {
  const { calls, user } = await renderApp()
  await openReferences(user)
  await user.click(screen.getByRole('link', { name: 'nada' }))

  await waitFor(() => expect(calls).toContainEqual({ cmd: 'resolve_link', args: { id: 'd-ref', href: '#nada' } }))
  expect(screen.queryByText(pt.document.linkUnsupported)).not.toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Referências', level: 1 })).toBeInTheDocument()
})

test('an empty workspace says so in projects and recents', async () => {
  await renderApp({ list_dir: () => ({ folders: [], documents: [] }), get_recent_documents: () => [] })

  expect(within(projects()).getByText(pt.sidebar.projectsEmpty)).toBeInTheDocument()
  expect(within(section(pt.sidebar.recents)).getByText(pt.sidebar.recentsEmpty)).toBeInTheDocument()
})

test('a workspace with documents shows no empty text', async () => {
  await renderApp()

  expect(screen.queryByText(pt.sidebar.projectsEmpty)).not.toBeInTheDocument()
  expect(screen.queryByText(pt.sidebar.recentsEmpty)).not.toBeInTheDocument()
})
