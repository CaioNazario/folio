import { expect, test } from 'vitest'
import { dropEmptyPages } from './docxPages'

function page(article: string): string {
  return `<section class="docx"><header><p>Cabeçalho</p></header><article>${article}</article></section>`
}

test('drops pages with no text and no image', () => {
  const host = document.createElement('div')
  host.innerHTML = page('<p>Introdução</p>') + page('<p> </p>') + page('<p><img alt="Figura 1"></p>')

  const pages = dropEmptyPages(host)

  expect(pages.map((kept) => kept.querySelector('article')?.innerHTML)).toEqual(['<p>Introdução</p>', '<p><img alt="Figura 1"></p>'])
  expect(host.querySelectorAll('section.docx')).toHaveLength(2)
})
