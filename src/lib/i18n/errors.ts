import i18n from 'i18next'
import type { BridgeError } from '@/lib/bindings/BridgeError'
import type { InvalidName } from '@/lib/bindings/InvalidName'

// One message per backend error variant (Design/specs/11-i18n.md). The
// dictionaries declare `satisfies ErrorMessages`, so a new Rust variant
// without a message breaks tsc -b.
export type ErrorMessages = Record<Exclude<BridgeError['kind'], 'InvalidName'>, string> & {
  InvalidName: Record<InvalidName, string>
}

export function errorMessage(error: BridgeError): string {
  if (error.kind === 'InvalidName') return i18n.t(`errors.InvalidName.${error.reason}`)
  return i18n.t(`errors.${error.kind}`, { ...error })
}

// Anything rejected by invoke(): a BridgeError, or an unexpected failure.
export function isBridgeError(value: unknown): value is BridgeError {
  return typeof value === 'object' && value !== null && 'kind' in value
}
