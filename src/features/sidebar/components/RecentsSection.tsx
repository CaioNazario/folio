import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import { useFormat } from '@/lib/i18n/useFormat'
import type { RecentItem, SidebarActions } from '../types'
import { pinKey } from '../sidebarData'
import { EmptyNote } from './EmptyNote'
import { NavRow, RowName } from './NavRow'
import { PinButton } from './PinButton'

type RecentsSectionProps = Pick<SidebarActions, 'onOpenDocument' | 'onTogglePin'> & {
  recents: RecentItem[]
  /** The empty text waits for the first answer. */
  loaded: boolean
  pinnedKeys: ReadonlySet<string>
  now: Date
}

// Recents (Design/specs/03-sidebar.md): no icon, relative time on the right.
export function RecentsSection({ recents, loaded, pinnedKeys, now, onOpenDocument, onTogglePin }: RecentsSectionProps) {
  const { t } = useTranslation()
  const format = useFormat()
  return (
    <section aria-labelledby="sidebar-recents">
      <SectionLabel id="sidebar-recents" tone="sidebar" className="px-2 pt-4 pb-1.5">
        {t('sidebar.recents')}
      </SectionLabel>
      {loaded && recents.length === 0 && <EmptyNote>{t('sidebar.recentsEmpty')}</EmptyNote>}
      {recents.map((item) => (
        <NavRow
          key={item.id}
          label={item.name}
          onClick={() => onOpenDocument({ id: item.id, name: item.name })}
          trailing={
            <>
              <PinButton
                pinned={pinnedKeys.has(pinKey({ kind: 'document', id: item.id }))}
                onToggle={() => onTogglePin({ kind: 'document', id: item.id })}
              />
              <span className="font-mono text-[10px] whitespace-nowrap text-tx-1">
                {format.relativeTime(item.openedAt, now)}
              </span>
            </>
          }
        >
          <RowName>{item.name}</RowName>
        </NavRow>
      ))}
    </section>
  )
}
