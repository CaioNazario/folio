import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, test, vi } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import type { PagedEvents, PagedStart } from '../paged/types'
import type { DocumentActions, OpenDocument, StageContent } from '../types'
import { useReaderState } from '../useReaderState'
import { DocumentView } from './DocumentView'

// pdf.js does not run in jsdom: the engine is replaced, and each test plays
// what pdf.js would report.
const engine = vi.hoisted(() => ({
  events: null as PagedEvents | null,
  start: null as PagedStart | null,
  session: { turn: vi.fn(), setZoom: vi.fn(), close: vi.fn() },
}))

vi.mock('../pdf/pdfEngine', () => ({
  openPdf: (_container: HTMLElement, _bytes: ArrayBuffer, start: PagedStart, events: PagedEvents) => {
    engine.start = start
    engine.events = events
    return engine.session
  },
}))

beforeEach(() => {
  engine.events = null
  engine.start = null
  vi.clearAllMocks()
})

const DOC: OpenDocument = {
  id: 'd-pdf',
  name: 'Artigo.pdf',
  format: 'pdf',
  folders: [],
  derived: false,
  conversions: [],
  valid: true,
  position: null,
}

const PDF: StageContent = { status: 'pdf', bytes: new ArrayBuffer(8) }

function actions(): DocumentActions {
  const names = ['onConvert', 'onRename', 'onShowInFolder', 'onCopyFile', 'onProperties', 'onTrash', 'onRetry', 'onLink', 'onPassword']
  return Object.fromEntries(names.map((name) => [name, vi.fn()])) as unknown as DocumentActions
}

function View({ document, acts }: { document: OpenDocument; acts: DocumentActions }) {
  const reader = useReaderState()
  return <DocumentView document={document} content={PDF} reader={reader} actions={acts} />
}

// Renders the PDF view and waits for pdf.js (the lazy display) to open it.
async function renderPdf(document: OpenDocument = DOC) {
  const acts = actions()
  const utils = render(<View document={document} acts={acts} />)
  await waitFor(() => expect(engine.events).not.toBeNull())
  const events = engine.events as PagedEvents
  return { ...utils, acts, events, user: userEvent.setup({ pointerEventsCheck: 0 }) }
}

const statusText = () => screen.getByText(/Preview|Fullscreen/, { selector: 'span' })

test('shows the current page and the total in the toolbar', async () => {
  const { events } = await renderPdf()
  act(() => events.onPages({ current: 3, total: 12 }))

  expect(statusText()).toHaveTextContent('Preview · página 3 de 12 · 100%')
})

test('asks for the password on the stage when the pdf is protected', async () => {
  const { events, user } = await renderPdf()
  const submit = vi.fn()
  act(() => events.onPassword(false, submit))
  expect(screen.getByText(pt.errors.PasswordRequired)).toBeInTheDocument()

  await user.type(screen.getByLabelText(pt.document.password.label), 'segredo')
  await user.click(screen.getByRole('button', { name: pt.document.password.open }))
  expect(submit).toHaveBeenCalledWith('segredo')
})

test('shows the wrong password error on the field', async () => {
  const { events } = await renderPdf()
  act(() => events.onPassword(true, vi.fn()))

  const field = screen.getByLabelText(pt.document.password.label)
  expect(field).toHaveAttribute('aria-invalid', 'true')
  expect(field).toHaveAccessibleDescription(pt.errors.WrongPassword)
})

test('shows the failed state when pdf.js cannot open the document', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {})
  const { events, user, acts } = await renderPdf()
  act(() => events.onFailed(new Error('Invalid PDF structure')))
  await user.click(screen.getByRole('button', { name: pt.document.stage.retry }))

  expect(screen.getByText(pt.document.stage.failed)).toBeInTheDocument()
  expect(acts.onRetry).toHaveBeenCalled()
})

test('opens on the saved page', async () => {
  await renderPdf({ ...DOC, position: { kind: 'Page', page: 7 } })

  expect(engine.start).toEqual({ page: 7, zoom: 100 })
})

test('saves the current page as the reading position', async () => {
  const calls = mockCommands()
  const { events, unmount } = await renderPdf()
  act(() => events.onPages({ current: 5, total: 9 }))
  unmount()

  expect(calls).toContainEqual({ cmd: 'save_position', args: { id: 'd-pdf', position: { kind: 'Page', page: 5 } } })
})

test('page keys turn to the next, previous, first and last page', async () => {
  const { user } = await renderPdf()
  await user.keyboard('{PageDown}{ArrowLeft}{Home}{End}')

  expect(engine.session.turn.mock.calls).toEqual([['next'], ['previous'], ['first'], ['last']])
})

test('sends external links to the core', async () => {
  const { events, acts } = await renderPdf()
  act(() => events.onLink('https://example.org'))

  expect(acts.onLink).toHaveBeenCalledWith('https://example.org')
})
