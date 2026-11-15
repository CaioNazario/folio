import type { RelativePath } from '@/lib/bindings/RelativePath'

// View models the sidebar renders, mapped from the ts-rs bindings in
// sidebarData.ts. Directories have no stable id: they are keyed by path
// (specs/03-workspace.md, "Ícones").

export type DocumentRef = { id: string; name: string }

export type FolderRef = { path: RelativePath; name: string }

export type PinTarget = { kind: 'document'; id: string } | { kind: 'folder'; path: RelativePath }

export type FavoriteItem = { key: string; name: string; target: PinTarget }

export type TreeFolder = FolderRef & { key: string; count: number; otherWorkspace: boolean }

export type TreeDocument = DocumentRef & { valid: boolean }

export type TreeLevel = { folders: TreeFolder[]; documents: TreeDocument[] }

/** `project` is null at the workspace root. */
export type RecentItem = DocumentRef & { project: string | null; size: number; openedAt: Date }

export type WorkspaceSummary = { path: string; userName: string; fileCount: number; totalBytes: number }

export type SidebarData = {
  favorites: FavoriteItem[]
  recents: RecentItem[]
  /** False until the first answer: the empty text waits for it. */
  recentsLoaded: boolean
  workspace: WorkspaceSummary | null
}

export type TreeView = {
  root: TreeLevel
  /** False until the root is listed: the empty text waits for it. */
  loaded: boolean
  children: ReadonlyMap<string, TreeLevel>
  openKeys: ReadonlySet<string>
  toggle: (folder: FolderRef) => void
}

export type SidebarActions = {
  /** The logo goes back to the Home. */
  onHome: () => void
  onImport: () => void
  onOpenDocument: (document: DocumentRef) => void
  onTogglePin: (target: PinTarget) => void
}
