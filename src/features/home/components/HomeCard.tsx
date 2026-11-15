import type * as React from 'react'
import { cn } from '@/lib/utils'

// Card (Design/specs/06-components.md), clickable as a whole.
export function HomeCard({ className, ...props }: React.ComponentProps<'button'>) {
  return (
    <button
      type="button"
      className={cn(
        'cursor-pointer rounded-18 border border-line-4 bg-surf-2 p-[14px] text-left text-tx-2 transition-colors hover:border-line-9',
        className,
      )}
      {...props}
    />
  )
}
