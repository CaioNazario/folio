import type { ReactNode } from 'react'

// Empty text of a sidebar section (Design/specs/03-sidebar.md, "Favoritos").
export function EmptyNote({ children }: { children: ReactNode }) {
  return <p className="m-0 px-2 pt-0.5 pb-1 text-[12px] leading-[1.45] text-tx-10">{children}</p>
}
