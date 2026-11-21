import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import type { Pages } from '../paged/types'
import { usePagedView } from '../paged/usePagedView'
import { openPdf } from '../pdf/pdfEngine'
import type { DocumentActions, Reading } from '../types'
import { useDelayedFlag } from '../useDelayedFlag'
import { PasswordPrompt } from './PasswordPrompt'
import { StageFailed } from './StageFailed'
import { StageMessage } from './StageMessage'

const LOADING_DELAY_MS = 300

type PdfDisplayProps = {
  bytes: ArrayBuffer
  zoom: number
  reading: Reading
  onPages: (pages: Pages) => void
  actions: Pick<DocumentActions, 'onRetry' | 'onLink'>
  showInFolder: ReactNode
}

// pdf.js styles its pages with these variables; the page follows the paper
// tokens (Design/specs/05-document-view.md, 5.5).
const PAGES = '[--page-border:1px_solid_var(--paper-edge)] [--page-margin:12px_auto] [&_.page]:shadow-page'

// PDF on the stage (specs/07-reader.md, "PDF"): pdf.js pages in continuous
// scroll, with the password, loading and failure states over them.
export function PdfDisplay({ bytes, zoom, reading, onPages, actions, showInFolder }: PdfDisplayProps) {
  const { t } = useTranslation()
  const initialPage = reading.start?.kind === 'Page' ? reading.start.page : null
  const options = { zoom, initialPage, onPages, onLink: actions.onLink, pagerRef: reading.pagerRef }
  const { container, phase, submitPassword } = usePagedView(openPdf, bytes, options)
  const showLoading = useDelayedFlag(phase.kind === 'loading', LOADING_DELAY_MS)
  return (
    <div className="relative flex-1 self-stretch">
      <div ref={container} className={cn('absolute inset-0 overflow-auto', PAGES, phase.kind !== 'ready' && 'invisible')}>
        <div className="pdfViewer" />
      </div>
      <div className="absolute inset-0 flex empty:hidden">
        {phase.kind === 'password' && <PasswordPrompt wrong={phase.wrong} onSubmit={submitPassword} />}
        {phase.kind === 'failed' && <StageFailed onRetry={actions.onRetry} showInFolder={showInFolder} />}
        {showLoading && <StageMessage>{t('document.stage.loading')}</StageMessage>}
      </div>
    </div>
  )
}
