import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import { useFormat } from '@/lib/i18n/useFormat'
import { EmptyNote } from './EmptyNote'
import { TreeLevelView, type TreeLevelProps } from './TreeLevelView'

// Project tree (Design/specs/03-sidebar.md, "Projetos"). The header counts
// the projects at the workspace root.
export function ProjectsSection(props: Omit<TreeLevelProps, 'level'>) {
  const { t } = useTranslation()
  const format = useFormat()
  const { root, loaded } = props.tree
  const empty = loaded && root.folders.length === 0 && root.documents.length === 0
  return (
    <section aria-labelledby="sidebar-projects">
      <div className="flex items-center justify-between px-2 pt-[10px] pb-1.5">
        <SectionLabel id="sidebar-projects" tone="sidebar">
          {t('sidebar.projects')}
        </SectionLabel>
        <span className="font-mono text-[11px] text-tx-1">{format.number(root.folders.length)}</span>
      </div>
      {empty && <EmptyNote>{t('sidebar.projectsEmpty')}</EmptyNote>}
      <TreeLevelView {...props} level={root} />
    </section>
  )
}
