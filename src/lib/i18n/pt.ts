import type { ErrorMessages } from './errors'

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
  errors: {
    AlreadyExists: 'Já existe um item com o nome {{name}}',
    NoAtomicSupport: 'Este sistema de arquivos não permite renomear {{path}} sem risco de sobrescrever',
    TrashUnavailable: 'A lixeira do sistema não está disponível para {{path}}',
    CopyMismatch: 'A cópia de {{path}} não confere com o original. Nada foi removido.',
    WorkspaceInUse: 'Este workspace já está aberto em outra janela',
    WorkspaceNotFound: 'Workspace não encontrado: {{path}}',
    ForbiddenRoot: '{{path}} não pode ser um workspace: é a raiz do sistema ou a pasta pessoal',
    NestedWorkspace: 'Esta pasta fica dentro do workspace {{path}}',
    PermissionDenied: 'Sem permissão para ler e gravar {{path}}',
    RelativeSymlink: 'O link {{path}} aponta para um caminho relativo e não pode mudar de pasta',
    DocumentNotFound: 'Este documento não está mais no disco',
    InvalidDocument: 'Este arquivo não é um documento válido',
    InvalidPosition: 'A posição de leitura não corresponde a este documento',
    MoveIntoItself: 'Não é possível mover uma pasta para dentro dela mesma',
    InvalidName: {
      Empty: 'O nome não pode ficar vazio',
      ContainsSlash: 'O nome não pode conter /',
      Reserved: 'Este nome é reservado pelo sistema',
      Hidden: 'O nome não pode começar com ponto',
      TooLong: 'O nome é longo demais',
    },
    InvalidPath: 'Caminho inválido dentro do workspace',
    NewerDatabase: 'Este workspace foi aberto por uma versão mais nova do Folio. Atualize o app para abri-lo.',
    Database: 'Não foi possível acessar o banco de dados em {{path}}',
    Io: 'Não foi possível acessar {{path}}',
  } satisfies ErrorMessages,
}

type DeepStrings<T> = { [K in keyof T]: T[K] extends string ? string : DeepStrings<T[K]> }

export type Resources = DeepStrings<typeof pt>
