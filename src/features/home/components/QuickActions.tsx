import { FilePlus, FolderPlus, type LucideIcon, Upload } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { HomeActions } from '../types'
import { HomeCard } from './HomeCard'

type QuickActionsProps = Pick<HomeActions, 'onImport' | 'onConvert' | 'onNewProject'>

type ActionKey = 'import' | 'convert' | 'newProject'

const ACTIONS: { key: ActionKey; icon: LucideIcon; handler: keyof QuickActionsProps }[] = [
  { key: 'import', icon: Upload, handler: 'onImport' },
  { key: 'convert', icon: FilePlus, handler: 'onConvert' },
  { key: 'newProject', icon: FolderPlus, handler: 'onNewProject' },
]

// Quick action cards (Design/specs/04-home.md, "Ações rápidas").
export function QuickActions(actions: QuickActionsProps) {
  const { t } = useTranslation()
  return (
    <div className="mt-8 grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-[10px]">
      {ACTIONS.map(({ key, icon: Icon, handler }) => (
        <HomeCard key={key} onClick={actions[handler]}>
          <span className="flex items-center gap-[9px] text-[13px] font-medium">
            <Icon size={15} aria-hidden="true" className="text-icon" />
            {t(`home.actions.${key}.title`)}
          </span>
          <span className="mt-[3px] block text-[12px] text-tx-8">{t(`home.actions.${key}.description`)}</span>
        </HomeCard>
      ))}
    </div>
  )
}
