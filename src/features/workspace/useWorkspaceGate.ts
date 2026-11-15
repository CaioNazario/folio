import { useCallback, useEffect, useState } from 'react'
import type { BridgeError } from '@/lib/bindings/BridgeError'
import { isBridgeError } from '@/lib/i18n/errors'
import { openLastWorkspace } from './api'

export type GateState =
  | { status: 'opening' }
  | { status: 'ready' }
  | { status: 'setup' }
  | { status: 'failed'; error: BridgeError | null }

function failure(error: unknown): GateState {
  return { status: 'failed', error: isBridgeError(error) ? error : null }
}

// Opens the last workspace at startup (specs/03-workspace.md). First run goes
// to setup; a workspace that is gone shows the error with retry.
export function useWorkspaceGate() {
  const [state, setState] = useState<GateState>({ status: 'opening' })
  const retry = useCallback(() => {
    openLastWorkspace()
      .then((opened) => setState({ status: opened ? 'ready' : 'setup' }))
      .catch((error: unknown) => setState(failure(error)))
  }, [])
  useEffect(retry, [retry])
  return {
    state,
    retry,
    chooseAnother: () => setState({ status: 'setup' }),
    opened: () => setState({ status: 'ready' }),
  }
}
