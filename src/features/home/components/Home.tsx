import { useTranslation } from 'react-i18next'
import type { HomeActions, HomeData } from '../types'
import { ContinueReading } from './ContinueReading'
import { ProjectCards } from './ProjectCards'
import { QuickActions } from './QuickActions'

type HomeProps = { data: HomeData; actions: HomeActions; now: Date }

// Home, "Documentos" (Design/specs/04-home.md): the main area when no
// document is open.
export function Home({ data, actions, now }: HomeProps) {
  const { t } = useTranslation()
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto max-w-[880px] px-10 pt-[72px] pb-[60px]">
        <h1 className="m-0 text-[26px] font-semibold tracking-[-0.01em] text-tx-1">{t('home.title')}</h1>
        <p className="mt-1.5 mb-0 text-[14px] text-tx-7">{t('home.subtitle')}</p>
        <QuickActions onImport={actions.onImport} onConvert={actions.onConvert} onNewProject={actions.onNewProject} />
        <ContinueReading
          recents={data.recents}
          now={now}
          onOpenDocument={actions.onOpenDocument}
          onSeeAllRecents={actions.onSeeAllRecents}
        />
        {data.projects.length > 0 && <ProjectCards projects={data.projects} onToggleProject={actions.onToggleProject} />}
      </div>
    </div>
  )
}
