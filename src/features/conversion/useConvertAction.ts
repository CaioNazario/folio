import { useState } from 'react'
import type { OpenDocument } from '@/features/document/types'
import type { Format } from '@/lib/bindings/Format'
import type { ConvertRequest } from './useConversions'

// The Converter menu's action for the open document: a protected one goes
// through the warning dialog first (specs/08-conversao.md, "Fluxo").
export function useConvertAction(document: OpenDocument | null, convert: (request: ConvertRequest) => void) {
  const [pending, setPending] = useState<Format | null>(null)
  const request = (target: Format, password: string | null) =>
    document && convert({ id: document.id, name: document.name, target, password })
  const onConvert = (target: Format) => (document?.protected ? setPending(target) : request(target, null))
  const confirm = (password: string) => {
    if (pending) request(pending, password)
    setPending(null)
  }
  return { onConvert, dialogOpen: pending !== null, confirm, close: () => setPending(null) }
}
