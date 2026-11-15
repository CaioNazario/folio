import { Monitor, Moon, Sun } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import type { Theme } from '@/lib/bindings/Theme'
import { usePreferences } from '../preferences-context'

const OPTIONS = [
  { value: 'light', Icon: Sun, key: 'preferences.theme.light' },
  { value: 'dark', Icon: Moon, key: 'preferences.theme.dark' },
  { value: 'system', Icon: Monitor, key: 'preferences.theme.system' },
] as const satisfies readonly { value: Theme; Icon: unknown; key: string }[]

// Three-segment theme selector (Design/specs/03-sidebar.md, 3.4).
export function ThemeSelector() {
  const { t } = useTranslation()
  const { theme: preference, setTheme: setPreference } = usePreferences()
  return (
    <div role="radiogroup" className="flex gap-0.5 rounded-14 border border-line-6 bg-surf-2 p-0.5">
      {OPTIONS.map(({ value, Icon, key }) => (
        <button
          key={value}
          type="button"
          role="radio"
          aria-checked={preference === value}
          title={t(key)}
          aria-label={t(key)}
          onClick={() => setPreference(value)}
          className={cn(
            'flex h-5 w-6 cursor-pointer items-center justify-center rounded-10 transition-colors duration-150 ease-in-out outline-none focus-visible:bg-surf-4',
            preference === value ? 'bg-surf-4 text-nav-accent' : 'text-tx-10',
          )}
        >
          <Icon size={13} />
        </button>
      ))}
    </div>
  )
}
