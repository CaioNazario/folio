import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { StageMessage } from './StageMessage'

// "Falha ao exibir" (specs/07-reader.md, "Estados do palco"): try again or
// show the file in its folder.
export function StageFailed({ onRetry, showInFolder }: { onRetry: () => void; showInFolder: ReactNode }) {
  const { t } = useTranslation()
  return (
    <StageMessage
      actions={
        <>
          <Button onClick={onRetry}>{t('document.stage.retry')}</Button>
          {showInFolder}
        </>
      }
    >
      {t('document.stage.failed')}
    </StageMessage>
  )
}
