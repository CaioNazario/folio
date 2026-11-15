import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { SidebarActions, SidebarData } from '../types'
import { useOpenProjects } from '../useOpenProjects'
import { FavoritesSection } from './FavoritesSection'
import { Logo } from './Logo'
import { ProjectsSection } from './ProjectsSection'
import { RecentsSection } from './RecentsSection'
import { SidebarActions as Actions } from './SidebarActions'
import { SidebarFooter } from './SidebarFooter'

type SidebarProps = { data: SidebarData; actions: SidebarActions; selectedId: string | null; now: Date }

// Sidebar column (Design/specs/02-layout.md and 03-sidebar.md).
export function Sidebar({ data, actions, selectedId, now }: SidebarProps) {
  const { t } = useTranslation()
  const { openIds, toggle } = useOpenProjects()
  const pinnedIds = useMemo(() => new Set(data.favorites.map((item) => item.id)), [data.favorites])
  const shared = { pinnedIds, onOpenDocument: actions.onOpenDocument, onTogglePin: actions.onTogglePin }
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
          onOpenFile={actions.onOpenDocument}
          onToggleFolder={toggle}
          onTogglePin={actions.onTogglePin}
        />
        <hr className="mx-2 mt-3 mb-0.5 h-px border-none bg-line-2" />
        <ProjectsSection {...shared} projects={data.projects} openIds={openIds} selectedId={selectedId} onToggleProject={toggle} />
        <RecentsSection {...shared} recents={data.recents} now={now} />
      </div>
      <SidebarFooter workspace={data.workspace} />
    </aside>
  )
}
