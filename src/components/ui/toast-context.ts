import { createContext, useContext } from 'react'

export const TOAST_DURATION_MS = 2600

/** A button inside the toast, such as "Cancelar". */
export type ToastAction = { label: string; onClick: () => void }

export type ToastOptions = {
  /** Stays until another toast replaces it or it is dismissed. */
  sticky?: boolean
  action?: ToastAction
}

export type Toast = { message: string; action?: ToastAction }

export type ToastContextValue = {
  toast: Toast | null
  show: (message: string, options?: ToastOptions) => void
  dismiss: () => void
}

export const ToastContext = createContext<ToastContextValue | null>(null)

// Shows a toast from anywhere under <ToastProvider>.
export function useToast(): ToastContextValue {
  const context = useContext(ToastContext)
  if (!context) throw new Error('useToast must be used inside <ToastProvider>')
  return context
}
