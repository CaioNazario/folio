import { type ReactNode, useContext, useMemo } from 'react'
import { ToastContext } from './toast-context'
import { useToastState } from './useToastState'

export function ToastProvider({ children }: { children: ReactNode }) {
  const { message, show } = useToastState()
  const value = useMemo(() => ({ message, show }), [message, show])
  return <ToastContext.Provider value={value}>{children}</ToastContext.Provider>
}

// Toast (Design/specs/06-components.md), anchored to the nearest positioned
// ancestor: the main area.
export function ToastViewport() {
  const message = useContext(ToastContext)?.message
  return (
    <div role="status" aria-live="polite" className="contents">
      {message && (
        <div className="absolute bottom-[26px] left-1/2 z-60 flex -translate-x-1/2 items-center gap-[11px] rounded-18 border border-bar bg-pop px-[14px] py-[10px] text-[12.5px] text-tx-2 shadow-pop">
          <span aria-hidden="true" className="size-[7px] shrink-0 rounded-full bg-teal" />
          {message}
        </div>
      )}
    </div>
  )
}
