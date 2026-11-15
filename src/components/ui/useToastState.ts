import { useCallback, useEffect, useRef, useState } from 'react'
import { TOAST_DURATION_MS, type ToastContextValue } from './toast-context'

// A new toast replaces the current one and restarts the dismiss timer.
export function useToastState(): ToastContextValue {
  const [message, setMessage] = useState<string | null>(null)
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const show = useCallback((text: string) => {
    clearTimeout(timer.current)
    setMessage(text)
    timer.current = setTimeout(() => setMessage(null), TOAST_DURATION_MS)
  }, [])

  useEffect(() => () => clearTimeout(timer.current), [])

  return { message, show }
}
