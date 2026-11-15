import { Folder } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import type { TreeFolder } from '@/features/sidebar/types'
import type { HomeActions } from '../types'
import { HomeCard } from './HomeCard'

type ProjectCardsProps = Pick<HomeActions, 'onToggleProject'> & { projects: TreeFolder[] }

// Project cards (Design/specs/04-home.md, "Projetos"). A click opens or
// closes the project in the sidebar, without leaving the Home.
export function ProjectCards({ projects, onToggleProject }: ProjectCardsProps) {
  const { t } = useTranslation()
  return (
    <section aria-labelledby="home-projects" className="mt-9">
      <SectionLabel id="home-projects" tone="home">{t('home.projects')}</SectionLabel>
      <div className="mt-3 grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-[10px]">
        {projects.map((project) => (
          <HomeCard key={project.key} onClick={() => onToggleProject(project)}>
            <Folder size={16} aria-hidden="true" className="text-icon" />
            <span className="mt-3 block truncate text-[13px] font-medium">{project.name}</span>
            <span className="mt-0.5 block font-mono text-[11px] text-tx-10">
              {t('home.documents', { count: project.count })}
            </span>
          </HomeCard>
        ))}
      </div>
    </section>
  )
}
