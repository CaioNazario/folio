import type { Dispatch, SetStateAction } from 'react'
import { useTranslation } from 'react-i18next'
import type { DocumentActions, OpenDocument } from '../types'
import { ConvertMenu } from './ConvertMenu'
import { MoreMenu } from './MoreMenu'

export type OpenMenu = 'convert' | 'more' | null

function nextMenu(current: OpenMenu, menu: Exclude<OpenMenu, null>, open: boolean): OpenMenu {
  if (open) return menu
  return current === menu ? null : current
}

type DocumentHeaderProps = {
  document: OpenDocument
  actions: DocumentActions
  openMenu: OpenMenu
  onOpenMenu: Dispatch<SetStateAction<OpenMenu>>
}

// Header (Design/specs/05-document-view.md, 5.1). Converter and Mais are
// mutually exclusive: opening one closes the other. Closing a menu only clears
// the state while it is still the open one, since the outside click that
// closes it may land after the other menu opened.
export function DocumentHeader({ document, actions, openMenu, onOpenMenu }: DocumentHeaderProps) {
  const { t } = useTranslation()
  const toggle = (menu: Exclude<OpenMenu, null>) => (open: boolean) =>
    onOpenMenu((current) => nextMenu(current, menu, open))
  return (
    <header className="flex h-14 shrink-0 items-center gap-[14px] border-b border-line-3 px-[18px]">
      <div className="min-w-0">
        <nav aria-label={t('document.breadcrumb')} className="flex gap-[7px] truncate font-mono text-[10.5px] text-tx-6">
          {document.folder && <span>{document.folder}</span>}
          {document.folder && <span aria-hidden="true">/</span>}
          <span className="truncate text-tx-4">{document.name}</span>
        </nav>
        <div className="mt-0.5 flex min-w-0 items-center gap-[9px]">
          <h1 className="m-0 truncate text-[15px] font-semibold tracking-[-0.005em] text-tx-1">{document.name}</h1>
          {document.derived && (
            <span className="shrink-0 rounded-8 bg-surf-4 px-[5px] py-0.5 font-mono text-[10px] text-tx-1">
              {t('document.derived')}
            </span>
          )}
        </div>
      </div>
      <div className="ml-auto flex shrink-0 gap-1.5">
        <ConvertMenu
          conversions={document.conversions}
          open={openMenu === 'convert'}
          onOpenChange={toggle('convert')}
          onConvert={actions.onConvert}
        />
        <MoreMenu {...actions} open={openMenu === 'more'} onOpenChange={toggle('more')} />
      </div>
    </header>
  )
}
