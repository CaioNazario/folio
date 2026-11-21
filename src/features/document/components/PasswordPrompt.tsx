import { type SubmitEvent, useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

type PasswordPromptProps = {
  /** The last password was rejected. */
  wrong: boolean
  onSubmit: (password: string) => void
}

// Stage state "Senha" (specs/07-reader.md, "Estados do palco"): the field
// and "Abrir" on the stage itself; a wrong password shows on the field.
export function PasswordPrompt({ wrong, onSubmit }: PasswordPromptProps) {
  const { t } = useTranslation()
  const [password, setPassword] = useState('')
  const errorId = useId()
  const submit = (event: SubmitEvent<HTMLFormElement>) => {
    event.preventDefault()
    onSubmit(password)
  }
  return (
    <form onSubmit={submit} className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
      <p className="m-0 text-[13px] text-tx-8">{t('errors.PasswordRequired')}</p>
      <div className="flex flex-col gap-1.5">
        <Input
          type="password"
          autoFocus
          aria-label={t('document.password.label')}
          aria-invalid={wrong}
          aria-describedby={wrong ? errorId : undefined}
          value={password}
          onChange={(event) => setPassword(event.target.value)}
        />
        {wrong && (
          <p id={errorId} className="m-0 text-[12px] text-danger">
            {t('errors.WrongPassword')}
          </p>
        )}
      </div>
      <Button type="submit" disabled={password === ''}>
        {t('document.password.open')}
      </Button>
    </form>
  )
}
