import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import type { Pages } from '../paged/types'
import type { DocumentActions, DocumentFormat, Reading, StageContent } from '../types'
import type { ReaderState } from '../useReaderState'
import { StageBody } from './StageBody'
import { StageToolbar } from './StageToolbar'

type StageProps = {
  content: StageContent
  format: DocumentFormat
  reading: Reading
  reader: ReaderState
  actions: DocumentActions
}

const MOTION = 'transition-[padding,border-radius] duration-200 ease-in-out'

// Stage (Design/specs/05-document-view.md, 5.4). Fullscreen only drops the
// padding, border and radius; sidebar and header stay.
export function Stage({ content, format, reading, reader, actions }: StageProps) {
  const { t } = useTranslation()
  const [pages, setPages] = useState<Pages | null>(null)
  const onPages = (value: Pages) => {
    setPages(value)
    reading.onPosition({ kind: 'Page', page: value.current })
  }
  const truncated = (content.status === 'text' || content.status === 'html') && content.truncated
  return (
    <div className={cn('flex min-w-0 flex-1 flex-col overflow-hidden', MOTION, reader.fullscreen ? 'p-0' : 'p-5')}>
      <section
        aria-label={t('document.stage.label')}
        className={cn(
          'flex flex-1 flex-col overflow-hidden bg-surf-1',
          MOTION,
          reader.fullscreen ? 'rounded-none border-none' : 'rounded-16 border border-line-4',
        )}
      >
        <StageToolbar reader={reader} pages={pages} />
        {truncated && (
          <p role="note" className="m-0 border-b border-line-3 px-3 py-1.5 text-[12px] text-tx-8">
            {t('document.stage.truncated')}
          </p>
        )}
        <div className={cn('flex min-h-0 flex-1 items-center justify-center overflow-hidden', MOTION, reader.fullscreen ? 'p-1' : 'p-2')}>
          <StageBody
            content={content}
            format={format}
            zoom={reader.zoom}
            reading={reading}
            onPages={onPages}
            actions={actions}
          />
        </div>
      </section>
    </div>
  )
}
