import { useCallback, useEffect, useRef, useState } from 'react'
import { TOAST_DURATION_MS, type Toast, type ToastContextValue, type ToastOptions } from './toast-context'

// A new toast replaces the current one and restarts the dismiss timer; a
// sticky one has no timer.
export function useToastState(): ToastContextValue {
  const [toast, setToast] = useState<Toast | null>(null)
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const dismiss = useCallback(() => {
    clearTimeout(timer.current)
    setToast(null)
  }, [])

  const show = useCallback((message: string, options: ToastOptions = {}) => {
    clearTimeout(timer.current)
    setToast({ message, action: options.action })
    if (!options.sticky) timer.current = setTimeout(() => setToast(null), TOAST_DURATION_MS)
  }, [])

  useEffect(() => () => clearTimeout(timer.current), [])

  return { toast, show, dismiss }
}
