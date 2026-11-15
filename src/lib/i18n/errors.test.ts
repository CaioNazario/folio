import { expect, test } from 'vitest'
import { errorMessage, isBridgeError } from './errors'

test('interpolates the error fields', () => {
  expect(errorMessage({ kind: 'AlreadyExists', path: '/w/Trabalho.docx', name: 'Trabalho.docx' })).toBe(
    'Já existe um item com o nome Trabalho.docx',
  )
})

test('picks the invalid name message by reason', () => {
  expect(errorMessage({ kind: 'InvalidName', reason: 'Hidden' })).toBe('O nome não pode começar com ponto')
})

test('generic I/O errors show only the path', () => {
  expect(errorMessage({ kind: 'Io', path: '/w/a.pdf' })).toBe('Não foi possível acessar /w/a.pdf')
})

test.each([[{ kind: 'Io', path: '/x' }, true], [new Error('boom'), false], ['text', false], [null, false]])(
  'recognizes bridge errors: %o',
  (value, expected) => {
    expect(isBridgeError(value)).toBe(expected)
  },
)
