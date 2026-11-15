# 05 · Reader

O Reader oferece a mesma experiência de leitura para os formatos da 0.1.0: **PDF, TXT, MD e DOCX**. XLSX, PPTX e imagens ficam para depois.

Esta spec cobre duas partes: a **exibição** (o que o usuário vê no palco) e os **leitores**, que transformam um arquivo no modelo de documento. Os leitores servem ao Reader, à conversão (spec 06), aos metadados (spec 02) e, no futuro, à busca no conteúdo.

## Arquitetura

```text
                  ┌─ bytes / texto / HTML ─▶ exibição no webview (React)
arquivo ─▶ core ──┤
                  └─ leitor ─▶ modelo de documento ─▶ conversão, metadados, busca
```

- **A exibição acontece no webview.** Cada formato tem um componente de exibição em React, atrás de uma interface comum (página atual, total, zoom, navegação). O core não renderiza páginas: ele entrega ao frontend o conteúdo de que cada exibição precisa.
- **Os leitores rodam no `folio-core`**, em Rust, e produzem o modelo de documento.
- O frontend nunca lê arquivos diretamente: pede o conteúdo por command, pelo id estável do documento (spec 02).

## Exibição

| Formato | Engine | O core entrega | Páginas |
|---|---|---|---|
| PDF | `pdfjs-dist` (pdf.js) | Bytes do arquivo | Exatas, do próprio PDF |
| DOCX | `docx-preview` | Bytes do arquivo | Aproximadas: só as quebras explícitas do documento (o `docx-preview` não diagrama páginas) |
| MD | `comrak` → HTML, sanitizado com `ammonia` | HTML pronto e o front matter separado | Rolagem contínua |
| TXT | Própria (texto com `white-space: pre-wrap`) | Texto decodificado | Rolagem contínua |

- **Bytes pela ponte:** PDF e DOCX chegam como binário puro (`tauri::ipc::Response`), sem JSON nem base64.
- **Markdown seguro:** o `comrak` roda sem HTML embutido (`render.unsafe = false`) e a saída passa pelo `ammonia`. Um `.md` nunca executa script no app.
- **Front matter:** é separado do corpo e exibido como metadados, não como texto (spec 02).
- **Controles** (aparência em `Design/specs/05-document-view.md`): zoom, página atual e total, navegação entre páginas e tela cheia. Em MD e TXT, que rolam continuamente, a toolbar mostra só o zoom.

## Modelo de documento

Representação neutra de um documento, no `folio-core`. Os leitores produzem o modelo; os escritores da spec 06 o consomem.

```text
Documento { metadados, blocos: [Bloco] }

Bloco:  Titulo { nivel 1–6, conteudo: [Inline] }
        Paragrafo { conteudo: [Inline] }
        Lista { ordenada, itens: [[Bloco]] }        ← itens podem ter listas aninhadas
        Citacao { blocos: [Bloco] }
        Codigo { linguagem?, texto }
        Tabela { linhas: [[celula: [Inline]]] }     ← simples: sem células mescladas
        Imagem { bytes, mime, texto_alternativo }
        Separador
        QuebraDePagina

Inline: Texto | Negrito [Inline] | Italico [Inline] | CodigoInline | Link { destino, [Inline] } | QuebraDeLinha
```

**Fica de fora do modelo na 0.1.0:** notas de rodapé, cabeçalho e rodapé de página, colunas, cores, fontes e tamanhos específicos, comentários e revisões, campos (sumário, numeração automática). Numa conversão, o texto desses elementos é preservado quando possível como parágrafo comum; a formatação se perde.

## Leitores

| Formato | Base | O que é nosso | O que reconhece na 0.1.0 |
|---|---|---|---|
| TXT | — | Decodificar (UTF-8, com fallback para Windows-1252 via `encoding_rs`) e separar parágrafos por linha em branco | Parágrafos |
| MD | `comrak` (árvore do Markdown, front matter) | Traduzir a árvore do comrak para o modelo | Tudo do modelo |
| DOCX | `zip` + `quick-xml` | Ler `document.xml`, `styles.xml`, `numbering.xml` e `docProps/core.xml` e traduzir: estilos "Heading N" viram títulos, numeração vira lista, negrito e itálico viram ênfase | Tudo do modelo |
| PDF | A definir no spike S1 (`pdfium-render` ou `hayro`) | **A dedução da estrutura:** juntar caracteres em linhas, linhas em parágrafos, reconhecer títulos (tamanho e peso da fonte) e listas (marcadores e recuo) | Títulos, parágrafos, listas, ênfase. Tabelas e imagens não são reconhecidas: tabela vira parágrafos, imagem é ignorada |

O leitor de PDF é a peça de maior risco do Reader e da conversão: um PDF guarda posição na página, não estrutura. Um PDF digitalizado (só imagem) produz um documento vazio; OCR está fora da 0.1.0.

## Restrição de plataforma

Nenhuma engine depende de binário do sistema. pdf.js e docx-preview rodam no webview, igual no Linux e no Android. Se o spike S1 escolher o PDFium, a biblioteca `libpdfium` vai embarcada no app, uma por plataforma; nunca a do sistema.

## Critérios de aceite

- [ ] Os quatro formatos abrem no palco com a aparência de `Design/specs/05-document-view.md`, nos temas escuro e claro.
- [ ] PDF: página atual e total corretos, zoom e navegação entre páginas, texto selecionável.
- [ ] DOCX: título, parágrafos, listas, tabelas e imagens aparecem com layout reconhecível.
- [ ] MD: um `.md` com `<script>` ou `onerror=` não executa nada; o front matter aparece como metadados.
- [ ] TXT em Windows-1252 abre com acentos corretos.
- [ ] Cada leitor tem testes com arquivos de exemplo em `crates/folio-core/tests/fixtures/` e produz o modelo esperado.
- [ ] PDF digitalizado abre na exibição e produz documento vazio no leitor, sem erro.
