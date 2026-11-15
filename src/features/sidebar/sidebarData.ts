import type { DocumentFormat } from '@/features/document/types'
import type { DirContents } from '@/lib/bindings/DirContents'
import type { Favorite } from '@/lib/bindings/Favorite'
import type { RecentDocument } from '@/lib/bindings/RecentDocument'
import type { RelativePath } from '@/lib/bindings/RelativePath'
import type { WorkspaceSummary as SummaryBinding } from '@/lib/bindings/WorkspaceSummary'
import type { FavoriteItem, PinTarget, RecentItem, TreeLevel, WorkspaceSummary } from './types'

// Paths are raw bytes: compared and keyed as given, never built here.
export const pathKey = (path: RelativePath) => `dir:${path.join(',')}`

export const pinKey = (target: PinTarget) => (target.kind === 'document' ? `doc:${target.id}` : pathKey(target.path))

export function toTreeLevel(contents: DirContents, folder: string | null): TreeLevel {
  return {
    folders: contents.folders.map((entry) => ({
      key: pathKey(entry.path),
      path: entry.path,
      name: entry.name,
      count: entry.documentCount,
      otherWorkspace: entry.kind === 'OtherWorkspace',
    })),
    documents: contents.documents.map(({ id, name, format, valid }) => ({ id, name, format, valid, folder })),
  }
}

export function toFavoriteItem({ name, target }: Favorite): FavoriteItem {
  const pin: PinTarget = target.kind === 'Document' ? { kind: 'document', id: target.id } : { kind: 'folder', path: target.path }
  return { key: pinKey(pin), name, target: pin }
}

// Recents carry no format: it comes from the name until the reader's open
// command returns the document.
export function formatFromName(name: string): DocumentFormat {
  const extension = name.split('.').pop()?.toLowerCase()
  return extension === 'pdf' || extension === 'docx' || extension === 'md' ? extension : 'txt'
}

export function toRecentItem(recent: RecentDocument): RecentItem {
  const { id, name, project, openedAt } = recent
  return { id, name, folder: project, format: formatFromName(name), openedAt: new Date(openedAt) }
}

export function toWorkspaceSummary({ location, userName, totals }: SummaryBinding): WorkspaceSummary {
  return { path: location, userName, fileCount: totals.documents, totalBytes: totals.size }
}
