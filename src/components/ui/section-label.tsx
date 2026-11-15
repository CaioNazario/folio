import type * as React from 'react'
import { cn } from '@/lib/utils'

// Section label (Design/specs/06-components.md): 11/600 uppercase, tone per area.
const TONES = { sidebar: 'text-tx-5', home: 'text-tx-9' }

type SectionLabelProps = React.ComponentProps<'h2'> & { tone: keyof typeof TONES }

export function SectionLabel({ tone, className, ...props }: SectionLabelProps) {
  return (
    <h2 className={cn('m-0 text-[11px] font-semibold tracking-[0.08em] uppercase', TONES[tone], className)} {...props} />
  )
}
