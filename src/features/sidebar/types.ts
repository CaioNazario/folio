import type { DocumentFormat } from '@/features/document/types'
import type { RelativePath } from '@/lib/bindings/RelativePath'

// View models the sidebar renders, mapped from the ts-rs bindings in
// sidebarData.ts. Directories have no stable id: they are keyed by path
// (specs/03-workspace.md, "Ícones").

export type DocumentRef = { id: string; name: string; format: DocumentFormat; folder: string | null }

export type FolderRef = { path: RelativePath; name: string }

export type PinTarget = { kind: 'document'; id: string } | { kind: 'folder'; path: RelativePath }

export type FavoriteItem = { key: string; name: string; target: PinTarget }

export type TreeFolder = FolderRef & { key: string; count: number; otherWorkspace: boolean }

export type TreeDocument = DocumentRef & { valid: boolean }

export type TreeLevel = { folders: TreeFolder[]; documents: TreeDocument[] }

export type RecentItem = DocumentRef & { openedAt: Date }

export type WorkspaceSummary = { path: string; userName: string; fileCount: number; totalBytes: number }

export type SidebarData = {
  favorites: FavoriteItem[]
  recents: RecentItem[]
  workspace: WorkspaceSummary | null
}

export type TreeView = {
  root: TreeLevel
  children: ReadonlyMap<string, TreeLevel>
  openKeys: ReadonlySet<string>
  toggle: (folder: FolderRef) => void
}

export type SidebarActions = {
  onImport: () => void
  onOpenDocument: (document: DocumentRef) => void
  onTogglePin: (target: PinTarget) => void
}
