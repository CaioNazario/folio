import { useSyncExternalStore } from 'react'

const QUERY = '(prefers-color-scheme: dark)'

function media(): MediaQueryList | null {
  return typeof window.matchMedia === 'function' ? window.matchMedia(QUERY) : null
}

export function systemDark(): boolean {
  return media()?.matches ?? true
}

function subscribe(onChange: () => void) {
  const list = media()
  list?.addEventListener('change', onChange)
  return () => list?.removeEventListener('change', onChange)
}

// Follows the system color scheme live; dark when it cannot be read.
export function useSystemDark(): boolean {
  return useSyncExternalStore(subscribe, systemDark)
}
