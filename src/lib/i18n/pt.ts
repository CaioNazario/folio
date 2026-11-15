// Portuguese resources: the reference shape for every other language
// (Design/specs/11-i18n.md).
export const pt = {
  app: {
    name: 'Folio',
  },
  shell: {
    sidebar: 'Barra lateral',
  },
  common: {
    files_one: '{{count, number}} arquivo',
    files_other: '{{count, number}} arquivos',
  },
  sidebar: {
    import: 'Importar documento',
    search: 'Pesquisar documentos',
    favorites: {
      title: 'Favoritos',
      empty: 'Fixe uma pasta ou documento com o pin para vê-lo aqui.',
    },
    projects: 'Projetos',
    recents: 'Recentes',
    pin: {
      add: 'Adicionar aos favoritos',
      remove: 'Remover dos favoritos',
    },
    footer: {
      summary: '{{files}} · {{size}}',
    },
  },
  preferences: {
    theme: {
      light: 'Tema claro',
      dark: 'Tema escuro',
      system: 'Seguir o sistema',
    },
    language: {
      label: 'Idioma',
      system: 'Seguir o sistema',
      ptBR: 'Português',
      en: 'English',
    },
  },
}

type DeepStrings<T> = { [K in keyof T]: T[K] extends string ? string : DeepStrings<T[K]> }

export type Resources = DeepStrings<typeof pt>
