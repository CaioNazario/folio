// Theme preference (Design/specs/07-state-interactions.md, "Tema"). Becomes
// the ts-rs binding once the preferences command exists.
export type ThemePreference = 'light' | 'dark' | 'system'

export type Theme = 'light' | 'dark'

export function resolveTheme(preference: ThemePreference, systemDark: boolean): Theme {
  if (preference !== 'system') return preference
  return systemDark ? 'dark' : 'light'
}
