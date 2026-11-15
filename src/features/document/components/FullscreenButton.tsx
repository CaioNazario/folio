import { Maximize, Minimize } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

const ICON = 'absolute transition-opacity duration-160 ease-in-out'

// Fullscreen toggle with the maximize/minimize crossfade
// (Design/specs/05-document-view.md, Toolbar).
export function FullscreenButton({ fullscreen, onToggle }: { fullscreen: boolean; onToggle: () => void }) {
  const { t } = useTranslation()
  const label = fullscreen ? t('document.stage.exitFullscreen') : t('document.stage.enterFullscreen')
  return (
    <Button variant="iconSmall" title={label} aria-label={label} onClick={onToggle} className="relative">
      <Maximize className={cn(ICON, fullscreen ? 'opacity-0' : 'opacity-100')} />
      <Minimize className={cn(ICON, fullscreen ? 'opacity-100' : 'opacity-0')} />
    </Button>
  )
}
