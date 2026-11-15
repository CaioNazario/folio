import { act, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { ToastProvider, ToastViewport } from './toast'
import { TOAST_DURATION_MS, useToast } from './toast-context'

function Triggers() {
  const { show } = useToast()
  return ['Primeiro', 'Segundo'].map((text) => (
    <button key={text} onClick={() => show(text)}>
      {text}
    </button>
  ))
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
