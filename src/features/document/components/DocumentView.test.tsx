import { act, fireEvent, render, renderHook, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test, vi } from 'vitest'
import { pt } from '@/lib/i18n/pt'
import type { DocumentActions, OpenDocument, StageContent } from '../types'
import { useReaderState } from '../useReaderState'
import { DocumentView } from './DocumentView'

const DOC: OpenDocument = {
  id: 'd1',
  name: 'Fichamento.md',
  format: 'md',
  folders: ['Faculdade'],
  derived: false,
  conversions: ['pdf', 'txt'],
  valid: true,
  scroll: null,
}

const TEXT: StageContent = { status: 'text', text: 'Primeiro parágrafo', truncated: false }

function actions(): DocumentActions {
  const names = ['onConvert', 'onRename', 'onShowInFolder', 'onCopyFile', 'onProperties', 'onTrash', 'onRetry', 'onLink']
  return Object.fromEntries(names.map((name) => [name, vi.fn()])) as unknown as DocumentActions
}

function View({ document = DOC, content = TEXT, acts }: { document?: OpenDocument; content?: StageContent; acts: DocumentActions }) {
  const reader = useReaderState()
  return <DocumentView document={document} content={content} reader={reader} actions={acts} />
}

function renderView(props: { document?: OpenDocument; content?: StageContent } = {}) {
  const acts = actions()
  const utils = render(<View {...props} acts={acts} />)
  return { ...utils, acts, user: userEvent.setup({ pointerEventsCheck: 0 }) }
}

const statusText = () => screen.getByText(/Preview|Fullscreen/, { selector: 'span' })

test('shows the breadcrumb and the title', () => {
  renderView()

  expect(screen.getByRole('navigation', { name: pt.document.breadcrumb })).toHaveTextContent('Faculdade/Fichamento.md')
  expect(screen.getByRole('heading', { name: 'Fichamento.md' })).toBeInTheDocument()
})

test('marks a derived document with the badge', () => {
  renderView({ document: { ...DOC, derived: true } })

  expect(screen.getByText(pt.document.derived)).toBeInTheDocument()
})

test('the convert menu lists only the available conversions', async () => {
  const { user, acts } = renderView()
  await user.click(screen.getByRole('button', { name: pt.document.convert }))

  expect(screen.getAllByRole('menuitem').map((item) => item.textContent)).toEqual(['pdfPDF', 'txtTexto simples'])
  await user.click(screen.getByRole('menuitem', { name: /Texto simples/ }))
  expect(acts.onConvert).toHaveBeenCalledWith('txt')
})

test('a document without conversions disables the convert button', () => {
  renderView({ document: { ...DOC, conversions: [] } })

  expect(screen.getByRole('button', { name: pt.document.convert })).toBeDisabled()
})

test('opening the more menu closes the convert menu', async () => {
  const { user } = renderView()
  await user.click(screen.getByRole('button', { name: pt.document.convert }))
  await user.click(screen.getByRole('button', { name: pt.document.more }))

  expect(screen.queryByText(pt.document.convertTo)).not.toBeInTheDocument()
  expect(screen.getByRole('menuitem', { name: pt.document.trash })).toBeInTheDocument()
})

test('the fullscreen button toggles the stage mode', async () => {
  const { user } = renderView()
  await user.click(screen.getByRole('button', { name: pt.document.stage.enterFullscreen }))

  expect(statusText()).toHaveTextContent('Fullscreen')
  expect(screen.getByRole('button', { name: pt.document.stage.exitFullscreen })).toBeInTheDocument()
})

test('keyboard shortcuts zoom and toggle fullscreen', async () => {
  const { user } = renderView()
  await user.keyboard('{Control>}={/Control}{Control>}={/Control}')
  expect(statusText()).toHaveTextContent('120%')

  await user.keyboard('{Control>}0{/Control}{F11}')
  expect(statusText()).toHaveTextContent('Fullscreen · 100%')

  await user.keyboard('{Escape}')
  expect(statusText()).toHaveTextContent('Preview')
})

test('shortcuts are ignored inside text fields', async () => {
  const { user } = renderView()
  const input = document.body.appendChild(document.createElement('input'))
  input.focus()
  await user.keyboard('{Control>}={/Control}')

  expect(statusText()).toHaveTextContent('100%')
  input.remove()
})

test('zoom stops at the limits', () => {
  const { result } = renderHook(() => useReaderState())
  for (let i = 0; i < 30; i++) act(() => result.current.zoomIn())

  expect(result.current.zoom).toBe(300)
})

test('loading shows nothing for the first 300 ms', () => {
  vi.useFakeTimers()
  renderView({ content: { status: 'loading' } })
  expect(screen.queryByText(pt.document.stage.loading)).not.toBeInTheDocument()

  act(() => vi.advanceTimersByTime(300))
  expect(screen.getByText(pt.document.stage.loading)).toBeInTheDocument()
  vi.useRealTimers()
})

test.each([
  [{ status: 'invalid' } as const, 'Este arquivo não é um MD válido'],
  [{ status: 'notFound' } as const, pt.errors.DocumentNotFound],
  [{ status: 'failed' } as const, pt.document.stage.failed],
  [{ status: 'text', text: '  \n', truncated: false } as const, pt.document.stage.empty],
])('shows the %o state', (content, message) => {
  renderView({ content })

  expect(screen.getByText(message)).toBeInTheDocument()
})

test('a failed display offers to try again', async () => {
  const { user, acts } = renderView({ content: { status: 'failed' } })
  await user.click(screen.getByRole('button', { name: pt.document.stage.retry }))

  expect(acts.onRetry).toHaveBeenCalled()
})

test('warns when a large file was cut', () => {
  renderView({ content: { ...TEXT, truncated: true } })

  expect(screen.getByRole('note')).toHaveTextContent(pt.document.stage.truncated)
})

test('a markdown link goes to the core instead of navigating', () => {
  const { acts } = renderView({ content: { status: 'html', html: '<p><a href="../Notas.md">ver</a></p>', truncated: false } })
  const link = screen.getByRole('link', { name: 'ver' })
  const notPrevented = fireEvent.click(link)

  expect(notPrevented).toBe(false)
  expect(acts.onLink).toHaveBeenCalledWith('../Notas.md')
})

// jsdom has no layout: give the page a size and a writable scroll.
function sizedPage() {
  const page = screen.getByRole('article')
  Object.defineProperties(page, {
    clientHeight: { value: 300 },
    scrollHeight: { value: 1000 },
    scrollTop: { value: 0, writable: true },
  })
  return page
}

test('page keys scroll the document one screen and to either end', async () => {
  const { user } = renderView()
  const page = sizedPage()

  await user.keyboard('{PageDown}{ArrowRight}')
  expect(page.scrollTop).toBe(600)
  await user.keyboard('{ArrowLeft}')
  expect(page.scrollTop).toBe(300)
  await user.keyboard('{End}')
  expect(page.scrollTop).toBe(700)
  await user.keyboard('{Home}')
  expect(page.scrollTop).toBe(0)
})

test('page keys stay with an open menu', async () => {
  const { user } = renderView()
  const page = sizedPage()
  await user.click(screen.getByRole('button', { name: pt.document.more }))
  await user.keyboard('{PageDown}')

  expect(page.scrollTop).toBe(0)
})
