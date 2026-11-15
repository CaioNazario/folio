import { Plus, Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'

// Import button and name search (Design/specs/03-sidebar.md, 3.2). Search
// results come with the search command (specs/06).
export function SidebarActions({ onImport }: { onImport: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col gap-2 px-3 pt-3 pb-2">
      <Button variant="import" onClick={onImport}>
        <Plus size={15} strokeWidth={3} className="text-plus" />
        {t('sidebar.import')}
      </Button>
      <label className="flex items-center gap-2 rounded-14 border border-surf-4 bg-surf-1 px-[10px] py-[7px]">
        <Search size={13} aria-hidden="true" className="shrink-0 text-tx-1" />
        <input
          type="search"
          placeholder={t('sidebar.search')}
          aria-label={t('sidebar.search')}
          className="min-w-0 flex-1 border-none bg-transparent text-[13px] text-tx-1 outline-none placeholder:text-tx-8"
        />
      </label>
    </div>
  )
}
