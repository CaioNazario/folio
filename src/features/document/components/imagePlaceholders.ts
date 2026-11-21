import type { TFunction } from 'i18next'

// Images a document cannot show become <span data-folio-image="reason">
// with the alt text (the core does it for MD, the DOCX view for images the
// webview cannot draw); the label is the front's, in the active language.
export const IMAGE_PLACEHOLDER =
  '[&_[data-folio-image]]:my-[0.4em] [&_[data-folio-image]]:block [&_[data-folio-image]]:rounded-7 [&_[data-folio-image]]:border [&_[data-folio-image]]:border-dashed [&_[data-folio-image]]:border-paper-rule [&_[data-folio-image]]:px-[0.8em] [&_[data-folio-image]]:py-[0.5em] [&_[data-folio-image]]:font-sans [&_[data-folio-image]]:text-[0.8em] [&_[data-folio-image]]:text-paper-mut2 [&_[data-folio-image]]:before:mr-[0.5em] [&_[data-folio-image]]:before:font-semibold [&_[data-folio-image]]:before:content-[attr(data-label)]'

const IMAGE_LABELS = {
  remote: 'document.image.remote',
  missing: 'document.image.missing',
  outside: 'document.image.outside',
  unsupported: 'document.image.unsupported',
  'too-large': 'document.image.tooLarge',
} as const

function imageLabel(reason: string | undefined) {
  return IMAGE_LABELS[(reason ?? 'missing') as keyof typeof IMAGE_LABELS] ?? IMAGE_LABELS.missing
}

export function labelImagePlaceholders(root: HTMLElement, t: TFunction) {
  root.querySelectorAll<HTMLElement>('[data-folio-image]').forEach((span) => {
    span.dataset.label = t(imageLabel(span.dataset.folioImage))
  })
}
