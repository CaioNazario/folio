import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type NavRowProps = {
  label: string
  onClick: () => void
  children: ReactNode
  trailing?: ReactNode
  className?: string
  expanded?: boolean
}

// Sidebar navigation row (Design/specs/06-components.md). The main action is a
// button covering the row; trailing controls (pin, counts) sit beside it.
export function NavRow({ label, onClick, children, trailing, className, expanded }: NavRowProps) {
  return (
    <div className={cn('group relative flex items-center gap-2 rounded-12 px-2 py-[5px] hover:bg-hover', className)}>
      <button
        type="button"
        aria-label={label}
        aria-expanded={expanded}
        onClick={onClick}
        className="absolute inset-0 cursor-pointer rounded-12 outline-none focus-visible:bg-hover"
      />
      <span className="pointer-events-none relative flex min-w-0 flex-1 items-center gap-2">{children}</span>
      <span className="relative flex shrink-0 items-center gap-2">{trailing}</span>
    </div>
  )
}

export function RowName({ children }: { children: ReactNode }) {
  return <span className="min-w-0 flex-1 truncate text-[13px] text-tx-1">{children}</span>
}
