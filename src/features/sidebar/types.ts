// View models the sidebar renders. They are UI props, not backend types: the
// sidebar api.ts maps the ts-rs bindings into them once the commands exist.

export type FavoriteItem = { id: string; name: string; kind: 'file' | 'folder' }

export type ProjectFile = { id: string; name: string }

export type ProjectItem = { id: string; name: string; count: number; files: ProjectFile[] }

export type RecentItem = { id: string; name: string; openedAt: Date }

// path is shown as the user sees it ("~/Documentos"); userName feeds the avatar.
export type WorkspaceSummary = { path: string; userName: string; fileCount: number; totalBytes: number }

export type SidebarData = {
  favorites: FavoriteItem[]
  projects: ProjectItem[]
  recents: RecentItem[]
  workspace: WorkspaceSummary | null
}

export type SidebarActions = {
  onImport: () => void
  onOpenDocument: (id: string) => void
  onTogglePin: (id: string, kind: FavoriteItem['kind']) => void
}
