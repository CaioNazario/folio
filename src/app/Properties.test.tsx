import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { DEFAULT_PREFERENCES } from '@/features/preferences/api'
import type { ConversionLink } from '@/lib/bindings/ConversionLink'
import type { Document } from '@/lib/bindings/Document'
import type { DocumentProperties } from '@/lib/bindings/DocumentProperties'
import type { Metadata } from '@/lib/bindings/Metadata'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import { App } from './App'
import { workspaceCommands } from './fixtures'

// The Properties dialog, opened from the More menu of the open document
// (Design/specs/05-document-view.md, 5.6).

const MARCH_18 = new Date(2026, 2, 18, 21, 30).getTime()
const MARCH_18_TEXT = '18 de mar. de 2026, 21:30'
const labels = pt.document.propertiesDialog

const README: Document = { id: 'd-readme', path: [1], name: 'Leia-me.txt', format: 'txt', valid: true, protected: false, size: 42 * 1024, modifiedAt: MARCH_18, createdAt: null }

const EMPTY_METADATA: Metadata = { title: null, author: null, subject: null, keywords: [], createdAt: null, modifiedAt: null, pages: null, extra: {} }

function properties(overrides: Partial<DocumentProperties> = {}): DocumentProperties {
  return { document: README, location: [], metadata: EMPTY_METADATA, convertedFrom: null, derivatives: [], ...overrides }
}

function link(document: Partial<Document>, changes: Partial<ConversionLink> = {}): ConversionLink {
  const linked = { ...README, ...document }
  return { document: linked, sourceFormat: 'txt', targetFormat: 'md', convertedAt: MARCH_18, sourceChanged: false, derivativeChanged: false, ...changes }
}

// Opens Leia-me.txt and its Properties, answered by `answer`.
async function openProperties(answer: () => unknown = () => properties()) {
  const calls = mockCommands(workspaceCommands({ get_properties: answer }))
  const user = userEvent.setup()
  render(<App preferences={DEFAULT_PREFERENCES} />)
  const sidebar = within(await screen.findByRole('complementary', { name: pt.shell.sidebar }))
  await user.click(await sidebar.findByRole('button', { name: 'Leia-me.txt' }))
  await screen.findByText('Conteúdo do trabalho')
  await user.click(screen.getByRole('button', { name: pt.document.more }))
  await user.click(screen.getByRole('menuitem', { name: pt.document.properties }))
  return { calls, user }
}

const dialog = () => within(screen.getByRole('dialog', { name: pt.document.properties }))
const block = (name: string) => within(dialog().getByRole('region', { name }))
const value = (name: string, label: string) => block(name).getByText(label, { selector: 'dt' }).nextElementSibling?.textContent

test('opens properties from the more menu with the file facts', async () => {
  const { calls } = await openProperties()
  await screen.findByRole('dialog', { name: pt.document.properties })

  expect(calls).toContainEqual({ cmd: 'get_properties', args: { id: 'd-readme' } })
  expect(value(labels.file, labels.name)).toBe('Leia-me.txt')
  expect(value(labels.file, labels.location)).toBe(labels.root)
  expect(value(labels.file, labels.format)).toBe(pt.document.formats.txt)
  expect(value(labels.file, labels.size)).toBe('42 KB')
  expect(value(labels.file, labels.modified)).toBe(MARCH_18_TEXT)
  expect(block(labels.file).queryByText(labels.created)).toBeNull()
})

test('shows the document metadata with pages and labelled extra fields', async () => {
  const metadata = { ...EMPTY_METADATA, title: 'Tese', keywords: ['rust', 'leitura'], pages: 36, extra: { creator: 'Microsoft Word', curso: ['SI', 'BCC'] } }
  await openProperties(() => properties({ metadata }))
  await screen.findByRole('dialog', { name: pt.document.properties })

  expect(value(labels.document, labels.title)).toBe('Tese')
  expect(value(labels.document, labels.keywords)).toBe('rust, leitura')
  expect(value(labels.document, labels.pages)).toBe('36')
  expect(value(labels.document, labels.extra.creator)).toBe('Microsoft Word')
  expect(value(labels.document, 'curso')).toBe('SI, BCC')
})

test('says the metadata was not extracted for a protected document', async () => {
  await openProperties(() => properties({ document: { ...README, protected: true }, metadata: null }))

  expect(await screen.findByText(labels.protected)).toBeInTheDocument()
})

test('says the metadata could not be read for a valid document without it', async () => {
  await openProperties(() => properties({ metadata: null }))

  expect(await screen.findByText(labels.unreadable)).toBeInTheDocument()
})

test('lists the source and the derivatives with their stale warnings', async () => {
  const convertedFrom = link({ id: 'd-pdf', name: 'Artigo.pdf' }, { sourceFormat: 'pdf', targetFormat: 'txt', sourceChanged: true })
  const derivatives = [link({ id: 'd-ref', name: 'Referências.md' }, { derivativeChanged: true })]
  await openProperties(() => properties({ convertedFrom, derivatives }))
  await screen.findByRole('dialog', { name: pt.document.properties })

  const source = within(block(labels.conversion).getByRole('list', { name: labels.convertedFrom }))
  expect(source.getByRole('button', { name: 'Artigo.pdf' })).toBeInTheDocument()
  expect(source.getByText(`PDF → TXT · ${MARCH_18_TEXT}`)).toBeInTheDocument()
  expect(source.getByText(labels.sourceChanged)).toBeInTheDocument()
  const derived = within(block(labels.conversion).getByRole('list', { name: labels.derivatives }))
  expect(derived.getByRole('button', { name: 'Referências.md' })).toBeInTheDocument()
  expect(derived.getByText(labels.derivativeChanged)).toBeInTheDocument()
})

test('clicking a derivative closes the dialog and opens it', async () => {
  const derivatives = [link({ id: 'd-ref', name: 'Referências.md', format: 'md' })]
  const { calls, user } = await openProperties(() => properties({ derivatives }))
  await user.click(await screen.findByRole('button', { name: 'Referências.md' }))

  expect(await screen.findByRole('heading', { name: 'Referências.md' })).toBeInTheDocument()
  expect(screen.queryByRole('dialog')).toBeNull()
  expect(calls).toContainEqual({ cmd: 'open_document', args: { id: 'd-ref' } })
})

test('escape closes the dialog', async () => {
  const { user } = await openProperties()
  await screen.findByRole('dialog', { name: pt.document.properties })
  await user.keyboard('{Escape}')

  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
})

test('a failed properties command opens nothing and shows a toast', async () => {
  await openProperties(() => {
    throw { kind: 'DocumentNotFound', id: 'd-readme' }
  })

  expect(await screen.findByText(labels.loadFailed)).toBeInTheDocument()
  expect(screen.queryByRole('dialog')).toBeNull()
})
