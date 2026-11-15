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

**Resultado:** pendente.

## Arquitetura

Toda conversão é um leitor (spec 07) seguido de um escritor, ligados pelo modelo de documento:

```text
origem ─▶ leitor do formato de origem ─▶ modelo de documento ─▶ escritor do formato de destino ─▶ destino
```

Com 4 leitores e 4 escritores, as 12 conversões saem das combinações. Melhorar um leitor melhora todas as conversões que saem daquele formato. É a mesma arquitetura do Pandoc, reproduzida em Rust com bibliotecas embarcáveis.

A conversão reconstrói o documento: preserva texto, ordem e estrutura (títulos, listas, ênfase, tabelas, imagens, conforme o leitor de origem reconhece), mas não a aparência exata da origem (posições, fontes, colunas).

## Escritores

| Formato | Base | O que é nosso |
|---|---|---|
| TXT | — | Imprimir o texto: títulos e parágrafos separados por linha em branco, itens de lista com `- ` ou `1. ` |
| MD | `comrak` (impressão de CommonMark a partir da árvore) | Montar a árvore do comrak a partir do modelo; metadados viram front matter |
| DOCX | `docx-rs` | Traduzir o modelo em parágrafos com os estilos do Word ("Heading 1"…, "Normal"), listas numeradas e com marcadores, tabelas e imagens; metadados vão para `docProps/core.xml` |
| PDF | `typst` (biblioteca, via `typst-as-lib`) + `typst-pdf` | Gerar o texto-fonte do Typst a partir do modelo e um template de página (margens, fontes, tamanhos). Todo texto do documento entra escapado: nunca é interpretado como código Typst |

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

1. O usuário escolhe Converter → formato de destino. O dropdown só lista as conversões disponíveis para aquele documento (`Design/specs/05-document-view.md`).
2. O leitor de origem produz o modelo e o escritor de destino grava o resultado num **arquivo temporário dentro do workspace**.
3. Sem erro, o temporário é renomeado **atomicamente** para o destino final, por padrão ao lado do original (`Trabalho.pdf` → `Trabalho.docx`).
4. Se o nome já existir, o Folio pergunta ou usa um sufixo (`Trabalho (1).docx`); nunca sobrescreve em silêncio.
5. O índice registra o novo documento e a relação `converted_to` (origem, destino, formatos, data).
6. O derivado aparece no Folio e pode ser lido, movido, compartilhado e convertido de novo como qualquer outro.

**Falha:** o temporário é descartado, o original fica intacto, o usuário recebe a mensagem de erro e nenhuma relação é registrada. Temporários órfãos de uma conversão interrompida são removidos na próxima abertura do workspace (spec 01).

## Critério mínimo de qualidade

Uma conversão só entra na 0.1.0 se, num corpus de documentos reais por formato de origem:

1. **Nenhum texto se perde nem muda de ordem.** Exceção: PDF digitalizado, que não tem texto.
2. **A estrutura que a origem tem chega ao destino:** títulos continuam títulos, listas continuam listas, ênfase continua ênfase.
3. **O destino abre sem erro nem aviso** no programa de referência do formato (LibreOffice para DOCX, um leitor de PDF, um editor de texto para MD e TXT).

A conversão que não atingir o critério sai da 0.1.0 em vez de atrasar o lançamento, e o dropdown deixa de oferecê-la.

## Critérios de aceite

- [ ] As 12 conversões que passarem no critério mínimo funcionam pela interface, cada uma com teste de integração no `folio-core`.
- [ ] O original tem o mesmo hash antes e depois de qualquer conversão, inclusive das que falham.
- [ ] Uma conversão interrompida no meio não deixa arquivo final nem relação no índice.
- [ ] Colisão de nome nunca sobrescreve.
- [ ] Texto com caracteres especiais do Typst (`#`, `$`, `*`, `_`, `@`, `<`) sai literal no PDF.
