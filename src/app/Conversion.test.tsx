import { emit } from '@tauri-apps/api/event'
import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import type { ConversionEvent } from '@/lib/bindings/ConversionEvent'
import type { Document } from '@/lib/bindings/Document'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import { App } from './App'
import { workspaceCommands } from './fixtures'

// Converting the open document from the Converter menu (specs/08-conversao.md,
// "Fluxo" and "Execução"). The fixtures offer PDF for every document.

const DERIVATIVE: Document = { id: 'd-readme-pdf', path: [2], name: 'Leia-me.pdf', format: 'pdf', valid: true, protected: false, size: 1024, modifiedAt: 0, createdAt: null }

async function convertReadme(overrides: Record<string, (args: unknown) => unknown> = {}) {
  const calls = mockCommands(workspaceCommands({ convert_document: () => 1, ...overrides }))
  const user = userEvent.setup()
  render(<App preferences={DEFAULT_PREFERENCES} />)
  const sidebar = within(await screen.findByRole('complementary', { name: pt.shell.sidebar }))
  await user.click(await sidebar.findByRole('button', { name: 'Leia-me.txt' }))
  await screen.findByText('Conteúdo do trabalho')
  await user.click(screen.getByRole('button', { name: pt.document.convert }))
  await user.click(screen.getByRole('menuitem', { name: /PDF/ }))
  return { calls, user }
}

const send = (event: ConversionEvent) => act(() => emit('conversion', event))
const toast = () => screen.getByRole('status')

test('asks the core to convert the open document', async () => {
  const { calls } = await convertReadme()

  expect(calls).toContainEqual({ cmd: 'convert_document', args: { id: 'd-readme', target: 'pdf', password: null } })
})

test('shows the running conversion with its progress and a cancel button', async () => {
  const { calls, user } = await convertReadme()
  await send({ kind: 'queued', job: 1, document: 'd-readme', target: 'pdf' })
  await send({ kind: 'started', job: 1 })
  await send({ kind: 'progress', job: 1, progress: { stage: 'reading', fraction: 0.4 } })

  expect(toast()).toHaveTextContent('Convertendo Leia-me.txt para PDF… 40%')
  await user.click(within(toast()).getByRole('button', { name: pt.conversion.cancel }))
  expect(calls).toContainEqual({ cmd: 'cancel_conversion', args: { job: 1 } })
})

test('a finished conversion opens the derivative and confirms with a toast', async () => {
  const derivativeView = { document: DERIVATIVE, folders: [], derived: true, conversionTargets: [], position: null }
  const fallback = workspaceCommands().open_document
  const openDocument = (args: unknown) => ((args as { id: string }).id === DERIVATIVE.id ? derivativeView : fallback(args))
  await convertReadme({ open_document: openDocument })
  await send({ kind: 'queued', job: 1, document: 'd-readme', target: 'pdf' })
  await send({ kind: 'started', job: 1 })
  await send({ kind: 'finished', job: 1, document: DERIVATIVE })

  expect(toast()).toHaveTextContent('Leia-me.pdf criado · Leia-me.txt preservado')
  expect(await screen.findByRole('heading', { name: 'Leia-me.pdf' })).toBeInTheDocument()
})

test('a cancelled conversion says so', async () => {
  await convertReadme()
  await send({ kind: 'queued', job: 1, document: 'd-readme', target: 'pdf' })
  await send({ kind: 'cancelled', job: 1 })

  expect(toast()).toHaveTextContent(pt.errors.ConversionCancelled)
})

test('a failed conversion shows the error', async () => {
  await convertReadme()
  await send({ kind: 'queued', job: 1, document: 'd-readme', target: 'pdf' })
  await send({ kind: 'failed', job: 1, error: { kind: 'WriteFailed', id: 'd-readme' } })

  expect(toast()).toHaveTextContent(pt.errors.WriteFailed)
})

test('a protected document warns and asks for the password before converting', async () => {
  const readme = workspaceCommands().open_document({ id: 'd-readme' }) as { document: Document }
  const protectedView = { ...readme, document: { ...readme.document, protected: true } }
  const { calls, user } = await convertReadme({ open_document: () => protectedView })
  const dialog = within(screen.getByRole('dialog', { name: pt.conversion.protected.title }))

  expect(dialog.getByText(pt.conversion.protected.warning)).toBeInTheDocument()
  expect(calls.some(({ cmd }) => cmd === 'convert_document')).toBe(false)
  await user.type(dialog.getByLabelText(pt.document.password.label), 'segredo')
  await user.click(dialog.getByRole('button', { name: pt.conversion.protected.convert }))
  expect(calls).toContainEqual({ cmd: 'convert_document', args: { id: 'd-readme', target: 'pdf', password: 'segredo' } })
})
