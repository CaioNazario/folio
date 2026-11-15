import { File, Folder } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SectionLabel } from '@/components/ui/section-label'
import type { FavoriteItem } from '../types'
import { NavRow, RowName } from './NavRow'
import { PinButton } from './PinButton'

type FavoritesSectionProps = {
  favorites: FavoriteItem[]
  onOpenFile: (id: string) => void
  onToggleFolder: (id: string) => void
  onTogglePin: (id: string, kind: FavoriteItem['kind']) => void
}

// Favorites (Design/specs/03-sidebar.md, 3.3), in insertion order.
export function FavoritesSection({ favorites, onOpenFile, onToggleFolder, onTogglePin }: FavoritesSectionProps) {
  const { t } = useTranslation()
  return (
    <section aria-labelledby="sidebar-favorites">
      <SectionLabel id="sidebar-favorites" tone="sidebar" className="px-2 pt-[10px] pb-1.5">
        {t('sidebar.favorites.title')}
      </SectionLabel>
      {favorites.length === 0 && (
        <p className="m-0 px-2 pt-0.5 pb-1 text-[12px] leading-[1.45] text-tx-10">{t('sidebar.favorites.empty')}</p>
      )}
      {favorites.map((item) => {
        const Icon = item.kind === 'folder' ? Folder : File
        const open = item.kind === 'folder' ? onToggleFolder : onOpenFile
        return (
          <NavRow
            key={item.id}
            label={item.name}
            onClick={() => open(item.id)}
            trailing={<PinButton pinned onToggle={() => onTogglePin(item.id, item.kind)} />}
          >
            <Icon size={13} className="shrink-0 text-tx-5" />
            <RowName>{item.name}</RowName>
          </NavRow>
        )
      })}
    </section>
  )
}
