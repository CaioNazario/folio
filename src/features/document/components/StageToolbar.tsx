import { Minus, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { useFormat } from '@/lib/i18n/useFormat'
import type { Pages } from '../pdf/pdfEngine'
import type { ReaderState } from '../useReaderState'
import { MAX_ZOOM, MIN_ZOOM } from '../zoom'
import { FullscreenButton } from './FullscreenButton'

// Stage toolbar (Design/specs/05-document-view.md). MD and TXT scroll
// continuously, so only the zoom shows; a PDF also shows its current page.
export function StageToolbar({ reader, pages }: { reader: ReaderState; pages: Pages | null }) {
  const { t } = useTranslation()
  const format = useFormat()
  const mode = reader.fullscreen ? t('document.stage.fullscreen') : t('document.stage.preview')
  const zoom = format.percent(reader.zoom / 100)
  return (
    <div className="flex items-center gap-[10px] border-b border-line-3 bg-surf-2 px-3 py-2">
      <span className="font-mono text-[10px] text-tx-5">
        {pages
          ? t('document.stage.statusPaged', { mode, page: format.number(pages.current), total: format.number(pages.total), zoom })
          : t('document.stage.status', { mode, zoom })}
      </span>
      <div className="ml-auto flex gap-1.5">
        <Button
          variant="iconSmall"
          title={t('document.stage.zoomOut')}
          aria-label={t('document.stage.zoomOut')}
          disabled={reader.zoom <= MIN_ZOOM}
          onClick={reader.zoomOut}
        >
          <Minus />
        </Button>
        <Button
          variant="iconSmall"
          title={t('document.stage.zoomIn')}
          aria-label={t('document.stage.zoomIn')}
          disabled={reader.zoom >= MAX_ZOOM}
          onClick={reader.zoomIn}
        >
          <Plus />
        </Button>
        <FullscreenButton fullscreen={reader.fullscreen} onToggle={reader.toggleFullscreen} />
      </div>
    </div>
  )
}
