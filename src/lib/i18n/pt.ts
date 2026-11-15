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
  language: {
    label: 'Idioma',
    system: 'Seguir o sistema',
    ptBR: 'Português',
    en: 'English',
  },
}

type DeepStrings<T> = { [K in keyof T]: T[K] extends string ? string : DeepStrings<T[K]> }

export type Resources = DeepStrings<typeof pt>
