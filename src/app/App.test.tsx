import { render, screen } from '@testing-library/react'
import { expect, test } from 'vitest'
import { App } from './App'

test('renderiza a sidebar e a área principal', () => {
  render(<App />)

  expect(screen.getByRole('complementary', { name: 'Sidebar' })).toBeInTheDocument()
  expect(screen.getByRole('main')).toBeInTheDocument()
})
