# 00 · Visão geral (backend)

## O produto

O Folio é um leitor e gerenciador local de documentos. Os documentos são arquivos reais no filesystem; o SQLite guarda só o que o Folio entende sobre eles (identidades, favoritos, recentes, ícones, relações de conversão, cache de metadados, índice de busca). Toda a lógica vive no crate `folio-core`, sem dependência do Tauri; o `src-tauri` só expõe commands que delegam a ele. Plataforma da 0.1.0: Linux.

Estas specs cobrem o comportamento do backend. A aparência está em `Design/specs/` (comece por `Design/specs/00-visao-geral.md`).

## Glossário

| Termo | Significado |
|---|---|
| **Workspace** | Diretório marcado por uma pasta oculta `.folio/`; um por janela |
| **Projeto** | Diretório de primeiro nível do workspace; não há cadastro |
| **Pasta** | Qualquer diretório dentro de um projeto |
| **Documento** | Arquivo do workspace reconhecido pela extensão e confirmado por magic bytes |
| **Id estável** | Identidade do documento gerada pelo Folio; sobrevive a renomear e mover |
| **Documento derivado** | Resultado de uma conversão: arquivo comum, salvo ao lado do original |
| **`converted_to`** | Relação no índice entre origem e derivado (formatos e data) |
| **Entrada** | Visão dos documentos da pasta Downloads, sem transformá-la em workspace |
| **Banco global** | SQLite do app (`~/.local/share/folio/`): workspaces, preferências, caminho da Entrada |
| **Banco do workspace** | `<workspace>/.folio/folio.db`: índice e estado daquele workspace |
| **Engine** | Biblioteca que exibe, lê ou gera um formato (pdf.js, comrak, Typst...) (specs 07, 08) |
| **Modelo de documento** | Representação neutra de um documento: títulos, parágrafos, listas, ênfase, tabelas, imagens (spec 07) |
| **Leitor** | Transforma um arquivo de um formato no modelo de documento; um por formato (spec 07) |
| **Escritor** | Transforma o modelo de documento num arquivo de um formato; um por formato (spec 08) |
| **Reindexação** | Reconstrução do banco do workspace a partir do disco |

## Mapa de specs

**O número é a ordem de implementação:** uma spec nunca depende de outra de número maior. Os spikes ficam no topo da spec que eles bloqueiam. As specs ainda não escritas têm o conteúdo na seção indicada do `AGENTS.md`, que sai de lá quando a spec for escrita.

| # | Spec | Cobre | Spikes | Origem no `AGENTS.md` | Status |
|---|---|---|---|---|---|
| 01 | [integridade](01-integridade.md) | Garantias transversais: sem sobrescrita, temporários, lixeira, disco antes do banco, banco recuperável, lock do workspace | — | — | Escrita |
| 02 | [persistencia](02-persistencia.md) | Banco global e do workspace, abertura, migrações, caminhos em bytes, `WorkspaceStore` | S7 | — | Escrita |
| 03 | [workspace](03-workspace.md) | Modelo de vault, abertura, diretório default, workspace aninhado, operações em projetos e pastas, ícones | — | — | Escrita |
| 04 | documentos | Reconhecimento, id estável, metadados por formato, documentos derivados | — | Documentos e documentos derivados | A escrever |
| 05 | indexacao | Escaneamento, mudanças externas, reconexão por hash, reindexação | S5, S6 | Integridade (mudanças externas) | A escrever |
| 06 | busca-favoritos-recentes | Busca por nome normalizado, filtros, favoritos, recentes | — | Busca, favoritos e recentes | A escrever |
| 07 | [reader](07-reader.md) | Exibição por formato, modelo de documento, leitores | S1, S2, S4 | — | Escrita |
| 08 | [conversao](08-conversao.md) | Escritores, as 12 conversões, fluxo atômico, critério mínimo de qualidade | S3 | — | Escrita |
| 09 | entrada | Downloads, watcher, downloads em andamento, mover para projeto | — | Entrada (Downloads) | A escrever |
| 10 | compartilhamento | Mostrar na pasta, arrastar para fora, copiar arquivo | S8 | Compartilhamento | A escrever |

## Ordem de implementação

A ordem é a dos números. Cada spec é escrita antes da feature dela, e os spikes de uma spec se resolvem antes da parte que eles bloqueiam.

- **Base do `folio-core`** (tipos de erro, trait `Clock`, referência a arquivo por caminho ou URI): nasce junto com a primeira spec que precisar dela.
- **01 (integridade)** é escrita primeiro: as garantias dela valem desde a primeira linha de código e são cobertas pelos testes de cada spec seguinte.
- **07 (reader):** modelo de documento, depois leitores e exibição na ordem TXT → MD → DOCX → PDF.
- **08 (conversão):** escritores na ordem TXT → MD → DOCX → PDF.

## Spikes das specs ainda não escritas

Cada spike fica no topo da spec que ele bloqueia. Os destas specs ainda não escritas ficam aqui e vão para a spec quando ela for escrita.

### S5 · Reconectar ao id um arquivo movido por fora

**Bloqueia:** spec 05 (indexação).

**Pergunta:** como reconhecer que um arquivo renomeado ou movido fora do Folio é o mesmo documento: hash do conteúdo, inode, ou os dois?

**Resultado:** pendente.

### S6 · Indexação rápida de workspace grande

**Bloqueia:** spec 05 (indexação).

**Pergunta:** como indexar um workspace com milhares de arquivos na primeira abertura sem travar a interface (escaneamento incremental, hash sob demanda, paralelismo)?

**Resultado:** pendente.

### S8 · "Copiar arquivo" no Wayland/Hyprland

**Bloqueia:** spec 10 (compartilhamento).

**Pergunta:** colocar o arquivo no clipboard como `text/uri-list` funciona de forma confiável no Wayland/Hyprland, a ponto de colar no gerenciador de arquivos e em apps como o WhatsApp Web?

**Resultado:** pendente. Se não funcionar, "Copiar arquivo" sai da 0.1.0 no Linux.

## Fora de escopo

O mesmo de "Fora do escopo da 0.1.0" no `AGENTS.md`: edição, versionamento, documentos no SQLite, busca no conteúdo, OCR, monitorar mudanças externas em tempo real dentro dos projetos, nuvem e colaboração, IA, builds de Windows e Android.

## Critérios de aceite globais

- [ ] Teste do banco: apagar o `folio.db` não perde nenhum documento, e a reindexação reconstrói o índice a partir do disco.
- [ ] Nenhuma operação sobrescreve em silêncio nem apaga permanentemente: colisão pede confirmação ou gera sufixo, exclusão vai para a lixeira.
- [ ] No banco do workspace, todo caminho é relativo à raiz do workspace.
- [ ] O `folio-core` não depende do Tauri e nenhuma engine depende de binário do sistema.
- [ ] Toda lógica do `folio-core` nasce por TDD (ver `AGENTS.md`).
- [ ] `just check` verde.
