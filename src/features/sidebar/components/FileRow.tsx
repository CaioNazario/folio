import { cn } from '@/lib/utils'
import type { TreeDocument } from '../types'
import { RowName } from './NavRow'
import { PinButton } from './PinButton'

type FileRowProps = {
  document: TreeDocument
  selected: boolean
  pinned: boolean
  onOpen: () => void
  onTogglePin: () => void
}

// Document inside an open folder (Design/specs/03-sidebar.md). The tree guide
// is on the parent level; hover and selection share the rounded overlay that
// covers it (10-known-issues.md #2).
export function FileRow({ document, selected, pinned, onOpen, onTogglePin }: FileRowProps) {
  return (
    <div className="group relative flex items-center gap-2 py-[5px] pr-2 pl-[25px]">
      <span
        aria-hidden="true"
        className={cn(
          'absolute inset-[0_0_0_-8px] rounded-12',
          selected ? 'border-l-2 border-nav-accent bg-surf-4' : 'group-hover:bg-hover',
        )}
      />
      <button
        type="button"
        aria-label={document.name}
        aria-current={selected ? 'page' : undefined}
        onClick={onOpen}
        className="absolute inset-[0_0_0_-8px] cursor-pointer rounded-12 outline-none focus-visible:bg-hover"
      />
      <span className="pointer-events-none relative flex min-w-0 flex-1">
        <RowName>{document.name}</RowName>
      </span>
      <span className="relative flex">
        <PinButton pinned={pinned} onToggle={onTogglePin} />
      </span>
    </div>
  )
}
