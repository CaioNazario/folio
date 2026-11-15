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
  sidebar: {
    import: 'Import document',
    search: 'Search documents',
    favorites: {
      title: 'Favorites',
      empty: 'Pin a folder or document to see it here.',
    },
    projects: 'Projects',
    recents: 'Recent',
    pin: {
      add: 'Add to favorites',
      remove: 'Remove from favorites',
    },
    footer: {
      summary: '{{files}} · {{size}}',
    },
  },
  preferences: {
    theme: {
      light: 'Light theme',
      dark: 'Dark theme',
      system: 'Follow system',
    },
    language: {
      label: 'Language',
      system: 'Follow system',
      ptBR: 'Português',
      en: 'English',
    },
  },
} satisfies Resources
