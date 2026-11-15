import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useToast } from '@/components/ui/toast-context'
import type { DocumentRef } from '@/features/sidebar/types'
import type { LinkTarget } from '@/lib/bindings/LinkTarget'
import { useReportError } from '@/lib/useReportError'
import { openExternal, resolveLink } from './api'

// Name to show while a linked document opens: the last part of the href.
function linkedName(href: string): string {
  const last = href.split('#')[0].split('/').pop() ?? ''
  try {
    return decodeURIComponent(last)
  } catch {
    return last
  }
}

// Follows a link clicked in the document as the core resolves it
// (specs/07-reader.md, "Links"). Anchors wait for heading ids from the core.
export function useLinkHandler(documentId: string | null, open: (ref: DocumentRef) => void) {
  const { t } = useTranslation()
  const { show } = useToast()
  const report = useReportError()
  const follow = useCallback(
    (href: string) => (target: LinkTarget) => {
      if (target.kind === 'External') return openExternal(target.url)
      if (target.kind === 'Document') return open({ id: target.id, name: linkedName(href) })
      if (target.kind === 'Unsupported') show(t('document.linkUnsupported'))
    },
    [open, show, t],
  )
  return useCallback(
    (href: string) => {
      if (!documentId) return
      resolveLink(documentId, href).then(follow(href)).catch(report)
    },
    [documentId, follow, report],
  )
}
