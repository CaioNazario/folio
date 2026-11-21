import { type MouseEvent, useLayoutEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { IMAGE_PLACEHOLDER, labelImagePlaceholders } from './imagePlaceholders'
import { PAGE_TYPOGRAPHY } from './pageTypography'

type MarkdownDisplayProps = { html: string; onLink: (href: string) => void }

function useImageLabels(html: string) {
  const { t, i18n } = useTranslation()
  const container = useRef<HTMLDivElement>(null)
  useLayoutEffect(() => {
    if (container.current) labelImagePlaceholders(container.current, t)
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
