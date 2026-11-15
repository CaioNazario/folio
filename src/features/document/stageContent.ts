import { isBridgeError } from '@/lib/i18n/errors'
import type { StageContent } from './types'

// Which stage state a failed load shows (specs/07-reader.md, "Estados do palco").
export function failedContent(error: unknown): StageContent {
  if (!isBridgeError(error)) return { status: 'failed' }
  if (error.kind === 'DocumentNotFound') return { status: 'notFound' }
  if (error.kind === 'InvalidDocument') return { status: 'invalid' }
  return { status: 'failed' }
}
