import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import type { RecentItem } from '@/features/sidebar/types'
import { useFormat } from '@/lib/i18n/useFormat'
import type { HomeActions } from '../types'

type ContinueReadingProps = Pick<HomeActions, 'onOpenDocument' | 'onSeeAllRecents'> & {
  recents: RecentItem[]
  now: Date
}

// "Continuar de onde parou" (Design/specs/04-home.md): the sidebar's recents,
// with project and size. A click opens the document like in the sidebar.
export function ContinueReading({ recents, now, onOpenDocument, onSeeAllRecents }: ContinueReadingProps) {
  const { t } = useTranslation()
  const format = useFormat()
  const meta = (item: RecentItem) =>
    item.project ? t('home.recentMeta', { project: item.project, size: format.size(item.size) }) : format.size(item.size)
  return (
    <section aria-labelledby="home-continue" className="mt-10">
      <div className="flex items-baseline justify-between">
        <SectionLabel id="home-continue" tone="home">{t('home.continue')}</SectionLabel>
        <button type="button" onClick={onSeeAllRecents} className="cursor-pointer text-[12px] text-tx-8">
          {t('home.seeAll')}
        </button>
      </div>
      {recents.length > 0 && (
        <ul className="mt-3 overflow-hidden rounded-18 border border-line-4">
          {recents.map((item) => (
            <li key={item.id} className="border-b border-line-3 last:border-b-0">
              <button
                type="button"
                onClick={() => onOpenDocument({ id: item.id, name: item.name })}
                className="flex w-full cursor-pointer items-baseline gap-3 bg-surf-2 px-[14px] py-3 text-left hover:bg-hover-2"
              >
                <span className="min-w-0 truncate text-[13px] text-tx-2">{item.name}</span>
                <span className="text-[12px] whitespace-nowrap text-tx-8">{meta(item)}</span>
                <span className="ml-auto font-mono text-[11px] whitespace-nowrap text-tx-10">
                  {format.relativeTime(item.openedAt, now)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
