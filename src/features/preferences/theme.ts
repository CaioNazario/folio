import type { Theme } from '@/lib/bindings/Theme'

export type ResolvedTheme = 'light' | 'dark'

// Design/specs/07-state-interactions.md, "Tema".
export function resolveTheme(preference: Theme, systemDark: boolean): ResolvedTheme {
  if (preference !== 'system') return preference
  return systemDark ? 'dark' : 'light'
}

export function applyTheme(theme: ResolvedTheme) {
  document.documentElement.dataset.theme = theme
}
