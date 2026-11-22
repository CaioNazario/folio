import { TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { ConversionLink } from '@/lib/bindings/ConversionLink'
import { useFormat } from '@/lib/i18n/useFormat'

type ConversionItemProps = { link: ConversionLink; onOpen: (link: ConversionLink) => void }

// One side of a conversion in Properties: the document, how and when it was
// converted, and its stale warnings (Design/specs/05-document-view.md, 5.6).
export function ConversionItem({ link, onOpen }: ConversionItemProps) {
  const { t } = useTranslation()
  const format = useFormat()
  const warnings = [
    link.sourceChanged && t('document.propertiesDialog.sourceChanged'),
    link.derivativeChanged && t('document.propertiesDialog.derivativeChanged'),
  ].filter((warning) => typeof warning === 'string')
  const detail = {
    from: link.sourceFormat.toUpperCase(),
    to: link.targetFormat.toUpperCase(),
    date: format.fullDateTime(new Date(link.convertedAt)),
  }
  return (
    <li className="flex flex-col gap-0.5">
      <button type="button" className="cursor-pointer self-start text-left text-[13px] break-words text-link hover:text-link-hover" onClick={() => onOpen(link)}>
        {link.document.name}
      </button>
      <span className="text-[12px] text-tx-8">{t('document.propertiesDialog.conversionDetail', detail)}</span>
      {warnings.map((warning) => (
        <span key={warning} className="flex items-center gap-1.5 text-[12px] text-tx-7">
          <TriangleAlert aria-hidden="true" className="size-3 shrink-0" />
          {warning}
        </span>
      ))}
    </li>
  )
}
