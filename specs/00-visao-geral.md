# 00 · Visão geral (backend)

## O produto

O Folio é um leitor e gerenciador local de documentos. Os documentos são arquivos reais no filesystem; o SQLite guarda só o que o Folio entende sobre eles (identidades, favoritos, recentes, ícones, relações de conversão, cache de metadados, índice de busca). Toda a lógica vive no crate `folio-core`, sem dependência do Tauri; o `src-tauri` só expõe commands que delegam a ele.

> **Princípio central:** o filesystem guarda os documentos. O SQLite guarda o que o Folio entende sobre eles. Nada que o usuário possa perder mora só no banco.

| Campo | Valor |
|---|---|
| Tipo | Aplicação nativa multiplataforma, local-first e offline-first |
| Stack | Rust + Tauri 2 + React |
| Plataformas, em ordem de prioridade | 1. Linux (alvo de referência: Omarchy/Arch) · 2. Windows · 3. Android |
| Plataforma da 0.1.0 | Linux. Windows e Android vêm depois, mas a arquitetura já é escrita para as três |
| Licença | GPL-3.0-or-later |

Estas specs cobrem o comportamento do backend. A aparência está em `Design/specs/` (comece por `Design/specs/00-visao-geral.md`).

## Princípios

Cada camada tem uma única responsabilidade e uma única fonte de verdade.

| Camada | Guarda | Responde |
|---|---|---|
| Filesystem | Os bytes de todos os documentos: originais, convertidos e importados | Qual é o conteúdo deste arquivo? |
| SQLite | Estado e índice: identidades, favoritos, recentes, ícones, relações de conversão, cache de metadados, índice de busca | O que este arquivo significa para o usuário? |
| Git (versão futura) | Histórico dos bytes, dentro de `.folio/` | Como este arquivo era antes? |

1. **Nenhum documento é armazenado no SQLite**, nem como BLOB, nem como cópia.
2. **Teste do banco:** se o usuário apagar o `folio.db`, nenhum documento é perdido. Ele pode perder favoritos, ícones e relações; documentos, nunca.
3. **O filesystem é a verdade da hierarquia.** Projetos e pastas são diretórios reais; o SQLite indexa e cacheia o que está no disco e pode ser reconstruído escaneando o workspace.
4. **Tudo que é derivado pode ser reconstruído:** cache de metadados e índice de busca são recalculáveis a partir dos arquivos.
5. **Pronto para git sem implementá-lo:** como todo documento é um arquivo real, o versionamento futuro só precisa apontar para o workspace. A 0.1.0 não cria abstrações de versionamento por antecipação.
6. **Nenhuma engine depende de binário do sistema** (LibreOffice, Pandoc, poppler...). Biblioteca nativa, se necessária, vai embarcada no app. **Dependências precisam ter licença compatível com a GPL-3.0:** o Folio é GPL-3.0-or-later.

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

## Spikes

Cada spike fica no topo da spec que ele bloqueia e se resolve antes da parte que depende dele; o resultado vai para a spec.

| Spike | Pergunta | Spec |
|---|---|---|
| S1 | Leitor de PDF: hayro ou PDFium (resolvido: hayro) | 07 |
| S2 | pdf.js no WebKitGTK e no WebView do Android (resolvido no Linux; falta o Android) | 07 |
| S3 | Typst embarcado (resolvido no Linux; falta o Android) | 08 |
| S4 | Fidelidade do docx-preview (resolvido no Linux; falta o Android) | 07 |
| S5 | Reconectar ao id um arquivo movido por fora (resolvido) | 05 |
| S6 | Indexação rápida de workspace grande (resolvido no SSD; falta medir em HD externo) | 05 |
| S7 | Workspace em pasta sincronizada (adiado; decisão provisória na spec) | 02 |
| S8 | "Copiar arquivo" no Wayland/Hyprland | 10 |
| S9 | Arrastar para fora no WebKitGTK/Wayland | 10 |

## Fora do escopo da 0.1.0

Não implementar:

- Edição de documentos e editor próprio.
- Versionamento: git, commits, diff, restauração, timeline.
- Armazenar documentos no SQLite.
- Busca no conteúdo dos documentos (full-text) e procurar dentro do documento aberto (`Ctrl+F`).
- Tags.
- OCR.
- Monitorar mudanças externas em tempo real dentro dos projetos (a Entrada é a única pasta observada, spec 09), locking de documentos, FUSE.
- Sincronização em nuvem e colaboração.
- IA integrada.
- Builds de Windows e Android (a arquitetura os suporta, mas não são entregues na 0.1.0).

## Notas para versões futuras

- **Versionamento:** não depender do binário `git` (inexistente no Android); usar biblioteca (gix ou git2) dentro de `.folio/`. DOCX e XLSX são ZIPs e versionam mal no git, então avaliar normalizar o ZIP (modo store, ordem e timestamps fixos) antes de cada commit; rodar compactação (`gc`) periodicamente.
- **Busca no conteúdo:** tabela FTS5 do SQLite (já disponível pelo `rusqlite` com `bundled`) com tokenizer `unicode61 remove_diacritics`, alimentada pelos leitores do modelo de documento (spec 07). O índice é derivado e reconstruível. Português não tem stemmer embutido; plurais são cobertos por busca por prefixo.

## Critérios de aceite globais

- [x] Teste do banco: apagar o `folio.db` não perde nenhum documento, e a reindexação reconstrói o índice a partir do disco.
- [x] Nenhuma operação sobrescreve em silêncio nem apaga permanentemente: colisão gera sufixo ou erro (spec 01), exclusão vai para a lixeira.
- [x] No banco do workspace, todo caminho é relativo à raiz do workspace.
- [x] O `folio-core` não depende do Tauri e nenhuma engine depende de binário do sistema.
- [ ] Toda lógica do `folio-core` nasce por TDD (ver `AGENTS.md`).
- [x] `just check` verde.
