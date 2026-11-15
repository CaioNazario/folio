# 11 · Spikes

Pontos em aberto que precisam de experimento antes da feature que depende deles. Cada spike se resolve **antes** de implementar essa feature, e o resultado vai para a spec dela e para a seção "Resultado" do spike.

| # | Spike | Bloqueia | Status |
|---|---|---|---|
| S1 | Leitor de PDF: PDFium ou hayro | Leitor de PDF (05), conversões a partir de PDF (06), metadados de PDF (02) | Aberto |
| S2 | pdf.js no WebKitGTK e no WebView do Android | Exibição de PDF (05) | Aberto |
| S3 | Typst embarcado | Escritor de PDF (06) | Aberto |
| S4 | Fidelidade do docx-preview | Exibição de DOCX (05) | Aberto |
| S5 | Reconectar ao id um arquivo movido por fora | Indexação (04) | Aberto |
| S6 | Indexação rápida de workspace grande | Indexação (04) | Aberto |
| S7 | Workspace em pasta sincronizada | Persistência (03) | Aberto |
| S8 | "Copiar arquivo" no Wayland/Hyprland | Compartilhamento (09) | Aberto |

## S1 · Leitor de PDF: PDFium ou hayro

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

## S2 · pdf.js no WebKitGTK e no WebView do Android

**Pergunta:** o `pdfjs-dist` atual roda no WebKitGTK do Linux e no WebView do Android, com o worker carregado dentro do Tauri?

**Como testar:** abrir os PDFs do corpus do S1 no app (`npm run tauri dev`), com o worker do pdf.js empacotado pelo Vite. Conferir renderização, seleção de texto, zoom e uso de memória num PDF de 300 páginas. Anotar a versão do WebKitGTK testada e a CSP necessária.

**Resultado:** pendente.

## S3 · Typst embarcado

**Pergunta:** quanto o Typst custa no app, e ele gera um PDF bom a partir do modelo de documento?

**Como testar:** escritor mínimo modelo → Typst → PDF com `typst-as-lib` e `typst-pdf`, com as fontes do `typst-assets`. Medir:

1. Tamanho do binário do app antes e depois
2. Tempo de compilação limpa e incremental do workspace
3. Tempo para converter um MD de ~50 páginas
4. Escape: texto com `#`, `$`, `*`, `_`, `@` e `<` sai literal
5. Compila para Android (`aarch64-linux-android`)

**Resultado:** pendente.

## S4 · Fidelidade do docx-preview

**Pergunta:** o `docx-preview` exibe DOCX reais com fidelidade de layout razoável, e como contar páginas?

**Como testar:** corpus de ~10 DOCX reais (trabalho acadêmico com sumário, contrato, documento com tabelas e imagens, documento do Google Docs exportado). Comparar com o LibreOffice: texto, estilos, tabelas, imagens, cabeçalho e rodapé. Verificar o que a opção de quebra de página produz como contagem de páginas.

**Resultado:** pendente.

## S5 · Reconectar ao id um arquivo movido por fora

**Pergunta:** como reconhecer que um arquivo renomeado ou movido fora do Folio é o mesmo documento: hash do conteúdo, inode, ou os dois?

**Resultado:** pendente.

## S6 · Indexação rápida de workspace grande

**Pergunta:** como indexar um workspace com milhares de arquivos na primeira abertura sem travar a interface (escaneamento incremental, hash sob demanda, paralelismo)?

**Resultado:** pendente.

## S7 · Workspace em pasta sincronizada

**Pergunta:** o que acontece quando o workspace está numa pasta sincronizada (Dropbox, Google Drive), onde o SQLite pode corromper? Detectar e avisar, mover o banco para fora, ou bloquear?

**Resultado:** pendente.

## S8 · "Copiar arquivo" no Wayland/Hyprland

**Pergunta:** colocar o arquivo no clipboard como `text/uri-list` funciona de forma confiável no Wayland/Hyprland, a ponto de colar no gerenciador de arquivos e em apps como o WhatsApp Web?

**Resultado:** pendente. Se não funcionar, "Copiar arquivo" sai da 0.1.0 no Linux.
