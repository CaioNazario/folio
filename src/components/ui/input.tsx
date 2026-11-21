import type * as React from 'react'
import { cn } from '@/lib/utils'

// Text field (Design/specs/06-components.md, "Campo de texto"). aria-invalid
// turns the border to --danger.
function Input({ className, type = 'text', ...props }: React.ComponentProps<'input'>) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        'rounded-12 border border-line-6 bg-surf-2 px-[10px] py-1.5 text-[13px] text-tx-1 outline-none placeholder:text-tx-8 focus-visible:border-line-9 aria-invalid:border-danger',
        className,
      )}
      {...props}
    />
  )
}

export { Input }
