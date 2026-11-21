// The pages docx-preview renders (<section class="docx">) and where the
// reader is among them (specs/07-reader.md, "DOCX").

export function renderedPages(host: HTMLElement): HTMLElement[] {
  return [...host.querySelectorAll<HTMLElement>(':scope > section.docx')]
}

function isEmpty(page: HTMLElement): boolean {
  const article = page.querySelector('article')
  return !article?.textContent?.trim() && !article?.querySelector('img')
}

// A Word page mark right after another break opens a page with nothing on
// it (spike S4). An empty document keeps its one page.
export function dropEmptyPages(host: HTMLElement): HTMLElement[] {
  const pages = renderedPages(host)
  const empty = pages.filter(isEmpty)
  const dropped = empty.length === pages.length ? empty.slice(1) : empty
  dropped.forEach((page) => page.remove())
  return renderedPages(host)
}

// The page holding the line a third down the stage, from 0.
export function pageAt(container: HTMLElement, pages: HTMLElement[]): number {
  const line = container.getBoundingClientRect().top + container.clientHeight / 3
  const index = pages.findIndex((page) => page.getBoundingClientRect().bottom > line)
  return index === -1 ? pages.length - 1 : index
}

export function scrollToPage(container: HTMLElement, page: HTMLElement, gutter: number) {
  container.scrollTop += page.getBoundingClientRect().top - container.getBoundingClientRect().top - gutter
}

const PX_PER_PT = 4 / 3

// The page width the DOCX declares (docx-preview writes it in pt), in CSS px.
export function declaredWidth(page: HTMLElement): number {
  const width = page.style.width
  return width.endsWith('pt') ? Number.parseFloat(width) * PX_PER_PT : page.offsetWidth
}
