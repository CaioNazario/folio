import { type MouseEvent, useLayoutEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { PAGE_TYPOGRAPHY } from './pageTypography'

type MarkdownDisplayProps = { html: string; onLink: (href: string) => void }

// Images the core could not show arrive as <span data-folio-image="reason">
// with the alt text; the label is the front's, in the active language.
const IMAGE_PLACEHOLDER =
  '[&_[data-folio-image]]:my-[0.4em] [&_[data-folio-image]]:block [&_[data-folio-image]]:rounded-7 [&_[data-folio-image]]:border [&_[data-folio-image]]:border-dashed [&_[data-folio-image]]:border-paper-rule [&_[data-folio-image]]:px-[0.8em] [&_[data-folio-image]]:py-[0.5em] [&_[data-folio-image]]:font-sans [&_[data-folio-image]]:text-[0.8em] [&_[data-folio-image]]:text-paper-mut2 [&_[data-folio-image]]:before:mr-[0.5em] [&_[data-folio-image]]:before:font-semibold [&_[data-folio-image]]:before:content-[attr(data-label)]'

const IMAGE_LABELS = {
  remote: 'document.image.remote',
  missing: 'document.image.missing',
  outside: 'document.image.outside',
  unsupported: 'document.image.unsupported',
  'too-large': 'document.image.tooLarge',
} as const

function imageLabel(reason: string | undefined) {
  return IMAGE_LABELS[(reason ?? 'missing') as keyof typeof IMAGE_LABELS] ?? IMAGE_LABELS.missing
}

function useImageLabels(html: string) {
  const { t, i18n } = useTranslation()
  const container = useRef<HTMLDivElement>(null)
  useLayoutEffect(() => {
    container.current?.querySelectorAll<HTMLElement>('[data-folio-image]').forEach((span) => {
      span.dataset.label = t(imageLabel(span.dataset.folioImage))
    })
  }, [html, t, i18n.language])
  return container
}

// MD view. The HTML comes from comrak and is sanitized by ammonia in the core
// (specs/07-reader.md, "Markdown seguro"). A link never navigates the
// webview: the core decides what it is.
export function MarkdownDisplay({ html, onLink }: MarkdownDisplayProps) {
  const container = useImageLabels(html)
  const onClick = (event: MouseEvent<HTMLDivElement>) => {
    const link = (event.target as HTMLElement).closest('a')
    if (!link) return
    event.preventDefault()
    onLink(link.getAttribute('href') ?? '')
  }
  return (
    <div
      ref={container}
      className={`${PAGE_TYPOGRAPHY} ${IMAGE_PLACEHOLDER}`}
      onClick={onClick}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
