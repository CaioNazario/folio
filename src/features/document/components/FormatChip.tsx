import type { DocumentFormat } from '../types'

// Format chip (Design/specs/06-components.md): the extension in Mono.
export function FormatChip({ format }: { format: DocumentFormat }) {
  return (
    <span className="w-[30px] shrink-0 rounded-7 bg-surf-4 px-1 py-0.5 text-center font-mono text-[9px] font-medium text-tx-1 uppercase">
      {format}
    </span>
  )
}
