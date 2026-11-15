import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import { useFormat } from '@/lib/i18n/useFormat'
import type { ProjectItem, SidebarActions } from '../types'
import { FileRow } from './FileRow'
import { ProjectRow } from './ProjectRow'

type ProjectsSectionProps = Pick<SidebarActions, 'onOpenDocument' | 'onTogglePin'> & {
  projects: ProjectItem[]
  openIds: ReadonlySet<string>
  pinnedIds: ReadonlySet<string>
  selectedId: string | null
  onToggleProject: (id: string) => void
}

// Project tree (Design/specs/03-sidebar.md, "Projetos").
export function ProjectsSection(props: ProjectsSectionProps) {
  const { projects, openIds, pinnedIds, selectedId, onToggleProject, onOpenDocument, onTogglePin } = props
  const { t } = useTranslation()
  const format = useFormat()
  return (
    <section aria-labelledby="sidebar-projects">
      <div className="flex items-center justify-between px-2 pt-[10px] pb-1.5">
        <SectionLabel id="sidebar-projects" tone="sidebar">
          {t('sidebar.projects')}
        </SectionLabel>
        <span className="font-mono text-[11px] text-tx-1">{format.number(projects.length)}</span>
      </div>
      {projects.map((project) => (
        <div key={project.id}>
          <ProjectRow
            project={project}
            open={openIds.has(project.id)}
            pinned={pinnedIds.has(project.id)}
            onToggle={() => onToggleProject(project.id)}
            onTogglePin={() => onTogglePin(project.id, 'folder')}
          />
          {openIds.has(project.id) &&
            project.files.map((file) => (
              <FileRow
                key={file.id}
                file={file}
                selected={file.id === selectedId}
                pinned={pinnedIds.has(file.id)}
                onOpen={() => onOpenDocument(file.id)}
                onTogglePin={() => onTogglePin(file.id, 'file')}
              />
            ))}
        </div>
      ))}
    </section>
  )
}
