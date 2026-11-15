import { useCallback, useState } from 'react'

// Expanded projects in the tree. Toggled by the project row, a pinned folder
// and the Home project cards.
export function useOpenProjects(initial: Iterable<string> = []) {
  const [openIds, setOpenIds] = useState<ReadonlySet<string>>(() => new Set(initial))

  const toggle = useCallback((id: string) => {
    setOpenIds((current) => {
      const next = new Set(current)
      if (!next.delete(id)) next.add(id)
      return next
    })
  }, [])

  return { openIds, toggle }
}
