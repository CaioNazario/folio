import { useTranslation } from 'react-i18next'
import { pinKey } from '../sidebarData'
import type { SidebarActions, TreeLevel, TreeView } from '../types'
import { FileRow } from './FileRow'
import { ProjectRow } from './ProjectRow'

export type TreeLevelProps = Pick<SidebarActions, 'onOpenDocument' | 'onTogglePin'> & {
  level: TreeLevel
  tree: TreeView
  pinnedKeys: ReadonlySet<string>
  selectedId: string | null
}

// One level of the tree: folders (expanding into a guided child level), then
// documents.
export function TreeLevelView(props: TreeLevelProps) {
  const { level, tree, pinnedKeys, selectedId, onOpenDocument, onTogglePin } = props
  const { t } = useTranslation()
  return (
    <>
      {level.folders.map((folder) => {
        const open = tree.openKeys.has(folder.key)
        const children = tree.children.get(folder.key)
        return (
          <div key={folder.key}>
            <ProjectRow
              folder={folder}
              open={open}
              pinned={pinnedKeys.has(folder.key)}
              onToggle={() => tree.toggle(folder)}
              onTogglePin={() => onTogglePin({ kind: 'folder', path: folder.path })}
            />
            {open && (
              <div className="ml-[7px] border-l border-line-3 pl-px">
                {folder.otherWorkspace && <p className="m-0 py-[5px] pl-[25px] text-[12px] text-tx-8">{t('sidebar.otherWorkspace')}</p>}
                {children && <TreeLevelView {...props} level={children} />}
              </div>
            )}
          </div>
        )
      })}
      {level.documents.map((document) => (
        <FileRow
          key={document.id}
          document={document}
          selected={document.id === selectedId}
          pinned={pinnedKeys.has(pinKey({ kind: 'document', id: document.id }))}
          onOpen={() => onOpenDocument({ id: document.id, name: document.name })}
          onTogglePin={() => onTogglePin({ kind: 'document', id: document.id })}
        />
      ))}
    </>
  )
}
