# 04 · Documentos

Um documento é um arquivo do workspace que o Folio reconhece como documento. Esta spec cobre o reconhecimento, o id estável, o que o índice guarda sobre cada documento, as operações sobre documentos e os documentos derivados.

A extração dos metadados de cada formato é feita pelos leitores (spec 07). Esta spec define onde eles ficam; até os leitores existirem, o índice tem só os dados do filesystem.

## Formatos da 0.1.0

| Formato | Extensões (sem diferenciar maiúsculas) | Assinatura exigida |
|---|---|---|
| PDF | `.pdf` | `%PDF-` nos primeiros 1024 bytes (onde os leitores de PDF aceitam o cabeçalho) |
| DOCX | `.docx` | ZIP (`PK\x03\x04`) que contém `word/document.xml`; ou, se protegido por senha, o contêiner criptografado do Office (CFB, `D0 CF 11 E0`, com os streams `EncryptionInfo` e `EncryptedPackage`) |
| MD | `.md`, `.markdown` | Texto: nenhum byte nulo nos primeiros 8 KiB |
| TXT | `.txt` | Texto: começa com a marca de UTF-16 (`FF FE` ou `FE FF`), ou não tem byte nulo nos primeiros 8 KiB |

Qualquer outra extensão não é documento: o arquivo fica no disco e não aparece na interface. Isso inclui `.doc`, `.xlsx`, `.pptx` e imagens, que ficam para depois da 0.1.0.

## Reconhecimento

A extensão decide o formato; a assinatura confirma.

| Extensão | Assinatura | Resultado |
|---|---|---|
| De documento | Confere | **Documento válido** |
| De documento | Não confere (um `.pdf` que é HTML, um download truncado, uma imagem renomeada para `.md`) | **Documento inválido:** aparece na interface com um aviso ("Este arquivo não é um PDF válido") e não abre. Não é escondido: o usuário precisa saber que ele existe para resolver |
| Outra | — | Não é documento |

Um documento protegido por senha é **válido**: ver "Documentos protegidos por senha".

**Também não aparecem:**

- Arquivos ocultos (nome começando com `.`), o que já cobre os temporários do Folio (`.folio-tmp-*`, spec 01) e os locks do LibreOffice (`.~lock.*#`).
- Locks do Word (nome começando com `~$`).

**Links simbólicos para arquivo aparecem.** Um atalho como `Pessoal/Boleto.pdf → ~/Downloads/Boleto.pdf` é um documento do workspace:

- Ler, reconhecer, calcular o hash e converter **seguem o atalho** até o arquivo real, que pode estar fora do workspace. Tamanho e datas são os do arquivo real.
- Renomear, mover e enviar para a lixeira **agem no atalho**. O arquivo real nunca é tocado.
- O derivado de uma conversão nasce ao lado do atalho, dentro do workspace, nunca ao lado do arquivo real.
- **Atalho quebrado** (o arquivo real sumiu) aparece como documento inválido.
- **Atalho com alvo relativo** (`../Downloads/Boleto.pdf`) só pode ser renomeado no mesmo diretório. Movê-lo para outro diretório o quebraria, e o Folio recusa (`RelativeSymlink`).
- Links simbólicos para diretório continuam não sendo seguidos (spec 03).

**Nome que não é UTF-8:** o formato vem da extensão, que é lida dos bytes; um nome inválido em UTF-8 com extensão `.pdf` é um PDF como qualquer outro (spec 02).

## Id estável

- Gerado quando o documento entra no índice pela primeira vez: UUID v7, pelo `IdGenerator` (spec 02).
- **Sobrevive a renomear e mover pelo Folio:** a operação atualiza só o caminho.
- Mover ou renomear **por fora** do Folio é tratado pela indexação (spec 05), que tenta reconectar o arquivo ao id pelo hash.
- A interface só conhece o id. Todo command sobre um documento recebe o id, e o core resolve o caminho.

## O que o índice guarda

### `documents`

| Coluna | Tipo | Origem |
|---|---|---|
| `id` | UUID, PK | Folio |
| `path` | `RelativePath`, único | Disco |
| `format` | `pdf`, `docx`, `md`, `txt` | Extensão |
| `valid` | booleano | Assinatura |
| `protected` | booleano | Assinatura (DOCX) ou dicionário `/Encrypt` com senha de abertura (PDF). O do PDF depende do leitor da spec 07; até ele existir, PDFs ficam como não protegidos |
| `size` | bytes | Filesystem |
| `modified_at` | ms UTC | Filesystem (mtime) |
| `created_at` | ms UTC, opcional | Filesystem (birth time, via `statx`; nem todo filesystem tem) |
| `hash` | BLAKE3, opcional | Calculado; a spec 05 define quando |
| `hash_of` | `size` + `modified_at` do momento do cálculo, em duas colunas (`hash_size`, `hash_modified_at`) | Para saber se o hash ficou velho |
| `missing_since` | ms UTC, opcional | Quando o documento saiu do disco (lixeira ou sumiço). A spec 05 define o que acontece com ele; enquanto está marcado, não aparece na interface nem nas contagens |

**Derivados, não guardados:**

- Nome e extensão: o último componente do caminho.
- MIME type: do formato (`application/pdf`, `application/vnd.openxmlformats-officedocument.wordprocessingml.document`, `text/markdown`, `text/plain`).
- Projeto: o primeiro componente do caminho, quando há mais de um. Sem projeto: o documento está na raiz.
- Pasta: os componentes entre o projeto e o nome.

**Quando o documento foi aberto pela última vez** é dado da tabela de recentes (spec 06), não de `documents`.

### `document_metadata`

Cache dos metadados do formato, preenchido pelos leitores (spec 07). Uma linha por documento, apagada junto com ele.

| Coluna | Observações |
|---|---|
| `document_id` | PK e FK para `documents` |
| `title`, `author`, `subject` | Texto, opcionais |
| `keywords` | Lista em JSON (palavras-chave do PDF e do DOCX, tags do front matter do MD) |
| `doc_created_at`, `doc_modified_at` | Datas internas do documento, opcionais (diferentes das do filesystem) |
| `pages` | Opcional: exato no PDF, aproximado no DOCX (spec 07), ausente em MD e TXT |
| `extra` | JSON com o que é próprio do formato: criador, produtor e versão do PDF; comentários e revisão do DOCX; os demais campos do front matter do MD |
| `extracted_from` | `size` + `modified_at` do arquivo na extração, em duas colunas (`extracted_size`, `extracted_modified_at`) |

O cache vale enquanto `extracted_from` bate com o arquivo; se não bate, é recalculado na próxima vez que alguém pedir os metadados. Apagar a tabela inteira não perde nada: tudo volta dos arquivos.

### `conversions`

| Coluna | Observações |
|---|---|
| `id` | UUID, PK |
| `source_id`, `target_id` | FK para `documents`, com `ON DELETE CASCADE` |
| `source_format`, `target_format` | Formatos no momento da conversão |
| `converted_at` | ms UTC |
| `source_modified_at`, `target_modified_at` | `modified_at` de cada arquivo logo depois da conversão, para os avisos de desatualizado |

A spec 08 grava a relação; esta spec define a tabela e a consulta.

## Documentos derivados

Um derivado é o resultado de uma conversão (spec 08): um arquivo comum, salvo por padrão ao lado do original, que pode ser lido, movido, compartilhado e convertido de novo como qualquer outro.

```text
Trabalho.pdf ──converted_to──▶ Trabalho.docx
             (PDF → DOCX, 2026-11-04)
```

- O core responde, pelo id de um documento, de qual documento ele veio (formato e data) e quais derivados ele tem.
- A relação acompanha os ids: renomear ou mover qualquer um dos dois pelo Folio não a quebra.
- Um original pode ter vários derivados (converter de novo gera outro arquivo e outra relação). Cada relação liga só origem e destino diretos: o core não monta a cadeia PDF → DOCX → MD.
- **Avisos de desatualizado:** o core compara o `modified_at` atual de cada arquivo com o guardado na relação e informa à interface:
  - "O original mudou depois da conversão": o derivado pode estar desatualizado.
  - "O derivado foi modificado depois da conversão": ele não é mais o resultado exato da conversão.

  A relação continua valendo nos dois casos. Os avisos são calculados na consulta, sem tarefa em segundo plano, com o `modified_at` lido do disco naquela hora: editar um dos arquivos produz o aviso sem esperar um novo escaneamento.
- **A relação fica oculta enquanto um dos lados está ausente** (excluído, ou sumiu do disco) **e some quando ele sai do índice**, 30 dias depois (spec 05). Os arquivos nunca são afetados.
- Se o banco for apagado, as relações se perdem e os dois arquivos continuam intactos (spec 01).

## Documentos protegidos por senha

PDF e DOCX podem ter senha de abertura. O Folio os exibe e converte, pedindo a senha.

| Formato | Como é reconhecido | Como abre |
|---|---|---|
| PDF | Assinatura `%PDF-` normal; o leitor encontra o `/Encrypt` com senha de abertura | O pdf.js pede a senha pelo callback `onPassword`; o leitor do core (spec 07) recebe a mesma senha |
| DOCX | Contêiner criptografado do Office no lugar do ZIP | O core decifra com o crate `office-crypto` (MIT, Rust puro) e entrega ao `docx-preview` e ao leitor os bytes do DOCX decifrado |

- **A senha nunca é guardada:** nem no banco, nem em arquivo. Ela vive só na memória enquanto o documento está aberto; fechou, reabrir pede de novo.
- **O conteúdo decifrado nunca vai para o disco.** A decifração acontece em memória.
- **Nada do conteúdo vai para o índice:** documento protegido não tem metadados extraídos em `document_metadata` (título, autor e páginas ficariam legíveis no banco). O índice guarda só o que vem do filesystem.
- Senha errada: o diálogo avisa e pede de novo (`WrongPassword`).
- **Converter** pede a senha (ou usa a do documento aberto) e avisa antes de começar: "O documento convertido não terá senha." O derivado nasce sem proteção.
- PDF com senha só de permissões (impressão, cópia), sem senha de abertura, abre direto, sem pedir nada: não é marcado como protegido.

## Operações em documentos

Todas seguem a spec 01: no disco primeiro, sem sobrescrita.

| Operação | Observações |
|---|---|
| Renomear | Edita só o nome, sem a extensão: a extensão é preservada, para um renomear não mudar o formato. Validação de nome da spec 03. Nome já existente é erro |
| Mover | Para outro projeto, pasta ou para a raiz. Nome já existente no destino gera sufixo (`Trabalho (1).pdf`, spec 01). Usado também pela Entrada (spec 09) |
| Enviar para a lixeira | Spec 01. O documento sai da interface e fica ausente no índice por 30 dias (spec 05): restaurado da lixeira nesse prazo, volta com o mesmo id, favorito e relações |

## Contagens

O core responde, a partir do índice:

- Quantidade de documentos de cada projeto, contando as pastas dentro dele (cards da Home e árvore da sidebar).
- Quantidade e tamanho total dos documentos do workspace (rodapé da sidebar).

Documentos inválidos entram nas contagens: eles aparecem na interface.

## Erros

| Erro | Quando |
|---|---|
| `RelativeSymlink` | Mover para outro diretório um link simbólico com alvo relativo |
| `DocumentNotFound` | O id não está no índice, ou o arquivo sumiu do disco desde o último escaneamento |
| `InvalidDocument` | Pedido para abrir ou converter um documento inválido |
| `PasswordRequired` | Pedido para ler um documento protegido sem senha |
| `WrongPassword` | A senha não abre o documento |

## Critérios de aceite

- [x] Os quatro formatos são reconhecidos pela extensão, sem diferenciar maiúsculas, e confirmados pela assinatura.
- [x] Um `.pdf` com conteúdo HTML aparece como documento inválido, com aviso, e não abre.
- [x] Um `.md` com byte nulo no início aparece como documento inválido.
- [ ] Um DOCX e um PDF protegidos por senha aparecem como válidos, abrem com a senha certa e recusam a errada.
- [x] A senha e o conteúdo decifrado nunca são gravados em disco nem no banco, e documento protegido não tem metadados extraídos no índice.
- [ ] Converter um documento protegido avisa que o derivado não terá senha.
- [x] Arquivos de outras extensões, ocultos e locks do Word não aparecem.
- [ ] Um link simbólico para arquivo aparece como documento; renomear, mover e enviar para a lixeira agem no link e o arquivo real fica intacto; link quebrado aparece como inválido.
- [x] Editar o original ou o derivado depois da conversão produz o aviso de desatualizado correspondente, sem desfazer a relação.
- [x] Renomear e mover pelo Folio preservam o id, os favoritos e as relações de conversão.
- [x] Renomear preserva a extensão.
- [x] Mover para um destino com o mesmo nome gera sufixo e nunca sobrescreve.
- [x] Apagar `document_metadata` e recalcular produz os mesmos metadados.
- [x] Excluir um documento esconde as relações em que ele aparece, sem tocar no outro arquivo.
