import { ChevronRight } from 'lucide-react'
import { useFormat } from '@/lib/i18n/useFormat'
import { cn } from '@/lib/utils'
import type { TreeFolder } from '../types'
import { FolderIcon } from './FolderIcon'
import { NavRow, RowName } from './NavRow'
import { PinButton } from './PinButton'

type ProjectRowProps = {
  folder: TreeFolder
  open: boolean
  pinned: boolean
  onToggle: () => void
  onTogglePin: () => void
}

// Project or folder row (Design/specs/03-sidebar.md): chevron without
// transition, crossfading folder icon, pin and document count.
export function ProjectRow({ folder, open, pinned, onToggle, onTogglePin }: ProjectRowProps) {
  const format = useFormat()
  return (
    <NavRow
      label={folder.name}
      expanded={open}
      onClick={onToggle}
      className="mb-px py-1.5"
      trailing={
        <>
          <PinButton pinned={pinned} onToggle={onTogglePin} />
          <span className="font-mono text-[10px] text-tx-1">{format.number(folder.count)}</span>
        </>
      }
    >
      <ChevronRight size={12} aria-hidden="true" className={cn('shrink-0 text-tx-1', open && 'rotate-90')} />
      <FolderIcon open={open} />
      <RowName>{folder.name}</RowName>
    </NavRow>
  )
}
