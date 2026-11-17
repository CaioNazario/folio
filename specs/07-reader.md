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

**Resultado:** o hayro cumpre 1, 2 e 3; o leitor de PDF usa `hayro-interpret` 0.9 e `hayro-font` 0.5 (2026-11-16).

O corpus foi gerado, não colhido: 11 PDFs feitos com LibreOffice (contrato, relatório com tabelas, artigo em duas colunas, slides do Impress, o `relatorio.docx` das fixtures exportado, um manual de 301 páginas), Ghostscript (fontes Type1 como as do LaTeX), à mão (as 14 fontes padrão sem embutir; fonte Type3 sem mapa Unicode), ImageMagick (página digitalizada) e qpdf (senha). O programa do spike ficou fora do repositório, porque depende da `libpdfium` baixada. As duas bibliotecas foram comparadas no Linux x86_64, com a `libpdfium` 160.0.8165 do `bblanchon/pdfium-binaries`:

| Critério | hayro | PDFium |
|---|---|---|
| 1. Texto e ordem (palavras e ordem contra o `pdftotext`, página a página) | 100% nos 9 PDFs com texto | 100% |
| 2. Posição e tamanho | Por glifo: a matriz dá origem e tamanho; a largura de avanço dá o fim | Por caractere (`origin`, `loose_bounds`, `scaled_font_size`) |
| 2. Fonte e peso | Por casamento: cada glifo traz a identidade do programa da fonte, e o leitor a casa com as fontes dos recursos da página, carregadas pelo `FontCache` público; o dicionário dá `BaseFont`, `FontWeight`, `Flags` e `ItalicAngle`. Certo em todas as fontes do corpus, inclusive Type1 e padrão | `font_weight` errado: Liberation Serif Bold sai 400, Times-Bold 555 e Times-Roman 580, fonte padrão 0; não marca itálico em Type1 |
| 3. Metadados | Info inteiro e versão do cabeçalho; os textos vêm em bytes (o leitor decodifica UTF-16BE com BOM e PDFDocEncoding), as datas já estruturadas | Info como texto, datas como texto `D:...` |
| 4. Tempo (manual de 301 páginas, 1,37 milhão de caracteres) | 0,6 a 0,8 s | 1,4 a 2,3 s; procura fontes do sistema para as fontes padrão |
| 5. Tamanho e empacotamento | +4,5 MB no binário (LTO, com as fontes padrão e CMaps embutidas); Rust puro, compila para Android pelo cargo | `libpdfium.so` de 8,3 MB (Linux x86_64) e 6,8 MB (Android arm64), uma por plataforma, carregada em tempo de execução |

Os dois casos em que nenhuma biblioteca extrai texto são os esperados: a página digitalizada não tem caracteres, e a fonte Type3 sem mapa Unicode não tem texto recuperável (o hayro devolve "sem Unicode"; o PDFium devolve o código do caractere como se fosse a letra).

Senha, nas duas bibliotecas: RC4 de 40 e 128 bits e AES de 128 e 256 bits abrem com a senha certa; sem ela ou com a errada, o hayro dá `PasswordProtected`, sem distinguir os dois casos. Um PDF com só senha de dono abre sem senha, mas recusa uma senha errada.

Robustez: 84 PDFs do corpus truncados ou com bytes trocados ao acaso. Nenhum pânico nem travamento; o hayro leu 62 e o PDFium 51.

Riscos que o corpus gerado não cobre e que os testes do leitor devem cobrir com PDFs reais: PDF do Word e do pdflatex (fonte Type1 com `FontFile`), texto em CJK, e fontes declaradas nos recursos de um Form XObject em vez dos da página (o casamento precisa descer nos recursos dos XObjects).

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

**Âncoras no MD:** cada título do HTML sai com `id="folio-h-<âncora>"`, gerado do texto do título pelo comrak (minúsculas, espaços viram `-`, pontuação sai; um título repetido ganha `-1`, `-2`...), sem link de âncora ao lado. O prefixo impede que um título tome um id do próprio app. O `Anchor` traz o fragmento com os escapes `%` decodificados e sem o prefixo; o frontend rola até o elemento `folio-h-` + fragmento.

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

O command é `get_properties` e devolve um `DocumentProperties`: o `Document` (arquivo), o `location` (projeto e pastas), o `metadata` e as ligações de conversão (`convertedFrom`, `derivatives`). `metadata` vem vazio quando nada foi extraído: documento protegido ou inválido; o frontend escolhe a mensagem por `protected` e `valid`.

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

A CSP do app fica em `app.security.csp` no `tauri.conf.json`:

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
- **Em desenvolvimento** (`app.security.devCsp`): a mesma lista, com `ws://localhost:1420` e `ws://localhost:1421` a mais no `connect-src`, para o recarregamento do Vite. O build de release usa só a `csp`.
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
| DOCX | `zip` + `quick-xml` | Ler `document.xml`, `styles.xml`, `numbering.xml`, as relações de `document.xml` e `docProps/core.xml` e traduzir. O estilo vale pelo nome (`w:name`), não pelo id, que o Word traduz (`Ttulo1`): "heading N" vira título, "Quote" e "Intense Quote" viram citação (parágrafos seguidos numa só). Numeração vira lista, ordenada a menos que o formato do nível seja `bullet` ou `none`, aninhada pelo nível (`w:ilvl`). Negrito e itálico (desligados por `w:val` `0`, `false` ou `off`) viram ênfase, e runs vizinhos com a mesma ênfase se juntam. `w:hyperlink` vira link para o alvo da relação ou para `#âncora`. `w:br` de página vira quebra de página; o resto de `w:br`, quebra de linha. Imagem embutida (PNG, JPEG, GIF, WebP ou SVG, pela assinatura) vira bloco de imagem no ponto em que aparece, com o `descr` como texto alternativo; imagem vinculada ou de outro tipo (EMF, WMF) é ignorada. Célula de tabela junta os parágrafos com quebra de linha. Parágrafo vazio é descartado; documento sem `w:body` é vazio | Tudo do modelo, menos código e separador, que não têm equivalente no DOCX |
| PDF | `hayro-interpret` (interpreta a página e entrega cada glifo com Unicode e posição) + `hayro-font` (fonte de cada glifo, pelo casamento descrito no S1) | **A dedução da estrutura**, na ordem do conteúdo, sem reordenar pela geometria (um processador de texto desenha uma coluna depois da outra). Glifo sem Unicode é descartado. Uma linha nova começa quando a linha de base se move mais de meio corpo; um espaço entra onde o vão entre glifos passa de 0,15 do corpo e não há espaço. Linhas do mesmo tamanho, com a linha de base até 1,8 corpo abaixo da anterior, formam um parágrafo, unidas por espaço, ou por nada depois de uma linha que termina em hífen (o hífen fica). O corpo do texto é o tamanho com mais glifos no documento; um tamanho a partir de 1,15 vez o corpo é título, com o nível pela ordem dos tamanhos, do maior (1) ao sexto. Negrito que cobre o título inteiro é descartado. Ênfase vem do dicionário da fonte do glifo, procurado nos recursos da página e dos Form XObjects que ela desenha: negrito por `FontWeight` a partir de 600, ou, sem ele, pelo nome (bold, black, heavy, semibold, demi) ou por `StemV` a partir de 110 (o CMBX10 do pdflatex tem 114, o CMR10 69), ou pela flag ForceBold; itálico pela flag Italic, por `ItalicAngle` diferente de zero ou pelo nome (italic, oblique). Uma linha que começa com marcador (`•`, `◦`, `▪`, `–`, `-` ou `*`, seguido de espaço) ou número de até três algarismos com `.` ou `)` abre um item de lista; o marcador sai do texto, itens seguidos formam uma lista, ordenada se o primeiro for numerado, e um item recuado mais de 2 pt em relação ao menos recuado aninha sob o anterior. Entre páginas entra uma quebra de página. Página sem texto não gera blocos | Títulos, parágrafos, listas, ênfase. Tabelas e imagens não são reconhecidas: tabela vira parágrafos, imagem é ignorada |

O leitor de PDF é a peça de maior risco do Reader e da conversão: um PDF guarda posição na página, não estrutura. Um PDF digitalizado (só imagem) produz um documento vazio; OCR está fora da 0.1.0.

**Senha:** os leitores de PDF e DOCX recebem uma senha opcional. Sem ela, um documento protegido falha com `PasswordRequired`; com a errada, `WrongPassword` (spec 04). O leitor de DOCX decifra com `office-crypto`, em memória; a senha é ignorada num DOCX sem proteção. O `office-crypto` não confere a senha: com a errada, o resultado não é um ZIP, e isso vira `WrongPassword`. Um contêiner cifrado malformado é `ReadFailed`, inclusive quando o `office-crypto` entra em pânico com ele (o pânico é contido). O leitor de PDF tenta primeiro a senha vazia, que abre um PDF sem senha de usuário (só com senha de dono), e a senha recebida é ignorada nesse caso; senão, `PasswordProtected` do hayro vira `PasswordRequired` sem senha e `WrongPassword` com ela. Os bytes que o palco recebe (`document_bytes`) seguem a mesma regra: DOCX protegido sai decifrado; PDF sai como está, porque quem pede a senha é o pdf.js.

### Metadados

Os leitores também extraem os metadados que vão para o cache `document_metadata` (spec 04), sem precisar montar o documento inteiro:

| Formato | De onde |
|---|---|
| PDF | Dicionário Info (título, autor, assunto, palavras-chave, datas, criador, produtor) e versão do cabeçalho; páginas exatas. Os textos são decodificados como UTF-16BE quando começam pela marca de ordem, senão como PDFDocEncoding; as palavras-chave são separadas por vírgula ou ponto e vírgula; as datas (`D:AAAAMMDDHHmmSS` com fuso) viram ms UTC; o criador, o produtor e a versão (`1.4`, `2.0`) vão para `extra` como `creator`, `producer` e `version`. Propriedade em branco é ignorada |
| DOCX | `docProps/core.xml` (título, autor, assunto, palavras-chave, comentários, revisão, datas); páginas aproximadas, como na exibição: uma, mais uma por quebra de página explícita. As palavras-chave são separadas por vírgula ou ponto e vírgula; as datas vêm em W3CDTF; comentários (`dc:description`) e revisão vão para `extra` como `comments` e `revision`. Propriedade vazia é ignorada, e um DOCX sem `core.xml` só tem as páginas |
| MD | Front matter: `title`, `author`, `tags` (ou `keywords`; lista ou um texto só), `date`; os demais campos vão para `extra`, convertidos de YAML para JSON. `date` vira a data de criação interna: instante RFC 3339 ou `AAAA-MM-DD` (meia-noite UTC); uma data ilegível fica em `extra` como texto. Front matter com YAML inválido continua fora do corpo e não gera metadados |
| TXT | Nenhum: só o filesystem |

Documento protegido por senha não tem metadados extraídos (spec 04).

## Erros

Além dos da spec 04 (`DocumentNotFound`, `InvalidDocument`, `PasswordRequired`, `WrongPassword`):

| Erro | Quando |
|---|---|
| `ReadFailed` | O leitor não conseguiu montar o documento (ZIP ou XML quebrado, PDF corrompido por dentro); o motivo técnico vai para o log |

## Restrição de plataforma

Nenhuma engine depende de binário do sistema. pdf.js e docx-preview rodam no webview, igual no Linux e no Android. O leitor de PDF é Rust puro (hayro, spike S1), com as 14 fontes padrão embutidas: não procura fontes do sistema.

## Critérios de aceite

- [ ] Os quatro formatos abrem no palco com a aparência de `Design/specs/05-document-view.md`, nos temas escuro e claro.
- [ ] PDF: página atual e total corretos, zoom e navegação entre páginas, texto selecionável.
- [ ] DOCX: título, parágrafos, listas, tabelas e imagens aparecem com layout reconhecível.
- [ ] MD: um `.md` com `<script>` ou `onerror=` não executa nada; o front matter aparece como metadados.
- [x] TXT em Windows-1252 e em UTF-16 com BOM abre com acentos corretos.
- [x] Cada leitor tem testes com arquivos de exemplo e produz o modelo esperado: TXT, MD e DOCX com arquivos em `crates/folio-core/tests/fixtures/`; PDF com documentos gerados no próprio teste (`tests/readers_pdf.rs`, pelo `krilla` com as fontes do `typst-assets`, e PDFs escritos à mão nos testes do leitor), sem PDF versionado.
- [ ] PDF digitalizado abre na exibição e produz documento vazio no leitor, sem erro.
- [ ] PDF e DOCX protegidos por senha abrem com a senha certa; o DOCX é decifrado em memória.
- [x] Os metadados de PDF, DOCX e MD dos arquivos de exemplo (gerados no teste, no PDF) chegam ao cache conforme a tabela de metadados.
- [ ] O documento reabre na posição em que o usuário parou.
- [ ] Um link `https:` abre no navegador do sistema; um link relativo para outro documento do workspace abre no Folio; nenhum link navega o webview.
- [x] Imagem relativa dentro do workspace aparece no MD; imagem remota não é baixada e vira espaço reservado.
- [x] Um TXT de 50 MB abre sem travar, com o aviso de corte.
- [ ] Cada estado do palco (carregando, senha, inválido, não encontrado, falha ao exibir, vazio) aparece na situação certa.
- [ ] Propriedades mostra arquivo, metadados e conversões, com os avisos de desatualizado.
- [x] Os atalhos de teclado funcionam e não disparam enquanto o foco está num campo de texto.
- [x] A CSP está definida no `tauri.conf.json` e nenhum documento consegue carregar recurso da rede.
