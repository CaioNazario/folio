import { cva, type VariantProps } from 'class-variance-authority'
import { Slot } from 'radix-ui'
import type * as React from 'react'
import { cn } from '@/lib/utils'

// Secondary button family (Design/specs/06-components.md, "Botão secundário").
const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center border border-line-6 bg-surf-2 outline-none hover:bg-line-3 focus-visible:border-line-9 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        import:
          'w-full justify-start gap-[9px] rounded-14 px-[10px] py-2 text-[13px] font-medium text-tx-1 hover:border-line-9',
        text: 'gap-[7px] rounded-12 px-3 py-1.5 text-[12.5px] text-tx-2',
        icon: 'h-[29px] w-[30px] justify-center rounded-12 text-tx-3 [&_svg]:size-[15px]',
        iconSmall: 'h-6 w-[26px] justify-center rounded-10 bg-pop text-tx-3 hover:bg-hover-3 [&_svg]:size-[13px]',
      },
    },
    defaultVariants: { variant: 'text' },
  },
)

type ButtonProps = React.ComponentProps<'button'> & VariantProps<typeof buttonVariants> & { asChild?: boolean }

function Button({ className, variant, asChild = false, type = 'button', ...props }: ButtonProps) {
  const Comp = asChild ? Slot.Root : 'button'
  return (
    <Comp
      data-slot="button"
      type={asChild ? undefined : type}
      className={cn(buttonVariants({ variant }), className)}
      {...props}
    />
  )
}

export { Button }
