import { expect, test } from 'vitest'
import { MAX_ZOOM, MIN_ZOOM, stepZoom } from './zoom'

test('steps by 10%', () => {
  expect(stepZoom(100, 1)).toBe(110)
  expect(stepZoom(100, -1)).toBe(90)
})

test('stays within the limits', () => {
  expect(stepZoom(MAX_ZOOM, 1)).toBe(MAX_ZOOM)
  expect(stepZoom(MIN_ZOOM, -1)).toBe(MIN_ZOOM)
})
