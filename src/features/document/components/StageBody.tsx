import { lazy, Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { Pages } from '../paged/types'
import type { DocumentActions, DocumentFormat, Reading, StageContent } from '../types'
import { useDelayedFlag } from '../useDelayedFlag'
import { MarkdownDisplay } from './MarkdownDisplay'
import { Page } from './Page'
import { PasswordPrompt } from './PasswordPrompt'
import { StageFailed } from './StageFailed'
import { StageMessage } from './StageMessage'
import { TextDisplay } from './TextDisplay'

// pdf.js and docx-preview only load when a document of theirs opens.
const PdfDisplay = lazy(() => import('./PdfDisplay').then((module) => ({ default: module.PdfDisplay })))
const DocxDisplay = lazy(() => import('./DocxDisplay').then((module) => ({ default: module.DocxDisplay })))

const LOADING_DELAY_MS = 300

type StageBodyProps = {
  content: StageContent
  format: DocumentFormat
  zoom: number
  reading: Reading
  onPages: (pages: Pages) => void
  actions: Pick<DocumentActions, 'onRetry' | 'onShowInFolder' | 'onLink' | 'onPassword'>
}

// What the reading area shows for each stage state.
export function StageBody({ content, format, zoom, reading, onPages, actions }: StageBodyProps) {
  const { t } = useTranslation()
  const showLoading = useDelayedFlag(content.status === 'loading', LOADING_DELAY_MS)
  const showInFolder = <Button onClick={actions.onShowInFolder}>{t('document.showInFolder')}</Button>
  switch (content.status) {
    case 'loading':
      return showLoading ? <StageMessage>{t('document.stage.loading')}</StageMessage> : null
    case 'invalid':
      return <StageMessage actions={showInFolder}>{t('document.stage.invalid', { format: format.toUpperCase() })}</StageMessage>
    case 'password':
      return <PasswordPrompt wrong={content.wrong} onSubmit={actions.onPassword} />
    case 'notFound':
      return <StageMessage>{t('errors.DocumentNotFound')}</StageMessage>
    case 'failed':
      return <StageFailed onRetry={actions.onRetry} showInFolder={showInFolder} />
    case 'pdf':
    case 'docx': {
      const Display = content.status === 'pdf' ? PdfDisplay : DocxDisplay
      return (
        <Suspense>
          <Display bytes={content.bytes} zoom={zoom} reading={reading} onPages={onPages} actions={actions} showInFolder={showInFolder} />
        </Suspense>
      )
    }
  }
  const body = content.status === 'text' ? content.text : content.html
  if (body.trim() === '') return <StageMessage>{t('document.stage.empty')}</StageMessage>
  const initialScroll = reading.start?.kind === 'Scroll' ? reading.start.fraction : null
  return (
    <Page zoom={zoom} initialScroll={initialScroll} onScrollFraction={(fraction) => reading.onPosition({ kind: 'Scroll', fraction })} pageRef={reading.pageRef}>
      {content.status === 'text' ? <TextDisplay text={body} /> : <MarkdownDisplay html={body} onLink={actions.onLink} />}
    </Page>
  )
}
