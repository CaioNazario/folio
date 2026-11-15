import { useCallback, useEffect, useState } from 'react'
import { onScanFinished } from '@/features/workspace/api'
import { useReportError } from '@/lib/useReportError'
import { getFavorites, getRecentDocuments, getWorkspaceSummary, toggleFavoriteDir, toggleFavoriteDocument } from './api'
import { toFavoriteItem, toRecentItem, toWorkspaceSummary } from './sidebarData'
import type { FavoriteItem, PinTarget, RecentItem, WorkspaceSummary } from './types'
import { useTree } from './useTree'

// Favorites, recents and the footer summary, each with its own reload.
function useSidebarLists() {
  const report = useReportError()
  const [favorites, setFavorites] = useState<FavoriteItem[]>([])
  const [recents, setRecents] = useState<RecentItem[]>([])
  const [workspace, setWorkspace] = useState<WorkspaceSummary | null>(null)
  const reloadFavorites = useCallback(() => getFavorites().then((list) => setFavorites(list.map(toFavoriteItem))).catch(report), [report])
  const reloadRecents = useCallback(() => getRecentDocuments().then((list) => setRecents(list.map(toRecentItem))).catch(report), [report])
  const reloadSummary = useCallback(() => getWorkspaceSummary().then((summary) => setWorkspace(toWorkspaceSummary(summary))).catch(report), [report])
  return { data: { favorites, recents, workspace }, reloadFavorites, reloadRecents, reloadSummary }
}

// Loads now and again after every scan.
function useScanFinished(reload: () => void) {
  useEffect(() => {
    reload()
    const unlisten = onScanFinished(reload)
    return () => void unlisten.then((stop) => stop())
  }, [reload])
}

// Sidebar data from the workspace commands. Everything reloads when a scan
// finishes; favorites after a pin, recents after a document opens.
export function useSidebar() {
  const report = useReportError()
  const tree = useTree()
  const { reload: reloadTree } = tree
  const { data, reloadFavorites, reloadRecents, reloadSummary } = useSidebarLists()
  const reload = useCallback(() => {
    reloadTree()
    void Promise.all([reloadFavorites(), reloadRecents(), reloadSummary()])
  }, [reloadTree, reloadFavorites, reloadRecents, reloadSummary])
  useScanFinished(reload)
  const togglePin = (target: PinTarget) =>
    (target.kind === 'document' ? toggleFavoriteDocument(target.id) : toggleFavoriteDir(target.path)).then(reloadFavorites).catch(report)
  return { data, tree, togglePin, reloadRecents }
}
