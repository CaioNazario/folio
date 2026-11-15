import { Check, Languages } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { LanguagePreference } from '@/lib/i18n/language'
import { useLanguagePreference } from '../useLanguagePreference'

const OPTIONS = [
  { value: 'system', key: 'preferences.language.system' },
  { value: 'pt-BR', key: 'preferences.language.ptBR' },
  { value: 'en', key: 'preferences.language.en' },
] as const satisfies readonly { value: LanguagePreference; key: string }[]

// Language selector (Design/specs/11-i18n.md): a one-segment group that opens
// the dropdown upwards, aligned to its right edge.
export function LanguageSelector() {
  const { t } = useTranslation()
  const { preference, choose } = useLanguagePreference()
  return (
    <DropdownMenu>
      <div className="flex rounded-14 border border-line-6 bg-surf-2 p-0.5">
        <DropdownMenuTrigger
          title={t('preferences.language.label')}
          aria-label={t('preferences.language.label')}
          className="flex h-5 w-6 cursor-pointer items-center justify-center rounded-10 text-tx-10 outline-none hover:bg-surf-4 hover:text-nav-accent focus-visible:bg-surf-4 data-[state=open]:bg-surf-4 data-[state=open]:text-nav-accent"
        >
          <Languages size={13} />
        </DropdownMenuTrigger>
      </div>
      <DropdownMenuContent side="top" align="end">
        {OPTIONS.map(({ value, key }) => (
          <DropdownMenuItem key={value} onSelect={() => choose(value)}>
            <span className="flex-1">{t(key)}</span>
            {preference === value && <Check size={13} aria-hidden="true" className="text-nav-accent" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
