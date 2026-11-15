import { type CSSProperties, type ReactNode, type UIEvent, useLayoutEffect, useRef } from 'react'

type PageProps = {
  zoom: number
  children: ReactNode
  /** Scroll fraction to restore when the page appears. */
  initialScroll?: number | null
  onScrollFraction?: (fraction: number) => void
}

function scrollFraction(element: HTMLElement): number {
  const range = element.scrollHeight - element.clientHeight
  return range > 0 ? Math.min(1, element.scrollTop / range) : 0
}

// Paper page (Design/specs/05-document-view.md, 5.5). Typography scales with
// the page width (container queries) times the zoom (--zoom). MD and TXT
// scroll inside the page.
export function Page({ zoom, children, initialScroll, onScrollFraction }: PageProps) {
  const element = useRef<HTMLElement>(null)
  useLayoutEffect(() => {
    const page = element.current
    if (!page || !initialScroll) return
    page.scrollTop = initialScroll * (page.scrollHeight - page.clientHeight)
    // Restored once, when the page appears.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
  const style = { '--zoom': zoom / 100 } as CSSProperties
  const onScroll = (event: UIEvent<HTMLElement>) => onScrollFraction?.(scrollFraction(event.currentTarget))
  return (
    <article
      ref={element}
      style={style}
      onScroll={onScroll}
      className="@container flex h-[97%] w-[97%] flex-col overflow-y-auto rounded-7 border border-paper-edge bg-paper px-[4.6cqw] py-[3.6cqw] font-serif text-paper-tx shadow-page"
    >
      {children}
    </article>
  )
}
