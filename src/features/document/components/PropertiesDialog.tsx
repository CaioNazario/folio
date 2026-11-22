import { useTranslation } from 'react-i18next'
import { Dialog } from '@/components/ui/dialog'
import type { ConversionLink } from '@/lib/bindings/ConversionLink'
import type { DocumentProperties } from '@/lib/bindings/DocumentProperties'
import { useFormat } from '@/lib/i18n/useFormat'
import { fileRows, metadataNote, metadataRows } from '../properties/propertyRows'
import { ConversionItem } from './ConversionItem'
import { PropertyNote, PropertyRows, PropertySection } from './PropertyList'

type PropertiesDialogProps = {
  properties: DocumentProperties | null
  onClose: () => void
  onOpenDocument: (link: ConversionLink) => void
}

// Properties of the open document (Design/specs/05-document-view.md, 5.6):
// file, document metadata and conversions.
export function PropertiesDialog({ properties, onClose, onOpenDocument }: PropertiesDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog
      open={properties !== null}
      onOpenChange={(open) => !open && onClose()}
      title={t('document.properties')}
      closeLabel={t('document.propertiesDialog.close')}
    >
      {properties && (
        <div className="flex flex-col gap-[18px]">
          <FileBlock properties={properties} />
          <MetadataBlock properties={properties} />
          <ConversionBlock properties={properties} onOpen={onOpenDocument} />
        </div>
      )}
    </Dialog>
  )
}

function FileBlock({ properties }: { properties: DocumentProperties }) {
  const { t } = useTranslation()
  const format = useFormat()
  return (
    <PropertySection title={t('document.propertiesDialog.file')}>
      <PropertyRows rows={fileRows(properties, t, format)} />
    </PropertySection>
  )
}

function MetadataBlock({ properties: { document, metadata } }: { properties: DocumentProperties }) {
  const { t } = useTranslation()
  const format = useFormat()
  const rows = metadata ? metadataRows(metadata, t, format) : []
  const note = metadataNote(document, metadata, rows)
  const notes = {
    invalid: t('document.stage.invalid', { format: document.format.toUpperCase() }),
    protected: t('document.propertiesDialog.protected'),
    unreadable: t('document.propertiesDialog.unreadable'),
    noMetadata: t('document.propertiesDialog.noMetadata'),
  }
  return (
    <PropertySection title={t('document.propertiesDialog.document')}>
      {note ? <PropertyNote>{notes[note]}</PropertyNote> : <PropertyRows rows={rows} />}
    </PropertySection>
  )
}

type ConversionBlockProps = { properties: DocumentProperties; onOpen: (link: ConversionLink) => void }

function ConversionBlock({ properties: { convertedFrom, derivatives }, onOpen }: ConversionBlockProps) {
  const { t } = useTranslation()
  const groups = [
    { label: t('document.propertiesDialog.convertedFrom'), links: convertedFrom ? [convertedFrom] : [] },
    { label: t('document.propertiesDialog.derivatives'), links: derivatives },
  ].filter((group) => group.links.length > 0)
  return (
    <PropertySection title={t('document.propertiesDialog.conversion')}>
      {groups.length === 0 && <PropertyNote>{t('document.propertiesDialog.noConversion')}</PropertyNote>}
      {groups.map((group) => (
        <div key={group.label} className="flex flex-col gap-1.5">
          <span className="text-[12px] text-tx-8">{group.label}</span>
          <ul aria-label={group.label} className="m-0 flex list-none flex-col gap-2.5 p-0">
            {group.links.map((link) => (
              <ConversionItem key={link.document.id} link={link} onOpen={onOpen} />
            ))}
          </ul>
        </div>
      ))}
    </PropertySection>
  )
}
