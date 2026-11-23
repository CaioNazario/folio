import { type ReactNode, useContext, useMemo } from 'react'
import { ToastContext } from './toast-context'
import { useToastState } from './useToastState'

export function ToastProvider({ children }: { children: ReactNode }) {
  const { toast, show, dismiss } = useToastState()
  const value = useMemo(() => ({ toast, show, dismiss }), [toast, show, dismiss])
  return <ToastContext.Provider value={value}>{children}</ToastContext.Provider>
}

// Toast (Design/specs/06-components.md), anchored to the nearest positioned
// ancestor: the main area. An action shows as a text button at the end.
export function ToastViewport() {
  const toast = useContext(ToastContext)?.toast
  return (
    <div role="status" aria-live="polite" className="contents">
      {toast && (
        <div className="absolute bottom-[26px] left-1/2 z-60 flex -translate-x-1/2 items-center gap-[11px] rounded-18 border border-bar bg-pop px-[14px] py-[10px] text-[12.5px] text-tx-2 shadow-pop">
          <span aria-hidden="true" className="size-[7px] shrink-0 rounded-full bg-teal" />
          {toast.message}
          {toast.action && (
            <button
              type="button"
              onClick={toast.action.onClick}
              className="cursor-pointer rounded-10 px-1.5 py-0.5 font-medium text-tx-1 outline-none hover:bg-hover-3 focus-visible:bg-hover-3"
            >
              {toast.action.label}
            </button>
          )}
        </div>
      )}
    </div>
  )
}
