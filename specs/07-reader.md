# 07 · Reader

O Reader oferece a mesma experiência de leitura para os formatos da 0.1.0: **PDF, TXT, MD e DOCX**. XLSX, PPTX e imagens ficam para depois.

Esta spec cobre duas partes: a **exibição** (o que o usuário vê no palco) e os **leitores**, que transformam um arquivo no modelo de documento. Os leitores servem ao Reader, à conversão (spec 08), ao cache de metadados (tabela `document_metadata`, spec 04) e, no futuro, à busca no conteúdo.

## Spikes

Resolver antes de implementar a parte que cada spike bloqueia. O resultado vai para a seção da spec que ele decide e para o "Resultado" do spike.

### S1 · Leitor de PDF: PDFium ou hayro

**Bloqueia:** o leitor de PDF e os metadados de PDF (abaixo) e as conversões a partir de PDF (spec 08).

**Pergunta:** qual biblioteca dá ao leitor de PDF o texto de cada caractere com posição, fonte, tamanho e peso, com qualidade suficiente para deduzir a estrutura?

**Candidatos:**
- `pdfium-render` (PDFium, o motor do Chrome; MIT/Apache): robusto e completo, mas exige embarcar a `libpdfium` por plataforma (~5 MB cada).
- `hayro` (Rust puro; MIT/Apache): sem binário extra e mais simples de distribuir, inclusive no Android. Não está confirmado se expõe texto com posição.

**Como testar:** um corpus de ~10 PDFs reais em `crates/folio-core/tests/fixtures/pdf/`: artigo acadêmico em duas colunas, contrato, relatório com tabelas, slides exportados, PDF gerado pelo Word, PDF gerado pelo LaTeX, PDF com fontes sem mapa Unicode, PDF digitalizado. Para cada biblioteca, extrair caracteres com posição e fonte e comparar:

1. Texto completo e na ordem de leitura (contra o texto copiado de um leitor de PDF)
2. Posição, tamanho e peso da fonte disponíveis por caractere ou trecho
3. Metadados do dicionário Info (título, autor, assunto, palavras-chave, datas, versão)
4. Tempo de extração do corpus
5. Tamanho somado ao app e esforço de empacotar para Linux e Android

**Decisão:** o hayro, se cumprir 1, 2 e 3; senão, o PDFium.

**Resultado:** pendente.

### S2 · pdf.js no WebKitGTK e no WebView do Android

**Bloqueia:** a exibição de PDF (abaixo).

**Pergunta:** o `pdfjs-dist` atual roda no WebKitGTK do Linux e no WebView do Android, com o worker carregado dentro do Tauri?

**Como testar:** abrir os PDFs do corpus do S1 no app (`npm run tauri dev`), com o worker do pdf.js empacotado pelo Vite. Conferir renderização, seleção de texto, zoom e uso de memória num PDF de 300 páginas. Anotar a versão do WebKitGTK testada e a CSP necessária.

**Resultado:** pendente.

### S4 · Fidelidade do docx-preview

**Bloqueia:** a exibição de DOCX (abaixo).

**Pergunta:** o `docx-preview` exibe DOCX reais com fidelidade de layout razoável, e como contar páginas?

**Como testar:** corpus de ~10 DOCX reais (trabalho acadêmico com sumário, contrato, documento com tabelas e imagens, documento do Google Docs exportado). Comparar com o LibreOffice: texto, estilos, tabelas, imagens, cabeçalho e rodapé. Verificar o que a opção de quebra de página produz como contagem de páginas.

**Resultado:** pendente.

## Arquitetura

```text
                  ┌─ bytes / texto / HTML ─▶ exibição no webview (React)
arquivo ─▶ core ──┤
                  └─ leitor ─▶ modelo de documento ─▶ conversão, metadados, busca
```

- **A exibição acontece no webview.** Cada formato tem um componente de exibição em React, atrás de uma interface comum (página atual, total, zoom, navegação). O core não renderiza páginas: ele entrega ao frontend o conteúdo de que cada exibição precisa.
- **Os leitores rodam no `folio-core`**, em Rust, e produzem o modelo de documento.
- O frontend nunca lê arquivos diretamente: pede o conteúdo por command, pelo id estável do documento (spec 04).

## Exibição

| Formato | Engine | O core entrega | Páginas |
|---|---|---|---|
| PDF | `pdfjs-dist` (pdf.js) | Bytes do arquivo | Exatas, do próprio PDF |
| DOCX | `docx-preview` | Bytes do arquivo (decifrados em memória, se protegido) | Aproximadas: só as quebras explícitas do documento (o `docx-preview` não diagrama páginas) |
| MD | `comrak` → HTML, sanitizado com `ammonia` | HTML pronto e o front matter separado | Rolagem contínua |
| TXT | Própria (texto com `white-space: pre-wrap`) | Texto decodificado | Rolagem contínua |

- **Bytes pela ponte:** PDF e DOCX chegam como binário puro (`tauri::ipc::Response`), sem JSON nem base64.
- **Markdown seguro:** o `comrak` roda sem HTML embutido (`render.unsafe = false`) e a saída passa pelo `ammonia`. Um `.md` nunca executa script no app.
- **Documento protegido por senha** (spec 04): o PDF pede a senha pelo `onPassword` do pdf.js; o DOCX pede a senha no frontend, e o core decifra com `office-crypto` antes de entregar os bytes. A senha vai junto no command e nunca é guardada; o conteúdo decifrado nunca vai para o disco.
- **Front matter:** é separado do corpo e exibido como metadados, não como texto (spec 04).
- **Controles** (aparência em `Design/specs/05-document-view.md`): zoom, página atual e total, navegação entre páginas e tela cheia. Em MD e TXT, que rolam continuamente, a toolbar mostra só o zoom.

## Modelo de documento

Representação neutra de um documento, no `folio-core`. Os leitores produzem o modelo; os escritores da spec 08 o consomem.

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
| PDF | A definir no spike S1 (`pdfium-render` ou `hayro`), acima | **A dedução da estrutura:** juntar caracteres em linhas, linhas em parágrafos, reconhecer títulos (tamanho e peso da fonte) e listas (marcadores e recuo) | Títulos, parágrafos, listas, ênfase. Tabelas e imagens não são reconhecidas: tabela vira parágrafos, imagem é ignorada |

O leitor de PDF é a peça de maior risco do Reader e da conversão: um PDF guarda posição na página, não estrutura. Um PDF digitalizado (só imagem) produz um documento vazio; OCR está fora da 0.1.0.

**Senha:** os leitores de PDF e DOCX recebem uma senha opcional. Sem ela, um documento protegido falha com `SenhaNecessaria`; com a errada, `SenhaIncorreta` (spec 04). O leitor de DOCX decifra com `office-crypto`, em memória.

### Metadados

Os leitores também extraem os metadados que vão para o cache `document_metadata` (spec 04), sem precisar montar o documento inteiro:

| Formato | De onde |
|---|---|
| PDF | Dicionário Info e versão do cabeçalho; páginas exatas |
| DOCX | `docProps/core.xml` (título, autor, assunto, palavras-chave, comentários, revisão, datas); páginas aproximadas, como na exibição |
| MD | Front matter: `title`, `author`, `tags` (ou `keywords`), `date`; os demais campos vão para `extra` |
| TXT | Nenhum: só o filesystem |

Documento protegido por senha não tem metadados extraídos (spec 04).

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
- [ ] PDF e DOCX protegidos por senha abrem com a senha certa; o DOCX é decifrado em memória.
- [ ] Os metadados de PDF, DOCX e MD do corpus de fixtures chegam ao cache conforme a tabela de metadados.
