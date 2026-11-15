import { useTranslation } from 'react-i18next'
import { LanguageSelector } from '@/features/preferences/components/LanguageSelector'
import { ThemeSelector } from '@/features/preferences/components/ThemeSelector'
import { useFormat } from '@/lib/i18n/useFormat'
import type { WorkspaceSummary } from '../types'

// Footer (Design/specs/03-sidebar.md, 3.4): workspace summary and the
// language and theme selectors.
export function SidebarFooter({ workspace }: { workspace: WorkspaceSummary | null }) {
  const { t } = useTranslation()
  const format = useFormat()
  return (
    <footer className="flex items-center gap-[9px] border-t border-line-1 px-[14px] py-[10px]">
      {workspace && (
        <>
          <span
            aria-hidden="true"
            className="flex size-[22px] shrink-0 items-center justify-center rounded-10 bg-line-5 text-[11px] font-semibold text-avatar-tx"
          >
            {workspace.userName.charAt(0).toUpperCase()}
          </span>
          <div className="min-w-0 leading-[1.25]">
            <div className="truncate text-[12px] text-tx-1">{workspace.path}</div>
            <div className="truncate font-mono text-[10px] text-tx-1">
              {t('sidebar.footer.summary', {
                files: t('common.files', { count: workspace.fileCount }),
                size: format.size(workspace.totalBytes),
              })}
            </div>
          </div>
        </>
      )}
      <div className="ml-auto flex items-center gap-[9px]">
        <LanguageSelector />
        <ThemeSelector />
      </div>
    </footer>
  )
}
