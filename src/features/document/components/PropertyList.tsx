import type { ReactNode } from 'react'
import { SectionLabel } from '@/components/ui/section-label'
import { cn } from '@/lib/utils'
import type { PropertyRow } from '../properties/propertyRows'

// The pieces of a Properties block (Design/specs/05-document-view.md, 5.6).

export function PropertySection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-2">
      <SectionLabel tone="home">{title}</SectionLabel>
      {children}
    </section>
  )
}

export function PropertyRows({ rows }: { rows: PropertyRow[] }) {
  return (
    <dl className="m-0 grid grid-cols-[112px_1fr] gap-x-[10px] gap-y-1.5">
      {rows.map(({ label, value, mono }) => (
        <div key={label} className="contents">
          <dt className="text-[12px] text-tx-8">{label}</dt>
          <dd className={cn('m-0 break-words text-tx-1', mono ? 'font-mono text-[11px] leading-[19px]' : 'text-[13px]')}>{value}</dd>
        </div>
      ))}
    </dl>
  )
}

export function PropertyNote({ children }: { children: ReactNode }) {
  return <p className="m-0 text-[12px] text-tx-8">{children}</p>
}
