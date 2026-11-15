import type { ProjectFile } from '../types'
import { RowName } from './NavRow'
import { PinButton } from './PinButton'
import { cn } from '@/lib/utils'

type FileRowProps = {
  file: ProjectFile
  selected: boolean
  pinned: boolean
  onOpen: () => void
  onTogglePin: () => void
}

// File inside an open project (Design/specs/03-sidebar.md). Hover and
// selection use the same rounded overlay over the tree guide
// (10-known-issues.md #2).
export function FileRow({ file, selected, pinned, onOpen, onTogglePin }: FileRowProps) {
  return (
    <div className="group relative ml-[7px] flex items-center gap-2 border-l border-line-3 py-[5px] pr-2 pl-[25px]">
      <span
        aria-hidden="true"
        className={cn(
          'absolute inset-[0_0_0_-7px] rounded-12',
          selected ? 'border-l-2 border-nav-accent bg-surf-4' : 'group-hover:bg-hover',
        )}
      />
      <button
        type="button"
        aria-label={file.name}
        aria-current={selected ? 'page' : undefined}
        onClick={onOpen}
        className="absolute inset-[0_0_0_-7px] cursor-pointer rounded-12 outline-none focus-visible:bg-hover"
      />
      <span className="pointer-events-none relative flex min-w-0 flex-1">
        <RowName>{file.name}</RowName>
      </span>
      <span className="relative flex">
        <PinButton pinned={pinned} onToggle={onTogglePin} />
      </span>
    </div>
  )
}
