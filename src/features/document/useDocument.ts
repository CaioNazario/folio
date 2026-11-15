import type { DocumentActions, OpenDocument, StageContent } from './types'

const NO_ACTIONS: DocumentActions = {
  onConvert: () => {},
  onRename: () => {},
  onShowInFolder: () => {},
  onCopyFile: () => {},
  onProperties: () => {},
  onTrash: () => {},
  onRetry: () => {},
  onLink: () => {},
}

type DocumentState = { document: OpenDocument | null; content: StageContent; actions: DocumentActions }

// The open document. Nothing is open until the reader commands exist; then
// this reads them through the feature's api.ts.
export function useDocument(): DocumentState {
  return { document: null, content: { status: 'loading' }, actions: NO_ACTIONS }
}
