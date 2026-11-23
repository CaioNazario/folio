import { act, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { ToastProvider, ToastViewport } from './toast'
import { TOAST_DURATION_MS, useToast } from './toast-context'

const onCancel = vi.fn()

function Triggers() {
  const { show } = useToast()
  const sticky = () => show('Convertendo', { sticky: true, action: { label: 'Cancelar', onClick: onCancel } })
  return (
    <>
      {['Primeiro', 'Segundo'].map((text) => (
        <button key={text} onClick={() => show(text)}>
          {text}
        </button>
      ))}
      <button onClick={sticky}>Fixo</button>
    </>
  )
}

const show = (text: string) => fireEvent.click(screen.getByRole('button', { name: text }))
const wait = (ms: number) => act(() => vi.advanceTimersByTime(ms))

beforeEach(() => {
  vi.useFakeTimers()
  render(
    <ToastProvider>
      <Triggers />
      <ToastViewport />
    </ToastProvider>,
  )
})

afterEach(() => vi.useRealTimers())

test('shows the message', () => {
  show('Primeiro')

  expect(screen.getByRole('status')).toHaveTextContent('Primeiro')
})

test('dismisses itself after the duration', () => {
  show('Primeiro')
  wait(TOAST_DURATION_MS)

  expect(screen.getByRole('status')).toBeEmptyDOMElement()
})

test('a new toast replaces the current one and restarts the timer', () => {
  show('Primeiro')
  wait(TOAST_DURATION_MS - 100)
  show('Segundo')
  wait(TOAST_DURATION_MS - 100)

  expect(screen.getByRole('status')).toHaveTextContent('Segundo')
})

test('a sticky toast stays past the duration', () => {
  show('Fixo')
  wait(TOAST_DURATION_MS * 3)

  expect(screen.getByRole('status')).toHaveTextContent('Convertendo')
})

test('the toast action runs when clicked', () => {
  show('Fixo')
  fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }))

  expect(onCancel).toHaveBeenCalledOnce()
})
