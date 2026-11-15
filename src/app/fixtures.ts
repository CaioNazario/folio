import type { DirContents } from '@/lib/bindings/DirContents'
import type { Document } from '@/lib/bindings/Document'

// Workspace data for the shell tests, shaped like the ts-rs bindings.

export const DAY = 86_400_000
const FACULDADE = [70]

function doc(id: string, name: string, format: Document['format']): Document {
  return { id, path: [1], name, format, valid: true, protected: false, size: 42 * 1024, modifiedAt: 0, createdAt: null }
}

export const ROOT: DirContents = {
  folders: [{ path: FACULDADE, name: 'Faculdade', kind: 'Folder', icon: null, documentCount: 2 }],
  documents: [doc('d-readme', 'Leia-me.txt', 'txt')],
}

export const FACULDADE_CONTENTS: DirContents = {
  folders: [],
  documents: [doc('d-v3', 'Trabalho v3.txt', 'txt'), doc('d-ref', 'Referências.md', 'md')],
}

export function workspaceCommands(overrides: Record<string, (args: unknown) => unknown> = {}) {
  return {
    open_last_workspace: () => true,
    list_dir: (args: unknown) => ((args as { parent: number[] | null }).parent ? FACULDADE_CONTENTS : ROOT),
    get_favorites: () => [{ id: 'fav-1', name: 'Faculdade', target: { kind: 'Project', path: FACULDADE } }],
    get_recent_documents: () => [
      { id: 'd-v3', name: 'Trabalho v3.txt', project: 'Faculdade', size: 42 * 1024, openedAt: Date.now() - 2 * DAY },
    ],
    get_workspace_summary: () => ({ location: '~/Documentos/Folio', totals: { documents: 1284, size: 3.2 * 1024 ** 3 }, userName: 'Caio' }),
    get_text_content: () => ({ text: 'Conteúdo do trabalho', truncated: false }),
    toggle_favorite_document: () => true,
    ...overrides,
  }
}
