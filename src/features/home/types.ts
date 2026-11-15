import type { DocumentRef, FolderRef, RecentItem, TreeFolder } from '@/features/sidebar/types'

// What the Home shows: the sidebar's recents and the workspace projects.
export type HomeData = {
  recents: RecentItem[]
  projects: TreeFolder[]
}

// The action cards and "Ver tudo" wait for their commands (specs 08 and 09).
export type HomeActions = {
  onImport: () => void
  onConvert: () => void
  onNewProject: () => void
  onSeeAllRecents: () => void
  onOpenDocument: (document: DocumentRef) => void
  /** Opens or closes the project in the sidebar tree; the Home stays. */
  onToggleProject: (project: FolderRef) => void
}
