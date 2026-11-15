# 03 · Workspace

O workspace é um diretório no modelo de vault, marcado por uma pasta oculta `.folio/`. Esta spec cobre a estrutura de diretórios (workspace, projetos, pastas), a abertura e as operações sobre diretórios. Quais arquivos são documentos e como eles aparecem é da spec 04; o escaneamento e as mudanças externas, da spec 05.

## Modelo

```plain
~/Documents/Folio/              ← WORKSPACE
├── .folio/                     ← reservado ao Folio
│   ├── folio.db                   banco do workspace (spec 02)
│   ├── tmp/                       temporários (spec 01)
│   └── lock                       lock da janela (spec 01)
├── Faculdade/                  ← PROJETO
│   ├── TCC/                    ← pasta
│   │   ├── Trabalho.pdf
│   │   └── Trabalho.docx
│   └── Disciplinas/            ← pasta
├── Trabalho/                   ← PROJETO
│   └── Contrato.docx
└── leia-me.pdf                 ← documento sem projeto
```

| Termo | Definição |
|---|---|
| Workspace | O diretório raiz, com `.folio/` |
| Projeto | Diretório de primeiro nível do workspace. Não há cadastro: se o diretório existe na raiz, é um projeto, mesmo vazio |
| Pasta | Qualquer diretório dentro de um projeto, mesmo vazio |
| Sem projeto | Os documentos soltos na raiz do workspace. O Folio nunca esconde um documento |

O banco guarda só o extra (ícone aqui, favorito na spec 06). A hierarquia vem sempre do disco.

## O que não aparece

| Item | Por quê |
|---|---|
| `.folio/` | Reservado ao Folio |
| Diretórios ocultos (nome começando com `.`: `.git`, `.obsidian`...) | Convenção do sistema; o gerenciador de arquivos também os esconde por padrão |
| Link simbólico para diretório | Não é seguido: pode sair do workspace ou formar ciclo. Fica intacto no disco |
| Outro workspace dentro deste (um subdiretório com `.folio/`) | Ver "Workspace dentro de workspace" |

Os mesmos critérios para arquivos estão na spec 04.

## Abertura

### Primeira execução

Sem nenhum workspace no banco global, o app abre o diálogo de escolha da raiz:

- **Diretório default pré-selecionado:** `Folio/` dentro da pasta de documentos do sistema, que segue o idioma do sistema. No Linux, `XDG_DOCUMENTS_DIR` (lido de `~/.config/user-dirs.dirs`): `~/Documentos/Folio` num sistema em português, `~/Documents/Folio` num em inglês. Sem a variável, o fallback é `~/Documents/Folio`. Windows e Android são especificados com as respectivas versões.
- O diálogo segue o idioma do sistema, como o resto do app (`Design/specs/11-i18n.md`).
- O usuário pode escolher outro diretório pelo seletor do sistema.
- O default é criado só quando o usuário confirma.
- A escolha acontece no app, não no instalador, para funcionar igual com AppImage, AUR, .exe e Android.

### Ao iniciar o app

Abre o último workspace aberto (`opened_at` mais novo no banco global, spec 02).

**Workspace que não existe mais** (apagado, renomeado, disco desmontado): o app mostra "Workspace não encontrado: `<caminho>`", com "Tentar de novo" e "Escolher outro". O workspace **continua na lista**: um disco desmontado volta.

### Passos da abertura

1. **Valida o diretório:** existe, é diretório, dá para ler e escrever.
2. **Recusa raiz perigosa:** a raiz do sistema (`/`) e a própria home não podem ser workspace ("Escolha uma pasta dentro da sua home"). O escaneamento delas indexaria o sistema inteiro.
3. **Recusa workspace aninhado:** procura `.folio/` nos ancestrais (ver abaixo).
4. **Pega o lock** de `.folio/lock` (spec 01). Se outra instância já estiver com ele, não abre.
5. **Cria `.folio/` e `.folio/tmp/`** se não existirem.
6. **Abre o banco** (spec 02) e **esvazia `.folio/tmp/`** (spec 01).
7. **Registra no banco global:** insere o workspace ou atualiza `opened_at`.
8. **Escaneia** (spec 05).

### Workspace dentro de workspace

É proibido, nos dois sentidos:

- **Escolhido dentro de um workspace** (um ancestral tem `.folio/`): bloqueia com "Esta pasta está dentro do workspace `<ancestral>`", e oferece abrir o ancestral.
- **Escolhido contendo um workspace** (um descendente tem `.folio/`): o escaneamento encontra o `.folio/` do descendente e **não entra** naquele diretório. Ele aparece na árvore como pasta bloqueada, com "Esta pasta é outro workspace" e a ação "Abrir este workspace". Nada é escondido em silêncio, e o `.folio/` do outro workspace nunca é tocado.

O segundo caso não bloqueia a abertura: o descendente pode surgir depois (um workspace copiado para dentro de um projeto), e bloquear o workspace inteiro por isso seria pior.

### Trocar de workspace

- Um workspace por janela, e a 0.1.0 tem uma janela só. Uma segunda execução do app foca a janela que já está aberta (`tauri-plugin-single-instance`).
- "Abrir/trocar workspace" lista os recentes do banco global e oferece escolher outro diretório. Trocar fecha o banco e solta o lock do workspace atual antes de abrir o novo.
- "Esquecer" tira um workspace da lista de recentes. Não toca no disco nem em `.folio/`.

## Operações em projetos e pastas

Todas seguem a spec 01: operação real no disco, sem sobrescrita, disco antes do banco.

| Operação | Onde | Observações |
|---|---|---|
| Criar projeto | Raiz do workspace | Diretório novo; nome já existente é erro |
| Criar pasta | Dentro de um projeto ou pasta | Idem |
| Renomear | Projeto ou pasta | Rename sem sobrescrita; o índice atualiza o caminho de tudo que está dentro, numa transação |
| Mover | Pasta para outro projeto ou pasta; pasta para a raiz (vira projeto); projeto para dentro de outro (vira pasta) | Mover para dentro de si mesmo ou de um descendente é erro. Entre filesystems diferentes não é suportado na 0.1.0 (erro) |
| Excluir | Projeto ou pasta | Vai para a lixeira inteiro (spec 01). Para o índice, é o mesmo que o diretório sumir do disco (spec 05) |
| Associar ícone | Projeto ou pasta | Ver "Ícones" |

**Validação de nome** (criar, renomear), com a mensagem que a interface mostra:

- Vazio ou só espaços: "Digite um nome". Espaços nas pontas são removidos.
- Contém `/` ou o caractere nulo: "O nome não pode conter /".
- `.` ou `..`: "Nome inválido".
- Começa com `.`: "Nomes começando com ponto ficam ocultos". O diretório sumiria da interface.
- Mais de 255 bytes em UTF-8: "Nome longo demais".
- Já existe no destino: "Já existe um item com esse nome" (spec 01, sem sufixo automático).

## Ícones

- Um projeto ou pasta pode ter um ícone, escolhido numa lista fechada de ícones Lucide definida pelo frontend. O core guarda o nome do ícone como texto e não o valida; nome desconhecido é exibido como a pasta padrão.
- Tabela `icons`: `path` (`CaminhoRelativo` do diretório, chave primária) e `icon` (texto). Migração desta spec.

**Diretórios não têm id estável.** Só documentos têm (spec 04). Tudo que o banco liga a um diretório (ícone, favorito na spec 06) usa o caminho:

- Renomear ou mover pelo Folio atualiza o caminho dessas ligações na mesma transação do índice.
- Renomear ou mover por fora do Folio perde a ligação: o diretório volta ao ícone padrão. O diretório e os documentos dentro dele ficam intactos.

## Erros

| Erro | Quando |
|---|---|
| `WorkspaceNaoEncontrado` | O diretório não existe ou não é diretório |
| `SemPermissao` | Não dá para ler ou escrever no diretório |
| `RaizProibida` | `/` ou a home |
| `WorkspaceAninhado` | Um ancestral tem `.folio/` (o erro traz o caminho do ancestral) |
| `NomeInvalido` | Falha na validação de nome (o erro traz o motivo) |
| `MoverParaDentroDeSi` | Destino é o próprio diretório ou um descendente |

Mais os da spec 01 (`JaExiste`, `WorkspaceEmUso`...).

## Critérios de aceite

- [ ] Na primeira execução, o diálogo vem com `Folio/` dentro do `XDG_DOCUMENTS_DIR` pré-selecionado (`~/Documentos/Folio` num sistema em português), e `~/Documents/Folio` sem a variável.
- [ ] Abrir um diretório sem `.folio/` cria `.folio/`, `.folio/tmp/` e o banco; abrir de novo reaproveita os três.
- [ ] Todo diretório de primeiro nível é um projeto, inclusive vazio; `.folio/`, ocultos e links simbólicos para diretório não aparecem.
- [ ] Diretório dentro de um workspace não abre e o erro aponta o ancestral.
- [ ] Workspace contido em outro aparece como pasta bloqueada e seu `.folio/` não é lido nem alterado.
- [ ] `/` e a home são recusados como workspace.
- [ ] Workspace que sumiu mostra o aviso e continua na lista de recentes.
- [ ] Criar, renomear e mover nunca sobrescrevem; nome inválido é recusado com o motivo.
- [ ] Mover um diretório para dentro de si mesmo é recusado.
- [ ] Renomear ou mover pelo Folio preserva o ícone e atualiza o caminho de todos os documentos dentro, numa transação.
- [ ] Excluir manda o diretório inteiro para a lixeira.
- [ ] Trocar de workspace solta o lock do anterior.
