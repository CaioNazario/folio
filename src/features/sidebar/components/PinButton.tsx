import { Pin } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

type PinButtonProps = { pinned: boolean; onToggle: () => void }

// Pin button (Design/specs/06-components.md). Hidden until its row (a `group`)
// is hovered, unless pinned. The click never reaches the row.
export function PinButton({ pinned, onToggle }: PinButtonProps) {
  const { t } = useTranslation()
  const label = pinned ? t('sidebar.pin.remove') : t('sidebar.pin.add')
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={pinned}
      onClick={(event) => {
        event.stopPropagation()
        onToggle()
      }}
      className={cn(
        'ml-auto flex shrink-0 cursor-pointer rounded-8 p-0.5 outline-none transition-[opacity,color] duration-150 ease-in-out hover:text-link hover:opacity-100 focus-visible:opacity-100',
        pinned ? 'text-nav-accent opacity-100' : 'text-tx-8 opacity-0 group-hover:opacity-85',
      )}
    >
      <Pin size={12} />
    </button>
  )
}
