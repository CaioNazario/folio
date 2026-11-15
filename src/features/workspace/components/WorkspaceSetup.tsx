import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Logo } from '@/components/ui/logo'
import { errorMessage } from '@/lib/i18n/errors'
import { useWorkspaceSetup } from '../useWorkspaceSetup'

// First run: choose where the workspace lives (specs/03-workspace.md,
// "Primeira execução"). Same card language as the rest of the app.
export function WorkspaceSetup({ onOpened }: { onOpened: () => void }) {
  const { t } = useTranslation()
  const setup = useWorkspaceSetup(onOpened)
  const nested = setup.error?.kind === 'NestedWorkspace' ? setup.error.path : null
  return (
    <div className="flex h-screen items-center justify-center bg-bg p-6 font-sans text-tx-2">
      <section aria-labelledby="setup-title" className="flex w-full max-w-[460px] flex-col gap-[14px] rounded-18 border border-line-4 bg-surf-2 p-[18px]">
        <span className="text-tx-1">
          <Logo />
        </span>
        <h1 id="setup-title" className="m-0 text-[15px] font-semibold text-tx-1">
          {t('workspace.setup.title')}
        </h1>
        <p className="m-0 text-[12px] leading-[1.45] text-tx-8">{t('workspace.setup.description')}</p>
        <div className="truncate rounded-14 border border-surf-4 bg-surf-1 px-[10px] py-[7px] font-mono text-[12px] text-tx-1">
          {setup.path ?? '…'}
        </div>
        {setup.error && (
          <p role="alert" className="m-0 text-[12px] leading-[1.45] text-danger">
            {errorMessage(setup.error)}
          </p>
        )}
        <div className="flex justify-end gap-1.5">
          {nested && <Button onClick={() => setup.openAncestor(nested)}>{t('workspace.setup.openAncestor')}</Button>}
          <Button onClick={setup.choose}>{t('workspace.setup.choose')}</Button>
          <Button onClick={setup.create} disabled={!setup.path} className="text-tx-1">
            {t('workspace.setup.create')}
          </Button>
        </div>
      </section>
    </div>
  )
}
