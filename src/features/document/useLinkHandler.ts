import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useToast } from '@/components/ui/toast-context'
import type { DocumentRef } from '@/features/sidebar/types'
import type { LinkTarget } from '@/lib/bindings/LinkTarget'
import { useReportError } from '@/lib/useReportError'
import { openExternal, resolveLink } from './api'
import { scrollToHeading } from './pageScroll'

// Name to show while a linked document opens: the last part of the href.
function linkedName(href: string): string {
  const last = href.split('#')[0].split('/').pop() ?? ''
  try {
    return decodeURIComponent(last)
  } catch {
    return last
  }
}

// Goes where the core said a link leads; Ignored does nothing.
function useFollowTarget(open: (ref: DocumentRef) => void) {
  const { t } = useTranslation()
  const { show } = useToast()
  return useCallback(
    (href: string) => (target: LinkTarget) => {
      if (target.kind === 'External') return openExternal(target.url)
      if (target.kind === 'Document') return open({ id: target.id, name: linkedName(href) })
      if (target.kind === 'Anchor') return scrollToHeading(target.fragment)
      if (target.kind === 'Unsupported') show(t('document.linkUnsupported'))
    },
    [open, show, t],
  )
}

// Follows a link clicked in the document as the core resolves it
// (specs/07-reader.md, "Links").
export function useLinkHandler(documentId: string | null, open: (ref: DocumentRef) => void) {
  const report = useReportError()
  const follow = useFollowTarget(open)
  return useCallback(
    (href: string) => {
      if (!documentId) return
      resolveLink(documentId, href).then(follow(href)).catch(report)
    },
    [documentId, follow, report],
  )
}
