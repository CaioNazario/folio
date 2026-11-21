import { type ReactNode, useLayoutEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { openDocx } from '../docx/docxEngine'
import type { Pages } from '../paged/types'
import { usePagedView } from '../paged/usePagedView'
import type { DocumentActions, Reading } from '../types'
import { useDelayedFlag } from '../useDelayedFlag'
import { IMAGE_PLACEHOLDER, labelImagePlaceholders } from './imagePlaceholders'
import { StageFailed } from './StageFailed'
import { StageMessage } from './StageMessage'

const LOADING_DELAY_MS = 300

type DocxDisplayProps = {
  bytes: ArrayBuffer
  zoom: number
  reading: Reading
  onPages: (pages: Pages) => void
  actions: Pick<DocumentActions, 'onRetry' | 'onLink'>
  showInFolder: ReactNode
}

// The engine renders into a div of its own inside the container: pages in a
// column (gutter DOCX_GUTTER), on the paper tokens. Images keep their size:
// the preflight's max-width would shrink floating ones to nothing (spike S4).
const PAGES =
  '[&>div]:flex [&>div]:flex-col [&>div]:items-center [&>div]:gap-3 [&>div]:p-3 [&_section.docx]:shrink-0 [&_section.docx]:isolate [&_section.docx]:bg-paper [&_section.docx]:shadow-page [&_img]:max-w-none'

// DOCX on the stage (specs/07-reader.md, "DOCX"): docx-preview pages in
// continuous scroll, with the loading and failure states over them.
export function DocxDisplay({ bytes, zoom, reading, onPages, actions, showInFolder }: DocxDisplayProps) {
  const { t, i18n } = useTranslation()
  const initialPage = reading.start?.kind === 'Page' ? reading.start.page : null
  const options = { zoom, initialPage, onPages, onLink: actions.onLink, pagerRef: reading.pagerRef }
  const { container, phase } = usePagedView(openDocx, bytes, options)
  const showLoading = useDelayedFlag(phase.kind === 'loading', LOADING_DELAY_MS)
  useLayoutEffect(() => {
    if (phase.kind === 'ready' && container.current) labelImagePlaceholders(container.current, t)
  }, [phase.kind, container, t, i18n.language])
  return (
    <div className="relative flex-1 self-stretch">
      <div ref={container} className={cn('absolute inset-0 overflow-auto', PAGES, IMAGE_PLACEHOLDER, phase.kind !== 'ready' && 'invisible')} />
      <div className="absolute inset-0 flex empty:hidden">
        {phase.kind === 'failed' && <StageFailed onRetry={actions.onRetry} showInFolder={showInFolder} />}
        {showLoading && <StageMessage>{t('document.stage.loading')}</StageMessage>}
      </div>
    </div>
  )
}
