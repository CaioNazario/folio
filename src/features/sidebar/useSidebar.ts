import type { SidebarActions, SidebarData } from './types'

const EMPTY: SidebarData = { favorites: [], projects: [], recents: [], workspace: null }

const NO_ACTIONS: SidebarActions = {
  onImport: () => {},
  onOpenDocument: () => {},
  onTogglePin: () => {},
}

// Sidebar data and actions. Empty until the workspace, favorites and recents
// commands exist; then this reads them through the feature's api.ts.
export function useSidebar() {
  return { data: EMPTY, actions: NO_ACTIONS, selectedId: null }
}
