import { useCallback, useRef, useState } from 'react'
import { useReportError } from '@/lib/useReportError'
import { listDir } from './api'
import { pathKey, toTreeLevel } from './sidebarData'
import type { FolderRef, TreeLevel } from './types'

const EMPTY: TreeLevel = { folders: [], documents: [] }

// Contents of the opened folders, keyed by path.
function useFolderContents() {
  const report = useReportError()
  const [children, setChildren] = useState<ReadonlyMap<string, TreeLevel>>(new Map())
  const load = useCallback(
    (folder: FolderRef) =>
      listDir(folder.path)
        .then((contents) => setChildren((map) => new Map(map).set(pathKey(folder.path), toTreeLevel(contents, folder.name))))
        .catch(report),
    [report],
  )
  return { children, load }
}

// Which folders are open, keeping each one's path to reload it.
function useOpenFolders(load: (folder: FolderRef) => Promise<unknown>) {
  const opened = useRef(new Map<string, FolderRef>())
  const [openKeys, setOpenKeys] = useState<ReadonlySet<string>>(new Set())
  const toggle = useCallback(
    (folder: FolderRef) => {
      const key = pathKey(folder.path)
      const wasOpen = opened.current.delete(key)
      if (!wasOpen) opened.current.set(key, folder)
      setOpenKeys(new Set(opened.current.keys()))
      if (!wasOpen) void load(folder)
    },
    [load],
  )
  const reloadOpen = useCallback(() => opened.current.forEach((folder) => void load(folder)), [load])
  return { openKeys, toggle, reloadOpen }
}

// Project tree, loaded lazily: the root at once, each folder when opened.
export function useTree() {
  const report = useReportError()
  const [root, setRoot] = useState<TreeLevel>(EMPTY)
  const { children, load } = useFolderContents()
  const { openKeys, toggle, reloadOpen } = useOpenFolders(load)
  const reload = useCallback(() => {
    listDir(null).then((contents) => setRoot(toTreeLevel(contents, null))).catch(report)
    reloadOpen()
  }, [reloadOpen, report])
  return { root, children, openKeys, toggle, reload }
}
