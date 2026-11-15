import { Folder, FolderOpen } from 'lucide-react'
import { cn } from '@/lib/utils'

const LAYER = 'absolute inset-0'
// Opacity 200ms ease, scale 260ms with the Folio curve (01-tokens.md, Motion).
const LAYER_MOTION = { transition: 'opacity 200ms ease, scale 260ms var(--ease-folio)' }

// Folder icon with the closed/open crossfade (Design/specs/03-sidebar.md).
export function FolderIcon({ open }: { open: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        'relative size-[14px] shrink-0 transition-colors duration-180 ease-in-out',
        open ? 'text-nav-accent' : 'text-icon',
      )}
    >
      <Folder size={14} style={LAYER_MOTION} className={cn(LAYER, open ? 'scale-90 opacity-0' : 'scale-100 opacity-100')} />
      <FolderOpen size={14} style={LAYER_MOTION} className={cn(LAYER, open ? 'scale-100 opacity-100' : 'scale-90 opacity-0')} />
    </span>
  )
}
