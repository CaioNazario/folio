import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, test, vi } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import { mockCommands } from '@/test-utils'
import type { PagedEvents, PagedStart } from '../paged/types'
import type { DocumentActions, OpenDocument, StageContent } from '../types'
import { useReaderState } from '../useReaderState'
import { DocumentView } from './DocumentView'

// docx-preview does not run in jsdom: the engine is replaced, and each test
// plays what it would report.
const engine = vi.hoisted(() => ({
  events: null as PagedEvents | null,
  start: null as PagedStart | null,
  session: { turn: vi.fn(), setZoom: vi.fn(), close: vi.fn() },
}))

vi.mock('../docx/docxEngine', () => ({
  openDocx: (_container: HTMLElement, _bytes: ArrayBuffer, start: PagedStart, events: PagedEvents) => {
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
  id: 'd-docx',
  name: 'Trabalho.docx',
  format: 'docx',
  folders: [],
  derived: false,
  conversions: [],
  valid: true,
  position: null,
}

const DOCX: StageContent = { status: 'docx', bytes: new ArrayBuffer(8) }

function actions(): DocumentActions {
  const names = ['onConvert', 'onRename', 'onShowInFolder', 'onCopyFile', 'onProperties', 'onTrash', 'onRetry', 'onLink', 'onPassword']
  return Object.fromEntries(names.map((name) => [name, vi.fn()])) as unknown as DocumentActions
}

function View({ document, acts }: { document: OpenDocument; acts: DocumentActions }) {
  const reader = useReaderState()
  return <DocumentView document={document} content={DOCX} reader={reader} actions={acts} />
}

// Renders the DOCX view and waits for the engine (the lazy display) to open it.
async function renderDocx(document: OpenDocument = DOC) {
  const acts = actions()
  const utils = render(<View document={document} acts={acts} />)
  await waitFor(() => expect(engine.events).not.toBeNull())
  const events = engine.events as PagedEvents
  return { ...utils, acts, events, user: userEvent.setup({ pointerEventsCheck: 0 }) }
}

const statusText = () => screen.getByText(/Preview|Fullscreen/, { selector: 'span' })

test('shows the failed state when docx-preview cannot render', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {})
  const { events, user, acts } = await renderDocx()
  act(() => events.onFailed(new Error('end of central directory not found')))
  await user.click(screen.getByRole('button', { name: pt.document.stage.retry }))

  expect(screen.getByText(pt.document.stage.failed)).toBeInTheDocument()
  expect(acts.onRetry).toHaveBeenCalled()
})

test('shows the current page and the total in the toolbar', async () => {
  const { events } = await renderDocx()
  act(() => events.onPages({ current: 2, total: 35 }))

  expect(statusText()).toHaveTextContent('Preview · página 2 de 35 · 100%')
})

test('opens on the saved page', async () => {
  await renderDocx({ ...DOC, position: { kind: 'Page', page: 12 } })

  expect(engine.start).toEqual({ page: 12, zoom: 100 })
})

test('saves the current page as the reading position', async () => {
  const calls = mockCommands()
  const { events, unmount } = await renderDocx()
  act(() => events.onPages({ current: 4, total: 9 }))
  unmount()

  expect(calls).toContainEqual({ cmd: 'save_position', args: { id: 'd-docx', position: { kind: 'Page', page: 4 } } })
})

test('page keys turn the docx pages', async () => {
  const { user } = await renderDocx()
  await user.keyboard('{ArrowRight}{PageUp}{End}{Home}')

  expect(engine.session.turn.mock.calls).toEqual([['next'], ['previous'], ['last'], ['first']])
})

test('sends links in the docx to the core', async () => {
  const { events, acts } = await renderDocx()
  act(() => events.onLink('https://www.gov.br/dnit'))

  expect(acts.onLink).toHaveBeenCalledWith('https://www.gov.br/dnit')
})
