import { useCallback } from 'react'
import { useToast } from '@/components/ui/toast-context'
import { errorMessage, isBridgeError } from '@/lib/i18n/errors'

// Shows a backend error in a toast, in the active language; anything else
// goes to the log only.
export function useReportError() {
  const { show } = useToast()
  return useCallback(
    (error: unknown) => {
      if (isBridgeError(error)) show(errorMessage(error))
      else console.error(error)
    },
    [show],
  )
}
