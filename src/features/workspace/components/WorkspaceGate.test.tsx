import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { mockCommands, renderWithProviders } from '@/test-utils'
import { WorkspaceGate } from './WorkspaceGate'

const DEFAULT_DIR = '/home/caio/Documentos/Folio'

function renderGate(commands: Record<string, (args: unknown) => unknown>) {
  const calls = mockCommands({ get_default_workspace_dir: () => DEFAULT_DIR, ...commands })
  renderWithProviders(
    <WorkspaceGate>
      <p>workspace aberto</p>
    </WorkspaceGate>,
  )
  return { calls, user: userEvent.setup() }
}

const throws = (error: object) => () => {
  throw error
}

test('shows the app when the last workspace opens', async () => {
  renderGate({ open_last_workspace: () => true })

  expect(await screen.findByText('workspace aberto')).toBeInTheDocument()
})

test('first run offers the default directory', async () => {
  renderGate({ open_last_workspace: () => false })

  expect(await screen.findByText(DEFAULT_DIR)).toBeInTheDocument()
})

test('creating the workspace opens the app', async () => {
  const { calls, user } = renderGate({ open_last_workspace: () => false })
  await screen.findByText(DEFAULT_DIR)
  await user.click(screen.getByRole('button', { name: pt.workspace.setup.create }))

  expect(await screen.findByText('workspace aberto')).toBeInTheDocument()
  expect(calls).toContainEqual({ cmd: 'create_workspace', args: { path: DEFAULT_DIR } })
})

test('a refused directory shows the reason', async () => {
  const { user } = renderGate({ open_last_workspace: () => false, create_workspace: throws({ kind: 'ForbiddenRoot', path: '/' }) })
  await screen.findByText(DEFAULT_DIR)
  await user.click(screen.getByRole('button', { name: pt.workspace.setup.create }))

  expect(await screen.findByRole('alert')).toHaveTextContent('/ não pode ser um workspace')
})

test('a directory inside another workspace offers to open that one', async () => {
  const { calls, user } = renderGate({
    open_last_workspace: () => false,
    create_workspace: throws({ kind: 'NestedWorkspace', path: '/home/caio/Docs' }),
  })
  await screen.findByText(DEFAULT_DIR)
  await user.click(screen.getByRole('button', { name: pt.workspace.setup.create }))
  await user.click(await screen.findByRole('button', { name: pt.workspace.setup.openAncestor }))

  expect(calls).toContainEqual({ cmd: 'open_workspace', args: { path: '/home/caio/Docs' } })
  expect(await screen.findByText('workspace aberto')).toBeInTheDocument()
})

test('a workspace that is gone shows the error with retry', async () => {
  renderGate({ open_last_workspace: throws({ kind: 'WorkspaceNotFound', path: '/mnt/hd/Folio' }) })

  expect(await screen.findByText('Workspace não encontrado: /mnt/hd/Folio')).toBeInTheDocument()
  expect(screen.getByRole('button', { name: pt.workspace.retry })).toBeInTheDocument()
})
