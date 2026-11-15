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

**Como testar:** abrir os PDFs do corpus do S1 no app (`npm run tauri dev`), com o worker do pdf.js empacotado pelo Vite. Conferir renderização, seleção de texto, zoom e uso de memória num PDF de 300 páginas. Medir também um PDF de 200 MB entregue pela ponte (`tauri::ipc::Response`): memória e tempo até a primeira página. Se não aguentar, o pdf.js passa a ler por faixas de bytes, servidas por um protocolo próprio do app pelo id do documento (nunca o protocolo de assets, que expõe o filesystem). Confirmar a CSP da seção "Segurança do webview".

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
| TXT | Própria (texto com `white-space: pre-wrap`) | Texto decodificado (ver o leitor de TXT) | Rolagem contínua |

- **Bytes pela ponte:** PDF e DOCX chegam como binário puro (`tauri::ipc::Response`), sem JSON nem base64.
- **Markdown seguro:** o `comrak` roda sem HTML embutido (`render.unsafe = false`) e a saída passa pelo `ammonia`. Um `.md` nunca executa script no app.
- **Documento protegido por senha** (spec 04): o PDF pede a senha pelo `onPassword` do pdf.js; o DOCX pede a senha no frontend, e o core decifra com `office-crypto` antes de entregar os bytes. A senha vai junto no command e nunca é guardada; o conteúdo decifrado nunca vai para o disco.
- **Front matter:** só em YAML, entre linhas `---` no início do arquivo. É separado do corpo e não aparece como texto: os campos vão para os metadados (spec 04) e aparecem em Propriedades. Um bloco entre `+++` (TOML) não é reconhecido e aparece como texto comum.
- **Controles** (aparência em `Design/specs/05-document-view.md`): zoom, página atual e total, navegação entre páginas e tela cheia. Em MD e TXT, que rolam continuamente, a toolbar mostra só o zoom. Zoom de 50% a 300%, em passos de 10%, igual para todos os documentos durante a sessão.
- **Posição:** o documento abre onde o usuário parou (página no PDF e no DOCX, fração da rolagem no MD e no TXT), e a exibição informa a posição ao sair (spec 06).

### Arquivos grandes

- **MD e TXT acima de 10 MB:** o core lê só os primeiros 10 MB, corta no fim da última linha completa e avisa que cortou. O palco mostra "Arquivo grande: mostrando os primeiros 10 MB". Um caractere UTF-8 partido pelo corte é descartado, sem fazer o texto cair no Windows-1252; uma linha única maior que o limite é cortada no último caractere inteiro. A conversão e os metadados continuam usando o arquivo inteiro.
- **PDF e DOCX:** entregues inteiros; o limite prático do PDF sai do S2.

### Links

Clicar num link dentro do documento (MD, DOCX ou PDF) nunca navega o webview. O frontend intercepta o clique e pergunta ao core o que o link é; a decisão fica no core:

| Link | O que acontece |
|---|---|
| `http:`, `https:`, `mailto:` | Abre no navegador ou no app de e-mail do sistema (`open_url` do `plugin-opener`) |
| Relativo, para um documento do workspace (`[ver](../Notas.md)`) | Abre esse documento no Folio |
| Relativo, para outro arquivo do workspace ou para fora dele | Nada abre; toast "Este link aponta para um arquivo que o Folio não abre" |
| `#âncora` | Rola até o ponto, no MD; vai à página de destino, no PDF |
| Qualquer outro esquema (`file:`, `javascript:`...) | Ignorado |

O caminho relativo é resolvido a partir do diretório do documento e precisa continuar dentro do workspace (nunca em `.folio/`); os escapes `%` são decodificados, e o fragmento e a query depois do arquivo (`Notas.md#secao`) são ignorados. O documento de destino precisa estar no índice e no disco. A resposta é um `LinkTarget` com tag `kind`: `External`, `Document`, `Unsupported`, `Anchor` ou `Ignored`.

### Imagens no Markdown

O HTML de um `.md` nunca carrega imagem por conta própria. Depois do `ammonia`, o core reescreve cada `<img>`:

| Origem da imagem | Resultado |
|---|---|
| Arquivo dentro do workspace, relativo ao documento | Embutido no HTML como `data:` (PNG, JPEG, GIF, WebP ou SVG, confirmados por assinatura) |
| Arquivo fora do workspace, inexistente ou de outro tipo | Espaço reservado com o texto alternativo |
| Remota (`https://...`) | **Não é baixada:** espaço reservado com o texto alternativo e o endereço como link. O Folio é offline-first, e uma imagem remota avisaria um terceiro de que o documento foi aberto |

Limite de 10 MB por imagem e 50 MB por documento; o que passar vira espaço reservado. O tamanho é conferido antes da leitura: um arquivo grande demais nunca é lido. SVG só entra como `<img>`, onde não executa script. A reescrita vem depois do `ammonia`, que remove qualquer `data:` vindo do próprio documento: só sobrevive o que o core gerou. Para isso, antes de gerar o HTML, o core troca o endereço de cada imagem por um marcador `folio-image:N`, que o `ammonia` deixa passar, e depois troca cada marcador pelo resultado.

**Espaço reservado:** `<span data-folio-image="motivo">texto alternativo</span>`, com o motivo `remote`, `missing`, `outside`, `unsupported` (esquema que não é `http(s)`, ou conteúdo que não é imagem reconhecida) ou `too-large`. A imagem remota também leva o endereço como link. O core não escreve texto de interface no HTML: o frontend rotula e estiliza o espaço reservado pelo motivo.

### Estados do palco

O palco tem um estado para cada situação, todos com os tokens e componentes de `Design/specs/06-components.md` e textos do i18n:

| Estado | Quando | O que mostra |
|---|---|---|
| Carregando | O conteúdo ainda não chegou | Nada nos primeiros 300 ms (para não piscar); depois, "Abrindo…" |
| Senha | Documento protegido (spec 04) | Campo de senha e "Abrir" no próprio palco; senha errada mostra o erro no campo |
| Inválido | Documento inválido (spec 04) | O motivo ("Este arquivo não é um PDF válido") e "Mostrar na pasta" |
| Não encontrado | O arquivo sumiu do disco | "Este documento não está mais no disco"; a árvore é reconciliada (spec 05) |
| Falha ao exibir | Passou na assinatura, mas o leitor ou a engine falhou (arquivo corrompido por dentro) | "Não foi possível exibir este documento", com "Tentar de novo" e "Mostrar na pasta". O detalhe técnico vai para o log |
| Vazio | MD ou TXT sem conteúdo | "Documento vazio" |

### Propriedades

O menu Mais ganha o item **Propriedades** (`Design/specs/05-document-view.md`), que abre um diálogo com tudo o que o Folio sabe do documento, num único command:

- **Arquivo:** nome, local (caminho relativo), formato, tamanho, criado e modificado.
- **Documento:** os metadados do formato (spec 04): título, autor, assunto, palavras-chave, datas internas, páginas e os campos de `extra`. Extraídos na hora se o cache estiver velho. Em documento protegido, "Protegido por senha: metadados não extraídos".
- **Conversão:** de qual documento ele veio (formato e data) e quais derivados tem, cada um com o aviso de desatualizado, se houver (spec 04). Clicar num deles abre o documento.

### Atalhos de teclado

Valem com a visualização de documento aberta e o foco fora de campos de texto:

| Tecla | Ação |
|---|---|
| `Page Down`, `→` / `Page Up`, `←` | Próxima / página anterior (PDF, DOCX); rola uma tela (MD, TXT) |
| `Home` / `End` | Primeira / última página, ou início / fim |
| `Ctrl` `+` / `Ctrl` `-` / `Ctrl` `0` | Mais zoom / menos zoom / 100% |
| `F11` | Liga e desliga o fullscreen do palco |
| `Esc` | Fecha o menu aberto; sem menu, sai do fullscreen |

### Fora da 0.1.0

- **Procurar no documento** (`Ctrl+F`): fica para depois, junto com a busca no conteúdo.

### Segurança do webview

A CSP do app (`app.security.csp` no `tauri.conf.json`, hoje `null`) é definida junto com o Reader, antes do Reader de MD:

```text
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
img-src 'self' data: blob:;
font-src 'self' data: blob:;
worker-src 'self' blob:;
connect-src 'self' ipc: http://ipc.localhost;
object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'
```

- Nenhuma origem de rede: o webview não carrega nada de fora, nem por engano de um documento.
- `'wasm-unsafe-eval'`: decodificadores de imagem do pdf.js em WebAssembly.
- `style-src 'unsafe-inline'`: o `docx-preview` injeta os estilos do documento.
- `blob:`: worker do pdf.js e imagens e fontes do `docx-preview`.

Os spikes S2 e S4 confirmam a lista; o que eles mostrarem desnecessário sai.

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
| TXT | — | Decodificar com `encoding_rs`: pela marca de início (BOM) quando existe (UTF-8, UTF-16 LE ou BE), senão UTF-8, com fallback para Windows-1252. Separar parágrafos por linha em branco (ou só com espaços); uma quebra simples vira quebra de linha dentro do parágrafo, e `\r\n` conta como `\n` | Parágrafos |
| MD | `comrak` (árvore do Markdown, com tabelas, tachado, listas de tarefas e front matter com delimitador `---`) + `saphyr` (YAML 1.2 em Rust puro, sem `unsafe`; o `serde_yaml` está descontinuado) | Traduzir a árvore do comrak para o modelo. Tachado vira texto comum, item de tarefa vira item de lista, quebra suave vira espaço. Tags HTML inline são descartadas e o texto entre elas fica; um bloco HTML inteiro é descartado. Até os leitores resolverem arquivos de imagem, uma imagem vira o seu texto alternativo | Tudo do modelo |
| DOCX | `zip` + `quick-xml` | Ler `document.xml`, `styles.xml`, `numbering.xml` e `docProps/core.xml` e traduzir: estilos "Heading N" viram títulos, numeração vira lista, negrito e itálico viram ênfase | Tudo do modelo |
| PDF | A definir no spike S1 (`pdfium-render` ou `hayro`), acima | **A dedução da estrutura:** juntar caracteres em linhas, linhas em parágrafos, reconhecer títulos (tamanho e peso da fonte) e listas (marcadores e recuo) | Títulos, parágrafos, listas, ênfase. Tabelas e imagens não são reconhecidas: tabela vira parágrafos, imagem é ignorada |

O leitor de PDF é a peça de maior risco do Reader e da conversão: um PDF guarda posição na página, não estrutura. Um PDF digitalizado (só imagem) produz um documento vazio; OCR está fora da 0.1.0.

**Senha:** os leitores de PDF e DOCX recebem uma senha opcional. Sem ela, um documento protegido falha com `PasswordRequired`; com a errada, `WrongPassword` (spec 04). O leitor de DOCX decifra com `office-crypto`, em memória.

### Metadados

Os leitores também extraem os metadados que vão para o cache `document_metadata` (spec 04), sem precisar montar o documento inteiro:

| Formato | De onde |
|---|---|
| PDF | Dicionário Info e versão do cabeçalho; páginas exatas |
| DOCX | `docProps/core.xml` (título, autor, assunto, palavras-chave, comentários, revisão, datas); páginas aproximadas, como na exibição |
| MD | Front matter: `title`, `author`, `tags` (ou `keywords`; lista ou um texto só), `date`; os demais campos vão para `extra`, convertidos de YAML para JSON. `date` vira a data de criação interna: instante RFC 3339 ou `AAAA-MM-DD` (meia-noite UTC); uma data ilegível fica em `extra` como texto. Front matter com YAML inválido continua fora do corpo e não gera metadados |
| TXT | Nenhum: só o filesystem |

Documento protegido por senha não tem metadados extraídos (spec 04).

## Erros

Além dos da spec 04 (`DocumentNotFound`, `InvalidDocument`, `PasswordRequired`, `WrongPassword`):

| Erro | Quando |
|---|---|
| `ReadFailed` | O leitor não conseguiu montar o documento (ZIP ou XML quebrado, PDF corrompido por dentro); o motivo técnico vai para o log |

## Restrição de plataforma

Nenhuma engine depende de binário do sistema. pdf.js e docx-preview rodam no webview, igual no Linux e no Android. Se o spike S1 escolher o PDFium, a biblioteca `libpdfium` vai embarcada no app, uma por plataforma; nunca a do sistema.

## Critérios de aceite

- [ ] Os quatro formatos abrem no palco com a aparência de `Design/specs/05-document-view.md`, nos temas escuro e claro.
- [ ] PDF: página atual e total corretos, zoom e navegação entre páginas, texto selecionável.
- [ ] DOCX: título, parágrafos, listas, tabelas e imagens aparecem com layout reconhecível.
- [ ] MD: um `.md` com `<script>` ou `onerror=` não executa nada; o front matter aparece como metadados.
- [ ] TXT em Windows-1252 e em UTF-16 com BOM abre com acentos corretos.
- [ ] Cada leitor tem testes com arquivos de exemplo em `crates/folio-core/tests/fixtures/` e produz o modelo esperado.
- [ ] PDF digitalizado abre na exibição e produz documento vazio no leitor, sem erro.
- [ ] PDF e DOCX protegidos por senha abrem com a senha certa; o DOCX é decifrado em memória.
- [ ] Os metadados de PDF, DOCX e MD do corpus de fixtures chegam ao cache conforme a tabela de metadados.
- [ ] O documento reabre na posição em que o usuário parou.
- [ ] Um link `https:` abre no navegador do sistema; um link relativo para outro documento do workspace abre no Folio; nenhum link navega o webview.
- [ ] Imagem relativa dentro do workspace aparece no MD; imagem remota não é baixada e vira espaço reservado.
- [ ] Um TXT de 50 MB abre sem travar, com o aviso de corte.
- [ ] Cada estado do palco (carregando, senha, inválido, não encontrado, falha ao exibir, vazio) aparece na situação certa.
- [ ] Propriedades mostra arquivo, metadados e conversões, com os avisos de desatualizado.
- [ ] Os atalhos de teclado funcionam e não disparam enquanto o foco está num campo de texto.
- [ ] A CSP está definida no `tauri.conf.json` e nenhum documento consegue carregar recurso da rede.
