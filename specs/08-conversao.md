# 08 · Conversão

A conversão gera um **novo arquivo** no filesystem e nunca altera o original. Formatos da 0.1.0: PDF, TXT, MD e DOCX, em qualquer direção (12 conversões).

## Spikes

Resolver antes de implementar a parte que cada spike bloqueia. O resultado vai para a seção da spec que ele decide e para o "Resultado" do spike.

### S3 · Typst embarcado

**Bloqueia:** o escritor de PDF (abaixo).

**Pergunta:** quanto o Typst custa no app, e ele gera um PDF bom a partir do modelo de documento?

**Como testar:** escritor mínimo modelo → Typst → PDF com `typst-as-lib` e `typst-pdf`, com as fontes do `typst-assets`. Medir:

1. Tamanho do binário do app antes e depois
2. Tempo de compilação limpa e incremental do workspace
3. Tempo para converter um MD de ~50 páginas
4. Escape: texto com `#`, `$`, `*`, `_`, `@` e `<` sai literal
5. Compila para Android (`aarch64-linux-android`)

**Resultado (Linux, 2026-11-22):** o Typst fica. Medido com o protótipo do escritor e um MD de 22.465 palavras (45 páginas no PDF):

| Medida | Sem Typst | Com Typst |
|---|---|---|
| Binário de release | 28,1 MB | 90,3 MB |
| Binário de release sem símbolos (`strip`) | 21,2 MB | 69,9 MB |
| Binário compactado com xz (perto do download) | 8,2 MB | 22,8 MB |
| Build limpo do workspace (dev) | 4 min 23 s | 11 min 48 s |
| Build incremental (dev, `-j 2`) | 23,7 s | 46 s |

1. **Tamanho:** +62 MB no binário, aceito. São ~22 MB de código e ~18 MB de dados (9,6 MB de fontes, mais destaque de sintaxe, hifenização e tabelas Unicode). Os crates do Typst não têm features para desligar partes, e LTO não reduz o binário. Decisões: todas as fontes do `typst-assets` ficam (não só as duas usadas), e os perfis de build não mudam (as informações de depuração ficam, no dev e no release).
2. **Compilação:** os binários de teste passam de ~570 MB e cada link usa ~1,7 GB de memória. Builds locais limitam os jobs (`-j 2` ou `-j 3`) para não esgotar a memória.
3. **Tempo:** 0,5 a 0,7 s para as 45 páginas (a leitura do MD leva 3 a 6 ms).
4. **Escape:** `#`, `$`, `*`, `_`, `@`, `<` e `[` saem literais (conferido com `pdftotext`). O texto do documento nunca entra no código Typst: o modelo vai para o template como dados, em `sys.inputs` (ver "Escritores").
5. **Android:** pendente, junto com o teste do S2 e do S4 no Android, ao fechar a 0.1.0.

## Arquitetura

Toda conversão é um leitor (spec 07) seguido de um escritor, ligados pelo modelo de documento:

```text
origem ─▶ leitor do formato de origem ─▶ modelo de documento ─▶ escritor do formato de destino ─▶ destino
```

Com 4 leitores e 4 escritores, as 12 conversões saem das combinações. Melhorar um leitor melhora todas as conversões que saem daquele formato. É a mesma arquitetura do Pandoc, reproduzida em Rust com bibliotecas embarcáveis.

A conversão reconstrói o documento: preserva texto, ordem e estrutura (títulos, listas, ênfase, tabelas, imagens, conforme o leitor de origem reconhece), mas não a aparência exata da origem: posições, colunas, espaçamento, alinhamento, cor e a fonte de cada trecho se perdem. Do PDF para o DOCX, a tipografia do documento (fonte e tamanho do corpo e de cada nível de título) é preservada.

## Escritores

| Formato | Base | O que é nosso |
|---|---|---|
| TXT | — | Imprimir o texto (regras abaixo) |
| MD | `comrak` (impressão de CommonMark a partir da árvore) + `saphyr` (front matter) | Montar a árvore do comrak a partir do modelo (regras abaixo) |
| DOCX | `zip` + `quick-xml` (os mesmos do leitor de DOCX) | Montar o pacote do Word a partir do modelo (regras abaixo). O `docx-rs` foi descartado: não grava título, assunto, autor nem palavras-chave, e grava autor "unknown" e datas de 1970 quando faltam |
| PDF | `typst` (biblioteca, via `typst-as-lib`) + `typst-pdf` | Um template Typst fixo (página, margens, fontes, tamanhos) e a tradução do modelo em dados para ele: dicionários com o tipo de cada bloco e de cada trecho, passados em `sys.inputs`. O template monta o documento a partir desses dados, então o texto do documento nunca é interpretado como código Typst e não precisa de escape |

**TXT:** texto em UTF-8 sem BOM, linhas terminadas em `\n`. É o único destino sem marcação; o que o texto puro não representa, como ênfase, metadados e páginas, se perde:

- **Blocos:** separados por uma linha em branco. O arquivo termina numa quebra de linha; modelo sem blocos gera arquivo vazio.
- **Título:** só o texto, numa linha.
- **Parágrafo:** o texto dos trechos; a quebra de linha vira `\n`. Ênfase e código inline ficam só com o texto.
- **Link:** o texto seguido do endereço entre parênteses, `Folio (https://example.org)`. Quando o texto já é o endereço, ele aparece uma vez só; link interno (`#...`) fica só com o texto.
- **Lista:** um item por linha, com `- ` ou `1. `, `2. `…, sem linha em branco entre os itens. O resto do item (outra linha, outro bloco, uma sublista) é recuado até o começo do texto do item.
- **Citação:** cada linha começa com `> `.
- **Código:** as linhas como estão, recuadas 4 espaços; a linguagem se perde.
- **Tabela:** uma linha por linha da tabela, com as células separadas por tabulação, para colar numa planilha. Quebra de linha dentro da célula vira espaço.
- **Imagem:** ver "Imagens".
- **Separador:** uma linha `---`.
- **Quebra de página e metadados:** somem.

**MD:** CommonMark com tabelas do GitHub, em UTF-8 sem BOM, linhas terminadas em `\n`. O escritor monta a árvore do comrak a partir do modelo e o comrak a imprime, então o escape do Markdown é o do comrak. O escritor devolve o texto do MD e as imagens a gravar na pasta `.assets`, e recebe o nome final dessa pasta (ver "Imagens"):

- **Metadados:** viram front matter YAML entre linhas `---`, só com os campos que existem: `title`, `author`, `subject`, `tags` (lista, das palavras-chave) e `date` (a data de criação, em RFC 3339 UTC). Sem nenhum desses campos, o arquivo não tem front matter. O `extra` não entra: é próprio do formato de origem.
- **Texto:** os caracteres que o Markdown leria como marcação (`*`, `_`, `#`, `[`, `<`...) saem escapados. Relido pelo leitor de MD, o texto é o mesmo.
- **Título, parágrafo, ênfase, código inline e link:** a marcação do Markdown (`#` a `######`, `**`, `*`, crases, `[texto](endereço)`). A quebra de linha vira quebra forçada.
- **Lista:** `-` ou `1.`, com aninhamento. Sem linha em branco entre os itens, a não ser que algum item tenha mais de um bloco além das sublistas (dois parágrafos, um parágrafo e um código): aí a lista sai espaçada, com linha em branco entre os blocos, senão os parágrafos de um item se juntariam ao reler.
- **Citação:** `>`.
- **Código:** bloco cercado por crases, com a linguagem quando existe.
- **Tabela:** tabela do GitHub; a primeira linha vira o cabeçalho. Linhas com menos células são completadas com células vazias, porque a tabela do GitHub exige o mesmo número de colunas em todas.
- **Imagem:** um parágrafo só com `![texto alternativo](pasta/imagem-N.ext)`, com N contado a partir de 1 na ordem do documento e a extensão do tipo (`png`, `jpg`, `gif`, `webp`, `svg`).
- **Separador:** quebra temática.
- **Quebra de página:** some.

**DOCX:** um pacote do Word montado pelo Folio, só com as partes necessárias: `[Content_Types].xml`, as relações, `word/document.xml`, `word/styles.xml`, `word/numbering.xml` (quando há lista), as imagens em `word/media/` e `docProps/core.xml` (quando há metadados). Relido pelo leitor de DOCX (spec 07), o modelo volta igual, menos o que esse leitor não reconhece: código, separador e código inline, que voltam como texto comum ou somem.

- **Página:** A4, margens de 2,5 cm, como no PDF.
- **Estilos:** os nomes internos do Word, que o Word e o LibreOffice mostram traduzidos: "Normal", "heading 1" a "heading 6", "Quote" e "Hyperlink", mais "Code" para código. Texto na fonte e no tamanho do corpo da tipografia do modelo; sem eles, em Calibri 11 pt (o LibreOffice usa a Carlito, de mesmas medidas). Cada "heading N" usa o tamanho do nível na tipografia (sem ele, 16, 14, 13, 12, 11 e 11 pt), a fonte do nível só quando difere da do corpo, e negrito quando o nível é negrito ou não há tipografia. O tamanho é copiado como está, mesmo de página que não é A4: um título de 40 pt de um PDF de slides continua com 40 pt na página A4. O DOCX só nomeia a fonte, sem embuti-la: quem abre usa a fonte se ela estiver instalada, ou a substituta que o programa escolher. Código em Courier New.
- **Parágrafo e título:** um parágrafo com o estilo; a quebra de linha vira `w:br`. Negrito e itálico viram `w:b` e `w:i`; código inline, um trecho em Courier New.
- **Link:** externo, por uma relação com `TargetMode="External"`; `#âncora`, por `w:anchor`. O texto do link leva o estilo de caractere "Hyperlink" (azul e sublinhado), como o Word faz.
- **Lista:** parágrafos numerados, com o nível (`w:ilvl`) dado pelo aninhamento, com marcador ou número decimal. Cada lista numerada começa do 1. Os blocos de um item depois do primeiro saem recuados até o texto do item, sem número; relidos, ficam fora da lista, porque o leitor só reconhece como item o parágrafo numerado.
- **Citação:** os parágrafos e títulos dela com o estilo "Quote"; os outros blocos dentro dela saem como se estivessem fora.
- **Código:** um parágrafo com o estilo "Code", com as linhas separadas por `w:br` e os espaços preservados.
- **Tabela:** grade com bordas, com a primeira linha como cabeçalho repetido em cada página. Linhas com menos células são completadas com células vazias.
- **Imagem:** embutida no ponto em que aparece, com o texto alternativo em `descr`. O tamanho é o da imagem a 96 dpi, reduzido proporcionalmente para caber na largura do texto; a largura e a altura vêm do cabeçalho do arquivo (PNG, JPEG, GIF, WebP). SVG, que o Word só aceita acompanhado de uma cópia em PNG, e imagem cujo tamanho não se consegue ler viram o texto alternativo entre colchetes, como no TXT; sem texto alternativo, somem.
- **Separador:** um parágrafo vazio com borda inferior.
- **Quebra de página:** `w:br` de página.
- **Metadados:** em `docProps/core.xml`, só os que existem: título, autor (`dc:creator`), assunto, palavras-chave (separadas por `; `), criação e modificação (W3CDTF, UTC). Sem nenhum desses, o pacote não tem `core.xml`.

**PDF:** o modelo vira dados (`sys.inputs`) para um template Typst fixo, e o `typst-pdf` gera o arquivo. O texto do documento nunca é código Typst. Relido pelo leitor de PDF (spec 07), o texto volta na mesma ordem; a estrutura volta até onde esse leitor a deduz (títulos, parágrafos, listas e ênfase).

- **Página e texto:** A4, margens de 2,5 cm; Libertinus Serif 11 pt, justificado e **sem hifenização**: o leitor de PDF mantém o hífen de fim de linha, e um texto hifenizado ganharia hífens no meio das palavras ao ser relido ou convertido de novo. A justificação só estica os espaços, nunca os encolhe: num espaço encolhido, os programas que extraem texto do PDF (`pdftotext`, a busca e a cópia dos leitores de PDF) deixam de ver o espaço e juntam as palavras.
- **Títulos:** em negrito, com um tamanho por nível, todos a partir de 1,16 vez o corpo, para que o leitor de PDF os reconheça como títulos (ele exige 1,15).
- **Ênfase e código inline:** negrito, itálico e DejaVu Sans Mono.
- **Link:** externo, clicável; `#âncora` sai só com o texto, porque o modelo não marca o destino.
- **Lista:** marcadores `•`, `◦` e `▪` por nível (os que o leitor de PDF reconhece) ou números `1.`, com aninhamento recuado.
- **Citação:** bloco recuado.
- **Código:** DejaVu Sans Mono 9 pt, com as quebras de linha do original.
- **Tabela:** grade com bordas, com a primeira linha como cabeçalho repetido em cada página, e o texto das células em 9 pt: sem hifenização, uma palavra longa numa tabela de muitas colunas (`ESPECIFICAÇÃO` numa grade de sete) invadiria a célula vizinha. Uma palavra que nem em 9 pt cabe na coluna ainda invade. Linhas com menos células são completadas com células vazias.
- **Imagem:** no tamanho próprio, reduzida para caber na largura do texto. PNG, JPEG, GIF, WebP e SVG entram; imagem de outro tipo ou cujo tamanho não se consegue ler vira o texto alternativo entre colchetes, como no TXT, e sem texto alternativo some.
- **Separador:** uma linha horizontal da largura do texto.
- **Quebra de página:** começa uma página nova, sem gerar página em branco.
- **Metadados:** título, autor, assunto, palavras-chave e data de criação, só os que existem. Sem data de criação no modelo, o PDF não tem data (o Typst usaria a de hoje).

**Fontes do PDF:** embarcadas no app, nunca as do sistema. Padrão: as fontes livres do `typst-assets` (Libertinus Serif no corpo, DejaVu Sans Mono no código). Georgia, usada na página do protótipo, é licenciada pela Microsoft e não pode ser embarcada.

## As 12 conversões

| Origem ↓ / Destino → | PDF | TXT | MD | DOCX |
|---|---|---|---|---|
| PDF | — | Simples | Difícil | Difícil |
| TXT | Simples | — | Trivial | Simples |
| MD | Média | Simples | — | Média |
| DOCX | Média | Simples | Média | — |

A dificuldade vem quase toda do leitor. Sair de PDF é difícil porque o leitor de PDF deduz a estrutura (spec 07). Gerar PDF deixou de ser o gargalo com o Typst: DOCX → PDF e MD → PDF passam a ser médias.

## Fluxo

1. O usuário escolhe Converter → formato de destino. O dropdown só lista as conversões disponíveis para aquele documento (`Design/specs/05-document-view.md`). Documento inválido (spec 04) não oferece conversão.
2. **Documento protegido por senha** (spec 04): o Folio usa a senha do documento aberto ou pede a senha, e avisa antes de começar: "O documento convertido não terá senha." O usuário confirma ou desiste.
3. A conversão entra na **fila** e roda em segundo plano (ver "Execução").
4. O leitor de origem produz o modelo e o escritor de destino grava o resultado num temporário em `.folio/tmp/` (spec 01).
5. Sem erro, o temporário é publicado **atomicamente, sem sobrescrita**, no destino final.
6. O índice registra o novo documento e a relação `converted_to`, com os `modified_at` dos dois arquivos (spec 04).
7. O derivado abre no Reader (e entra nos recentes, spec 06), e o toast confirma: "{nome do derivado} criado · {nome do original} preservado".
8. O derivado pode ser lido, movido, compartilhado e convertido de novo como qualquer outro.

**Destino:** na 0.1.0, sempre o diretório do original (de um link simbólico, o diretório do link, spec 04), com o nome do original e a extensão nova: `Trabalho.pdf` → `Trabalho.docx`. Se o nome já existir, sufixo automático (`Trabalho (1).docx`, spec 01); nunca sobrescreve. Não há escolha de outro diretório.

**Falha:** o temporário é descartado, o original fica intacto, o usuário recebe a mensagem de erro e nenhuma relação é registrada. Temporários órfãos de uma conversão interrompida (app fechado, queda de energia) são removidos na próxima abertura do workspace (spec 01).

## Execução

- **Em segundo plano:** a conversão roda numa thread do core e nunca trava a interface. O usuário pode continuar lendo e navegando.
- **Fila, uma por vez:** pedir outra conversão enquanto uma roda coloca a nova na fila. Uma de cada vez mantém o uso de memória previsível (o modelo de um documento grande, com imagens, ocupa memória) e simplifica a ordem dos sufixos de colisão.
- **Progresso:** o core informa a etapa (lendo, escrevendo) e, quando dá para medir, a fração: na 0.1.0, só as páginas lidas de um PDF, a única leitura longa. A escrita não tem fração: o Typst compila o PDF de uma vez, e os outros escritores são rápidos. Os eventos da fila (`Queued`, `Started`, `Progress`, `Finished`, `Failed`, `Cancelled`, por id do pedido) saem por um canal que o `src-tauri` repassa à interface. A interface mostra um toast que não some sozinho enquanto a conversão roda: "Convertendo {nome} para {formato}… 40%", com "Cancelar". Itens na fila aparecem como "Na fila".
- **Cancelar:** o core confere o pedido de cancelamento entre as páginas de um PDF lido e logo antes de publicar o derivado. Cancelada, a conversão descarta o temporário, não registra relação e não abre nada; o toast confirma "Conversão cancelada". Um item na fila sai dela sem começar.
- Fechar o app com conversão em andamento a interrompe; o temporário é limpo na próxima abertura (spec 01).

## Imagens

As imagens que o leitor de origem reconhece (spec 07) entram no destino que aceita imagem (MD, DOCX, PDF):

- **Do MD:** as mesmas regras da exibição (spec 07, "Imagens no Markdown"). Imagem local dentro do workspace entra no destino, imagem remota **não é baixada**: o destino recebe o texto alternativo no lugar dela, e o endereço como link quando o formato aceita link.
- **Do DOCX:** as imagens embutidas no próprio arquivo.
- **Do PDF:** as imagens raster que o leitor extrai (spec 07), em PNG; um PDF digitalizado vira um documento só de imagens, sem texto.
- **Para TXT:** imagem vira o texto alternativo entre colchetes (`[Diagrama da arquitetura]`); sem texto alternativo, some.
- **Para MD:** o MD não embute imagem. As imagens são gravadas numa pasta ao lado do derivado, `{nome do derivado}.assets/`, com nomes sequenciais e a extensão do tipo (`Trabalho.assets/imagem-1.png`), e o MD aponta para cada uma por caminho relativo: `![Diagrama](Trabalho.assets/imagem-1.png)`. O nome da pasta é o mesmo em qualquer idioma (convenção do Typora). Sem imagens, não há pasta.

**Publicação com pasta de imagens** (garantia 2 da spec 01):

1. O MD e a pasta nascem juntos num diretório temporário em `.folio/tmp/`.
2. O nome final é escolhido para os dois ao mesmo tempo: o menor sufixo em que **o arquivo e a pasta** estão livres (`Trabalho (1).md` com `Trabalho (1).assets/`). O MD aponta para o nome final da pasta.
3. **A pasta é publicada primeiro, depois o MD**, os dois sem sobrescrita. O MD nunca aparece apontando para uma pasta que não existe.
4. Se a publicação do MD falhar depois da pasta, a pasta volta para `.folio/tmp/` (rename no mesmo filesystem) e é descartada. Nada fica pela metade no workspace.

A pasta é um diretório comum: aparece na árvore e pode ser movida, renomeada ou excluída como qualquer pasta. As imagens dentro dela não são documentos e não aparecem (spec 04).

## Critério mínimo de qualidade

Uma conversão só entra na 0.1.0 se, num corpus de documentos reais por formato de origem:

1. **Nenhum texto se perde nem muda de ordem.** Exceção: PDF digitalizado, que não tem texto.
2. **A estrutura que a origem tem chega ao destino:** títulos continuam títulos, listas continuam listas, ênfase continua ênfase.
3. **O destino abre sem erro nem aviso** no programa de referência do formato (LibreOffice para DOCX, um leitor de PDF, um editor de texto para MD e TXT).

A conversão que não atingir o critério sai da 0.1.0 em vez de atrasar o lançamento, e o dropdown deixa de oferecê-la.

**Como verificar:**

- Os itens 1 e 2 têm teste de integração por conversão (`crates/folio-core/tests/conversions.rs`), com uma amostra por formato de origem: o derivado é relido e comparado com o original, palavra por palavra e em títulos, listas e ênfase (menos no TXT, que não tem marcação).
- No corpus real, `cargo run -p folio-core --example conversion_corpus -- <corpus> <saída>` converte cada documento para os outros três formatos num workspace novo em `<saída>/Folio`, e `scripts/check-derivatives.sh <saída>/Folio <corpus>` abre os derivados nos programas de referência (item 3). Os itens 1 e 2 no corpus são conferidos à mão.

**Resultado (Linux, 2026-11-26):** as 12 conversões ficam na 0.1.0. O corpus foram os documentos dos spikes S2 e S4: 9 PDFs (entre eles um de 190 MB, um digitalizado, um com JPEG2000 e um protegido), 13 DOCX e os 12 PDFs que o LibreOffice gera desses DOCX. Não há MD nem TXT reais no corpus: essas origens ficam cobertas pelos testes de integração.

1. **Abrir sem erro (item 3):** todos os derivados abrem no LibreOffice e no `pdftotext`, sem erro nem aviso. Os documentos protegidos falham com `PasswordRequired`, como esperado (o corpus não passa senha).
2. **Texto e estrutura (itens 1 e 2):** PDF → DOCX, MD e TXT e DOCX → MD e TXT não perdem texto, e a estrutura chega igual; DOCX → PDF só difere pelos limites do leitor de PDF (abaixo). O corpus achou três bugs, corrigidos: tabulação do DOCX juntando palavras (spec 07), espaços encolhidos pela justificação do PDF e palavra longa invadindo a célula vizinha numa tabela do PDF ("Escritores").
3. **Limites do leitor de PDF, não da conversão:** o derivado em PDF tem o texto inteiro, mas, relido, um número digitado no início do parágrafo vira item de lista (spec 07), e uma URL quebrada entre duas linhas volta com um espaço no ponto da quebra, porque o leitor junta as linhas com espaço.

## Erros

Além dos da spec 04 (`InvalidDocument`, `PasswordRequired`, `WrongPassword`) e da 07 (`ReadFailed`):

| Erro | Quando |
|---|---|
| `ConversionUnavailable` | Pedido de uma conversão que não está na 0.1.0 (fora do critério mínimo) |
| `WriteFailed` | O escritor não conseguiu gerar o destino (ex.: erro de compilação do Typst); o motivo técnico vai para o log |
| `ConversionCancelled` | O usuário cancelou; a interface mostra só o toast de cancelamento, sem tom de erro |

Erros de disco (cheio, sem permissão) seguem a spec 01.

## Critérios de aceite

- [x] As 12 conversões que passarem no critério mínimo funcionam pela interface, cada uma com teste de integração no `folio-core`.
- [x] O original tem o mesmo hash antes e depois de qualquer conversão, inclusive das que falham.
- [x] Uma conversão interrompida no meio não deixa arquivo final nem relação no índice.
- [x] Colisão de nome nunca sobrescreve.
- [x] Texto com caracteres especiais do Typst (`#`, `$`, `*`, `_`, `@`, `<`) sai literal no PDF.
- [x] Converter um documento protegido avisa que o derivado não terá senha e só começa depois da confirmação.
- [x] Converter um PDF grande não trava a interface e mostra o progresso.
- [x] Cancelar descarta o temporário: nenhum arquivo final, nenhuma relação, original intacto.
- [x] Duas conversões pedidas em seguida rodam uma depois da outra, sem colisão de nome.
- [x] O derivado nasce no diretório do original e abre no Reader, com o toast de confirmação.
- [x] Imagem remota de um MD não é baixada na conversão; imagem local entra no DOCX e no PDF.
- [x] Converter para MD um documento com imagens grava as imagens em `{nome}.assets/`, o MD as referencia por caminho relativo e o Folio as exibe.
- [x] Se o nome do MD ou da pasta já existir, os dois ganham o mesmo sufixo; uma falha ao publicar o MD não deixa a pasta no workspace.
