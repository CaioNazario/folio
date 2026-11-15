import type { MouseEvent } from 'react'
import { PAGE_TYPOGRAPHY } from './pageTypography'

type MarkdownDisplayProps = { html: string; onLink: (target: string) => void }

// MD view. The HTML comes from comrak and is sanitized by ammonia in the core
// (specs/07-reader.md, "Markdown seguro"). A link never navigates the
// webview: the core decides what it is.
export function MarkdownDisplay({ html, onLink }: MarkdownDisplayProps) {
  const onClick = (event: MouseEvent<HTMLDivElement>) => {
    const link = (event.target as HTMLElement).closest('a')
    if (!link) return
    event.preventDefault()
    onLink(link.getAttribute('href') ?? '')
  }
  return (
    <div className={PAGE_TYPOGRAPHY} onClick={onClick} dangerouslySetInnerHTML={{ __html: html }} />
  )
}
