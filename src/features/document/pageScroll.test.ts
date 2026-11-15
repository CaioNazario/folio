import { expect, test } from 'vitest'
import { pageMove, scrollTarget } from './pageScroll'

function page(scrollTop: number): HTMLElement {
  return { scrollTop, clientHeight: 300, scrollHeight: 1000 } as HTMLElement
}

test.each([
  ['PageDown', 'next'],
  ['ArrowRight', 'next'],
  ['PageUp', 'previous'],
  ['ArrowLeft', 'previous'],
  ['Home', 'first'],
  ['End', 'last'],
])('%s moves to the %s page', (key, move) => {
  expect(pageMove(key)).toBe(move)
})

test('other keys are not page moves', () => {
  expect(pageMove('ArrowDown')).toBeUndefined()
})

test.each([
  ['next', 400],
  ['previous', -200],
  ['first', 0],
  ['last', 700],
] as const)('%s scrolls a page to %i', (move, top) => {
  expect(scrollTarget(page(100), move)).toBe(top)
})
