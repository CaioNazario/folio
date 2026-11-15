import type { Resources } from './pt'

// English resources, checked against the Portuguese shape.
export const en = {
  app: {
    name: 'Folio',
  },
  shell: {
    sidebar: 'Sidebar',
  },
  common: {
    files_one: '{{count, number}} file',
    files_other: '{{count, number}} files',
  },
  language: {
    label: 'Language',
    system: 'Follow system',
    ptBR: 'Português',
    en: 'English',
  },
} satisfies Resources
