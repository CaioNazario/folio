import { X } from 'lucide-react'
import { Dialog as DialogPrimitive } from 'radix-ui'
import type { ReactNode } from 'react'
import { Button } from './button'

// Dialog (Design/specs/06-components.md): Radix closes it on Esc and outside
// click and returns focus to the opener. No enter or exit animation.

type DialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  closeLabel: string
  children: ReactNode
}

export function Dialog({ open, onOpenChange, title, closeLabel, children }: DialogProps) {
  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-bg/60" />
        <DialogPrimitive.Content
          aria-describedby={undefined}
          className="fixed top-1/2 left-1/2 z-50 flex max-h-[80vh] w-[min(440px,calc(100vw-32px))] -translate-1/2 flex-col gap-[14px] rounded-18 border border-bar bg-pop p-[18px] text-tx-2 shadow-pop outline-none"
        >
          <div className="flex items-center justify-between gap-[10px]">
            <DialogPrimitive.Title className="m-0 text-[15px] font-semibold tracking-[-0.005em] text-tx-1">{title}</DialogPrimitive.Title>
            <DialogPrimitive.Close asChild>
              <Button variant="iconSmall" title={closeLabel} aria-label={closeLabel}>
                <X />
              </Button>
            </DialogPrimitive.Close>
          </div>
          <div className="min-h-0 overflow-y-auto">{children}</div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  )
}
