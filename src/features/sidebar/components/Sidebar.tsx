import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Logo } from '@/components/ui/logo'
import { formatFromName as formatOf } from '../sidebarData'
import type { SidebarActions, SidebarData, TreeView } from '../types'
import { FavoritesSection } from './FavoritesSection'
import { ProjectsSection } from './ProjectsSection'
import { RecentsSection } from './RecentsSection'
import { SidebarActions as Actions } from './SidebarActions'
import { SidebarFooter } from './SidebarFooter'

type SidebarProps = { data: SidebarData; tree: TreeView; actions: SidebarActions; selectedId: string | null; now: Date }

// Sidebar column (Design/specs/02-layout.md and 03-sidebar.md).
export function Sidebar({ data, tree, actions, selectedId, now }: SidebarProps) {
  const { t } = useTranslation()
  const pinnedKeys = useMemo(() => new Set(data.favorites.map((item) => item.key)), [data.favorites])
  const shared = { pinnedKeys, onOpenDocument: actions.onOpenDocument, onTogglePin: actions.onTogglePin }
  const openFavorite = (id: string, name: string) => {
    const known = data.recents.find((item) => item.id === id)
    actions.onOpenDocument(known ?? { id, name, format: formatOf(name), folder: null })
  }
  return (
    <aside
      aria-label={t('shell.sidebar')}
      className="flex min-w-[196px] flex-[0_0_min(272px,24vw)] flex-col border-r border-surf-4 bg-bg"
    >
      <header className="flex h-[46px] shrink-0 items-center gap-[9px] border-b border-line-1 px-[14px] text-tx-1">
        <Logo />
      </header>
      <Actions onImport={actions.onImport} />
      <div className="min-h-0 flex-1 overflow-y-auto px-2 pt-1.5 pb-3">
        <FavoritesSection
          favorites={data.favorites}
          onOpenFile={openFavorite}
          onToggleFolder={tree.toggle}
          onTogglePin={actions.onTogglePin}
        />
        <hr className="mx-2 mt-3 mb-0.5 h-px border-none bg-line-2" />
        <ProjectsSection {...shared} tree={tree} selectedId={selectedId} />
        <RecentsSection {...shared} recents={data.recents} now={now} />
      </div>
      <SidebarFooter workspace={data.workspace} />
    </aside>
  )
}
