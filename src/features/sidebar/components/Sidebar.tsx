import { useTranslation } from 'react-i18next'
import { Logo } from './Logo'

// Sidebar column (Design/specs/02-layout.md and 03-sidebar.md).
export function Sidebar() {
  const { t } = useTranslation()
  return (
    <aside
      aria-label={t('shell.sidebar')}
      className="flex min-w-[196px] flex-[0_0_min(272px,24vw)] flex-col border-r border-surf-4 bg-bg"
    >
      <header className="flex h-[46px] shrink-0 items-center gap-[9px] border-b border-line-1 px-[14px] text-tx-1">
        <Logo />
      </header>
    </aside>
  )
}
