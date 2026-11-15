import type { RefObject } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { DocumentActions, DocumentFormat, StageContent } from '../types'
import { useDelayedFlag } from '../useDelayedFlag'
import { MarkdownDisplay } from './MarkdownDisplay'
import { Page } from './Page'
import { StageMessage } from './StageMessage'
import { TextDisplay } from './TextDisplay'

const LOADING_DELAY_MS = 300

type StageBodyProps = {
  content: StageContent
  format: DocumentFormat
  zoom: number
  initialScroll: number | null
  onScrollFraction: (fraction: number) => void
  pageRef: RefObject<HTMLElement | null>
  actions: Pick<DocumentActions, 'onRetry' | 'onShowInFolder' | 'onLink'>
}

// What the reading area shows for each stage state.
export function StageBody({ content, format, zoom, initialScroll, onScrollFraction, pageRef, actions }: StageBodyProps) {
  const { t } = useTranslation()
  const showLoading = useDelayedFlag(content.status === 'loading', LOADING_DELAY_MS)
  const showInFolder = <Button onClick={actions.onShowInFolder}>{t('document.showInFolder')}</Button>
  switch (content.status) {
    case 'loading':
      return showLoading ? <StageMessage>{t('document.stage.loading')}</StageMessage> : null
    case 'invalid':
      return <StageMessage actions={showInFolder}>{t('document.stage.invalid', { format: format.toUpperCase() })}</StageMessage>
    case 'unsupported':
      return <StageMessage actions={showInFolder}>{t('document.stage.unsupported', { format: format.toUpperCase() })}</StageMessage>
    case 'notFound':
      return <StageMessage>{t('errors.DocumentNotFound')}</StageMessage>
    case 'failed':
      return (
        <StageMessage actions={<><Button onClick={actions.onRetry}>{t('document.stage.retry')}</Button>{showInFolder}</>}>
          {t('document.stage.failed')}
        </StageMessage>
      )
  }
  const body = content.status === 'text' ? content.text : content.html
  if (body.trim() === '') return <StageMessage>{t('document.stage.empty')}</StageMessage>
  return (
    <Page zoom={zoom} initialScroll={initialScroll} onScrollFraction={onScrollFraction} pageRef={pageRef}>
      {content.status === 'text' ? <TextDisplay text={body} /> : <MarkdownDisplay html={body} onLink={actions.onLink} />}
    </Page>
  )
}
