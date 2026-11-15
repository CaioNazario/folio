import { EllipsisVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { DocumentActions } from '../types'

type MoreMenuProps = Pick<DocumentActions, 'onRename' | 'onShowInFolder' | 'onCopyFile' | 'onProperties' | 'onTrash'> & {
  open: boolean
  onOpenChange: (open: boolean) => void
}

// "Mais" menu (Design/specs/05-document-view.md, 5.3).
export function MoreMenu({ open, onOpenChange, ...actions }: MoreMenuProps) {
  const { t } = useTranslation()
  return (
    <DropdownMenu open={open} onOpenChange={onOpenChange} modal={false}>
      <DropdownMenuTrigger asChild>
        <Button variant="icon" title={t('document.more')} aria-label={t('document.more')}>
          <EllipsisVertical />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-[186px]">
        <DropdownMenuItem onSelect={actions.onRename}>{t('document.rename')}</DropdownMenuItem>
        <DropdownMenuItem onSelect={actions.onShowInFolder}>{t('document.showInFolder')}</DropdownMenuItem>
        <DropdownMenuItem onSelect={actions.onCopyFile}>{t('document.copyFile')}</DropdownMenuItem>
        <DropdownMenuItem onSelect={actions.onProperties}>{t('document.properties')}</DropdownMenuItem>
        <DropdownMenuItem variant="danger" onSelect={actions.onTrash}>
          {t('document.trash')}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
