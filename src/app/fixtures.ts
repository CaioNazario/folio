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

const ALL_DOCUMENTS = [...ROOT.documents, ...FACULDADE_CONTENTS.documents, doc('d-pdf', 'Artigo.pdf', 'pdf'), doc('d-docx', 'Trabalho.docx', 'docx')]

export const MARKDOWN =
  '<h1>Referências</h1><p><a href="https://example.org">site</a> <a href="Leia-me.txt">leia</a> <a href="x.xlsx">planilha</a> <a href="#notas">notas</a> <a href="#nada">nada</a></p>' +
  '<h2 id="folio-h-notas">Notas</h2>' +
  '<span data-folio-image="remote">diagrama <a href="https://img.example/d.png">https://img.example/d.png</a></span>'

function openedView(id: string) {
  const document = ALL_DOCUMENTS.find((item) => item.id === id)
  if (!document) throw { kind: 'DocumentNotFound', id }
  const folders = FACULDADE_CONTENTS.documents.includes(document) ? ['Faculdade'] : []
  return { document, folders, derived: false, conversionTargets: ['pdf'], position: null }
}

const LINKS: Record<string, object> = {
  'https://example.org': { kind: 'External', url: 'https://example.org' },
  'Leia-me.txt': { kind: 'Document', id: 'd-readme' },
  'x.xlsx': { kind: 'Unsupported' },
  '#notas': { kind: 'Anchor', fragment: 'notas' },
  '#nada': { kind: 'Anchor', fragment: 'nada' },
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
    open_document: (args: unknown) => openedView((args as { id: string }).id),
    get_text_content: () => ({ text: 'Conteúdo do trabalho', truncated: false }),
    get_markdown_content: () => ({ html: MARKDOWN, truncated: false }),
    get_document_bytes: () => new ArrayBuffer(8),
    toggle_favorite_document: () => true,
    follow_link: (args: unknown) => LINKS[(args as { href: string }).href] ?? { kind: 'Ignored' },
    ...overrides,
  }
}
