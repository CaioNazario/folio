import { createContext, useContext } from 'react'

export const TOAST_DURATION_MS = 2600

export type ToastContextValue = {
  message: string | null
  show: (message: string) => void
}

export const ToastContext = createContext<ToastContextValue | null>(null)

// Shows a toast from anywhere under <ToastProvider>.
export function useToast(): ToastContextValue {
  const context = useContext(ToastContext)
  if (!context) throw new Error('useToast must be used inside <ToastProvider>')
  return context
}
