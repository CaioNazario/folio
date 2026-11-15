import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import { Button } from './button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from './dropdown-menu'

function Menu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button>Abrir</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem>Renomear</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

// Radix sets pointer-events: none on the body while the menu is open, as a
// real outside click still reaches it.
async function openMenu() {
  const user = userEvent.setup({ pointerEventsCheck: 0 })
  render(<Menu />)
  await user.click(screen.getByRole('button', { name: 'Abrir' }))
  return user
}

test('opens on trigger click', async () => {
  await openMenu()

  expect(screen.getByRole('menuitem', { name: 'Renomear' })).toBeInTheDocument()
})

test('closes on Esc', async () => {
  const user = await openMenu()
  await user.keyboard('{Escape}')

  expect(screen.queryByRole('menu')).not.toBeInTheDocument()
})

test('closes on outside click', async () => {
  const user = await openMenu()
  await user.click(document.body)

  expect(screen.queryByRole('menu')).not.toBeInTheDocument()
})
