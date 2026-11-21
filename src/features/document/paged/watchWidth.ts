// Calls `refit` when the stage width changes (window, fullscreen), so a
// paged display keeps its zoom relative to the stage width.
export function watchWidth(container: HTMLElement, refit: () => void) {
  let width = container.clientWidth
  const observer = new ResizeObserver(() => {
    if (container.clientWidth === width) return
    width = container.clientWidth
    refit()
  })
  observer.observe(container)
  return () => observer.disconnect()
}
