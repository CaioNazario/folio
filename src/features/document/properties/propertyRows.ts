import type { TFunction } from 'i18next'
import type { Document } from '@/lib/bindings/Document'
import type { DocumentProperties } from '@/lib/bindings/DocumentProperties'
import type { Metadata } from '@/lib/bindings/Metadata'
import type { JsonValue } from '@/lib/bindings/serde_json/JsonValue'
import type { useFormat } from '@/lib/i18n/useFormat'

// The rows of the Properties dialog (Design/specs/05-document-view.md, 5.6).
// A row without a value is left out.

export type PropertyRow = { label: string; value: string; mono: boolean }

type Formatter = ReturnType<typeof useFormat>

const KNOWN_EXTRA = ['creator', 'producer', 'version', 'comments', 'revision'] as const
type KnownExtra = (typeof KNOWN_EXTRA)[number]

function row(label: string, value: string | null, mono = false): PropertyRow | null {
  return value ? { label, value, mono } : null
}

function present(rows: (PropertyRow | null)[]): PropertyRow[] {
  return rows.filter((item): item is PropertyRow => item !== null)
}

function fullDate(format: Formatter, ms: number | null): string | null {
  return ms === null ? null : format.fullDateTime(new Date(ms))
}

export function fileRows({ document, location }: DocumentProperties, t: TFunction, format: Formatter): PropertyRow[] {
  return present([
    row(t('document.propertiesDialog.name'), document.name),
    row(t('document.propertiesDialog.location'), location.length > 0 ? location.join(' / ') : t('document.propertiesDialog.root')),
    row(t('document.propertiesDialog.format'), t(`document.formats.${document.format}`)),
    row(t('document.propertiesDialog.size'), format.size(document.size), true),
    row(t('document.propertiesDialog.created'), fullDate(format, document.createdAt), true),
    row(t('document.propertiesDialog.modified'), fullDate(format, document.modifiedAt), true),
  ])
}

export function metadataRows(metadata: Metadata, t: TFunction, format: Formatter): PropertyRow[] {
  const pages = metadata.pages === null ? null : format.number(metadata.pages)
  return present([
    row(t('document.propertiesDialog.title'), metadata.title),
    row(t('document.propertiesDialog.author'), metadata.author),
    row(t('document.propertiesDialog.subject'), metadata.subject),
    row(t('document.propertiesDialog.keywords'), metadata.keywords.join(', ')),
    row(t('document.propertiesDialog.created'), fullDate(format, metadata.createdAt), true),
    row(t('document.propertiesDialog.modified'), fullDate(format, metadata.modifiedAt), true),
    row(t('document.propertiesDialog.pages'), pages, true),
    ...Object.entries(metadata.extra).map(([key, value]) => row(extraLabel(key, t), extraText(value))),
  ])
}

// Known fields get a translated label; the rest (an MD front matter's) keep
// their own name.
function extraLabel(key: string, t: TFunction): string {
  const known = (KNOWN_EXTRA as readonly string[]).includes(key)
  return known ? t(`document.propertiesDialog.extra.${key as KnownExtra}`) : key
}

function extraText(value: JsonValue): string | null {
  if (value === null) return null
  if (Array.isArray(value)) return value.map(extraText).filter(Boolean).join(', ')
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

export type MetadataNote = 'invalid' | 'protected' | 'unreadable' | 'noMetadata'

// What the Document block says instead of rows, if anything: without
// metadata, a valid unprotected document is one whose content is unreadable.
export function metadataNote(document: Document, metadata: Metadata | null, rows: PropertyRow[]): MetadataNote | null {
  if (!document.valid) return 'invalid'
  if (document.protected) return 'protected'
  if (metadata === null) return 'unreadable'
  return rows.length === 0 ? 'noMetadata' : null
}
