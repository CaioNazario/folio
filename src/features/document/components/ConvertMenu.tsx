import { ChevronDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { DocumentFormat } from '../types'
import { FormatChip } from './FormatChip'

type ConvertMenuProps = {
  conversions: DocumentFormat[]
  open: boolean
  onOpenChange: (open: boolean) => void
  onConvert: (format: DocumentFormat) => void
}

// "Converter" dropdown (Design/specs/05-document-view.md, 5.2). Lists only the
// conversions available for the open document.
export function ConvertMenu({ conversions, open, onOpenChange, onConvert }: ConvertMenuProps) {
  const { t } = useTranslation()
  return (
    <DropdownMenu open={open} onOpenChange={onOpenChange} modal={false}>
      <DropdownMenuTrigger asChild disabled={conversions.length === 0}>
        <Button variant="text" className="gap-1.5">
          {t('document.convert')}
          <ChevronDown size={13} className="text-tx-7" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-[196px]">
        <DropdownMenuLabel className="pt-[7px] pb-1.5 text-[11px]">{t('document.convertTo')}</DropdownMenuLabel>
        {conversions.map((format) => (
          <DropdownMenuItem key={format} onSelect={() => onConvert(format)}>
            <FormatChip format={format} />
            {t(`document.formats.${format}`)}
          </DropdownMenuItem>
        ))}
        <p className="mx-[9px] mt-[5px] mb-1 border-t border-bar pt-[7px] text-[11px] leading-[1.45] text-tx-9">
          {t('document.convertNote')}
        </p>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
