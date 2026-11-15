import { useTranslation } from 'react-i18next'

// Folio wordmark (Design/specs/09-assets.md): two offset sheets and the name
// in Source Serif 4 600, colored by currentColor.
export function Logo() {
  const { t } = useTranslation()
  return (
    <svg viewBox="0 0 164 58" height={22} role="img" aria-label={t('app.name')} xmlns="http://www.w3.org/2000/svg">
      <path d="M3 4H22V52H3A3 3 0 0 1 0 49V7A3 3 0 0 1 3 4Z" fill="currentColor" />
      <path d="M26 0H45A3 3 0 0 1 48 3V45A3 3 0 0 1 45 48H26Z" fill="currentColor" />
      <text
        x="62"
        y="46"
        fontFamily="'Source Serif 4', Georgia, serif"
        fontSize="46"
        fontWeight="600"
        letterSpacing="-0.9"
        fill="currentColor"
        textLength="96"
        lengthAdjust="spacingAndGlyphs"
        aria-hidden="true"
      >
        {t('app.name')}
      </text>
    </svg>
  )
}
