import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { errorMessage } from '@/lib/i18n/errors'
import { useWorkspaceGate } from '../useWorkspaceGate'
import { WorkspaceSetup } from './WorkspaceSetup'

// Shows the app only once a workspace is open.
export function WorkspaceGate({ children }: { children: ReactNode }) {
  const { t } = useTranslation()
  const gate = useWorkspaceGate()
  const { state } = gate
  if (state.status === 'ready') return children
  if (state.status === 'setup') return <WorkspaceSetup onOpened={gate.opened} />
  if (state.status === 'opening') return <div className="h-screen bg-bg" />
  return (
    <div role="alert" className="flex h-screen flex-col items-center justify-center gap-3 bg-bg p-6 text-center font-sans">
      <p className="m-0 text-[13px] text-tx-8">
        {state.error ? errorMessage(state.error) : t('workspace.openFailed')}
      </p>
      <div className="flex gap-1.5">
        <Button onClick={gate.retry}>{t('workspace.retry')}</Button>
        <Button onClick={gate.chooseAnother}>{t('workspace.chooseAnother')}</Button>
      </div>
    </div>
  )
}
