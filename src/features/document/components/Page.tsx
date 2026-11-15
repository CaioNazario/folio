import type { CSSProperties, ReactNode } from 'react'

// Paper page (Design/specs/05-document-view.md, 5.5). Typography scales with
// the page width (container queries) times the zoom (--zoom). MD and TXT
// scroll inside the page.
export function Page({ zoom, children }: { zoom: number; children: ReactNode }) {
  const style = { '--zoom': zoom / 100 } as CSSProperties
  return (
    <article
      style={style}
      className="@container flex h-[97%] w-[97%] flex-col overflow-y-auto rounded-7 border border-paper-edge bg-paper px-[4.6cqw] py-[3.6cqw] font-serif text-paper-tx shadow-page"
    >
      {children}
    </article>
  )
}
