import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useToast } from '@/components/ui/toast-context'
import type { DocumentProperties } from '@/lib/bindings/DocumentProperties'
import { getProperties } from '../api'

// The Properties dialog of the open document: it shows only once the data
// arrives; a failed command opens nothing and says so in a toast
// (Design/specs/05-document-view.md, 5.6).
export function useProperties(documentId: string | null) {
  const { t } = useTranslation()
  const { show } = useToast()
  const [properties, setProperties] = useState<DocumentProperties | null>(null)
  const open = useCallback(() => {
    if (!documentId) return
    getProperties(documentId).then(setProperties, (error: unknown) => {
      console.error(error)
      show(t('document.propertiesDialog.loadFailed'))
    })
  }, [documentId, show, t])
  const close = useCallback(() => setProperties(null), [])
  return { properties, open, close }
}
