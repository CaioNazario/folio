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
| **Engine** | Implementação de leitura ou conversão de um formato, atrás de uma interface comum |
| **Extração de texto** | Módulo único que extrai texto de cada formato para o Reader, a conversão e a busca futura |
| **Reindexação** | Reconstrução do banco do workspace a partir do disco |

## Mapa de specs

Nenhuma spec de backend foi escrita ainda. O conteúdo de cada uma está hoje na seção indicada do `AGENTS.md` e sai de lá quando a spec for escrita.

| # | Spec | Cobre | Origem no `AGENTS.md` | Status |
|---|---|---|---|---|
| 01 | workspace | Modelo de vault, abertura, diretório default, regras, operações em projetos e pastas | Modelo de workspace | A escrever |
| 02 | documentos | Reconhecimento, id estável, metadados por formato, documentos derivados | Documentos e documentos derivados | A escrever |
| 03 | persistencia | Banco global e do workspace, tabelas, caminhos relativos, `WorkspaceStore` | Persistência | A escrever |
| 04 | indexacao | Escaneamento, mudanças externas, reconexão por hash, reindexação | Integridade; Questões técnicas em aberto (3, 4) | A escrever |
| 05 | reader | Engines por formato, controles, extração de texto | Reader | A escrever |
| 06 | conversao | Fluxo atômico, colisão de nomes, as 12 conversões e o critério de qualidade | Conversão | A escrever |
| 07 | entrada | Downloads, watcher, downloads em andamento, mover para projeto | Entrada (Downloads) | A escrever |
| 08 | busca-favoritos-recentes | Busca por nome normalizado, filtros, favoritos, recentes | Busca, favoritos e recentes | A escrever |
| 09 | compartilhamento | Mostrar na pasta, arrastar para fora, copiar arquivo | Compartilhamento | A escrever |
| 10 | integridade | Garantias transversais: original intocável, lixeira, disco antes do banco, WAL, temporários | Integridade | A escrever |
| 11 | spikes | Pontos em aberto a resolver antes da feature dependente | Questões técnicas em aberto | A escrever |

## Ordem de implementação

Ordem planejada. Cada spec é escrita antes da feature dela; um spike de `11` se resolve antes da feature que depende dele, e o resultado vai para a spec.

1. Base do `folio-core`: tipos de erro, trait `Clock`, referência a arquivo (caminho ou URI)
2. 03 (persistência) → 01 (workspace): abrir e criar workspace, banco do workspace e banco global
3. 02 (documentos) → 04 (indexação): reconhecimento, id estável, escaneamento e reindexação
4. 08 (busca, favoritos, recentes)
5. 05 (reader): TXT → MD → PDF → DOCX
6. 06 (conversão): das simples (TXT ↔ MD, → TXT) para as difíceis
7. 07 (entrada) → 09 (compartilhamento)

A spec 10 (integridade) vale para todas as etapas desde a primeira.

## Fora de escopo

O mesmo de "Fora do escopo da 0.1.0" no `AGENTS.md`: edição, versionamento, documentos no SQLite, busca no conteúdo, OCR, monitorar mudanças externas em tempo real dentro dos projetos, nuvem e colaboração, IA, builds de Windows e Android.

## Critérios de aceite globais

- [ ] Teste do banco: apagar o `folio.db` não perde nenhum documento, e a reindexação reconstrói o índice a partir do disco.
- [ ] Nenhuma operação sobrescreve em silêncio nem apaga permanentemente: colisão pede confirmação ou gera sufixo, exclusão vai para a lixeira.
- [ ] No banco do workspace, todo caminho é relativo à raiz do workspace.
- [ ] O `folio-core` não depende do Tauri e nenhuma engine depende de binário do sistema.
- [ ] Toda lógica do `folio-core` nasce por TDD (ver `AGENTS.md`).
- [ ] `just check` verde.
