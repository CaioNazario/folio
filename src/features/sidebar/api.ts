import { invoke } from '@tauri-apps/api/core'
import type { DirContents } from '@/lib/bindings/DirContents'
import type { Favorite } from '@/lib/bindings/Favorite'
import type { RecentDocument } from '@/lib/bindings/RecentDocument'
import type { RelativePath } from '@/lib/bindings/RelativePath'
import type { WorkspaceSummary } from '@/lib/bindings/WorkspaceSummary'

export const RECENTS_LIMIT = 7

// What is directly inside `parent`; null is the workspace root.
export function listDir(parent: RelativePath | null): Promise<DirContents> {
  return invoke<DirContents>('list_dir', { parent })
}

export function getFavorites(): Promise<Favorite[]> {
  return invoke<Favorite[]>('get_favorites')
}

export function toggleFavoriteDocument(id: string): Promise<boolean> {
  return invoke<boolean>('toggle_favorite_document', { id })
}

export function toggleFavoriteDir(path: RelativePath): Promise<boolean> {
  return invoke<boolean>('toggle_favorite_dir', { path })
}

export function getRecentDocuments(): Promise<RecentDocument[]> {
  return invoke<RecentDocument[]>('get_recent_documents', { limit: RECENTS_LIMIT })
}

export function getWorkspaceSummary(): Promise<WorkspaceSummary> {
  return invoke<WorkspaceSummary>('get_workspace_summary')
}
