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

**O número é a ordem de implementação:** uma spec nunca depende de outra de número maior. Os spikes ficam no topo da spec que eles bloqueiam.

| # | Spec | Cobre | Spikes |
|---|---|---|---|
| 01 | [integridade](01-integridade.md) | Garantias transversais: sem sobrescrita, temporários, lixeira, disco antes do banco, banco recuperável, lock do workspace | — |
| 02 | [persistencia](02-persistencia.md) | Banco global e do workspace, abertura, migrações, caminhos em bytes, `WorkspaceStore` | S7 |
| 03 | [workspace](03-workspace.md) | Modelo de vault, abertura, diretório default, workspace aninhado, operações em projetos e pastas, ícones | — |
| 04 | [documentos](04-documentos.md) | Reconhecimento, protegidos por senha, links simbólicos, id estável, tabelas do índice, derivados e avisos de desatualizado | — |
| 05 | [indexacao](05-indexacao.md) | Reconciliação, escaneamento em segundo plano, hash, reconexão, ausentes por 30 dias, reindexação | S5, S6 |
| 06 | [busca-favoritos-recentes](06-busca-favoritos-recentes.md) | Busca por nome normalizado, filtros, favoritos, recentes, continuar de onde parou | — |
| 07 | [reader](07-reader.md) | Exibição por formato, posição, links, imagens do MD, estados do palco, Propriedades, atalhos, CSP, modelo de documento, leitores, metadados | S1, S2, S4 |
| 08 | [conversao](08-conversao.md) | Escritores, as 12 conversões, fluxo atômico, fila com progresso e cancelamento, imagens, critério mínimo de qualidade | S3 |
| 09 | [entrada](09-entrada.md) | Entrada (Downloads, watcher, downloads em andamento, mover para projeto) e importação (seletor, arrastar, copiar) | — |
| 10 | [compartilhamento](10-compartilhamento.md) | Mostrar na pasta, arrastar para fora, copiar arquivo, menu de contexto | S8, S9 |

## Ordem de implementação

A ordem é a dos números. Cada spec é escrita antes da feature dela, e os spikes de uma spec se resolvem antes da parte que eles bloqueiam.

- **Base do `folio-core`** (tipos de erro, trait `Clock`, referência a arquivo por caminho ou URI): nasce junto com a primeira spec que precisar dela.
- **01 (integridade)** é escrita primeiro: as garantias dela valem desde a primeira linha de código e são cobertas pelos testes de cada spec seguinte.
- **07 (reader):** modelo de documento, depois leitores e exibição na ordem TXT → MD → DOCX → PDF.
- **08 (conversão):** escritores na ordem TXT → MD → DOCX → PDF.

## Fora de escopo

O mesmo de "Fora do escopo da 0.1.0" no `AGENTS.md`: edição, versionamento, documentos no SQLite, busca no conteúdo, procurar dentro do documento, tags, OCR, monitorar mudanças externas em tempo real dentro dos projetos, nuvem e colaboração, IA, builds de Windows e Android.

## Critérios de aceite globais

- [ ] Teste do banco: apagar o `folio.db` não perde nenhum documento, e a reindexação reconstrói o índice a partir do disco.
- [ ] Nenhuma operação sobrescreve em silêncio nem apaga permanentemente: colisão gera sufixo ou erro (spec 01), exclusão vai para a lixeira.
- [ ] No banco do workspace, todo caminho é relativo à raiz do workspace.
- [ ] O `folio-core` não depende do Tauri e nenhuma engine depende de binário do sistema.
- [ ] Toda lógica do `folio-core` nasce por TDD (ver `AGENTS.md`).
- [ ] `just check` verde.
