import { useEffect, useState } from 'react'
import type { BridgeError } from '@/lib/bindings/BridgeError'
import { isBridgeError } from '@/lib/i18n/errors'
import { createWorkspace, getDefaultWorkspaceDir, openWorkspace, pickDirectory } from './api'

// The directory to create, starting at the system default.
function useChosenDirectory() {
  const [path, setPath] = useState<string | null>(null)
  useEffect(() => {
    getDefaultWorkspaceDir().then(setPath).catch(console.error)
  }, [])
  return { path, setPath }
}

// The choose-root dialog: default directory preselected, the system picker
// for another one, and the backend's reason when a choice is refused.
export function useWorkspaceSetup(onOpened: () => void) {
  const { path, setPath } = useChosenDirectory()
  const [error, setError] = useState<BridgeError | null>(null)
  const run = (action: Promise<void>) =>
    action.then(onOpened).catch((reason: unknown) => (isBridgeError(reason) ? setError(reason) : console.error(reason)))
  const choose = async () => {
    const chosen = await pickDirectory(path ?? '')
    if (!chosen) return
    setPath(chosen)
    setError(null)
  }
  return {
    path,
    error,
    choose,
    create: () => path && run(createWorkspace(path)),
    openAncestor: (ancestor: string) => run(openWorkspace(ancestor)),
  }
}
