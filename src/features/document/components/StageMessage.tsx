import type { ReactNode } from 'react'

// Centered message for the stage states (specs/07-reader.md, "Estados do
// palco"), with optional actions below it.
export function StageMessage({ children, actions }: { children: ReactNode; actions?: ReactNode }) {
  return (
    <div role="status" className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
      <p className="m-0 text-[13px] text-tx-8">{children}</p>
      {actions && <div className="flex gap-1.5">{actions}</div>}
    </div>
  )
}
