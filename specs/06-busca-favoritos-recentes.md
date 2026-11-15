# 06 · Busca, favoritos e recentes

Três formas de chegar a um documento sem navegar pela árvore. As três vivem no banco do workspace e só mostram o que está no disco: documentos ausentes (spec 05) não aparecem em nenhuma.

A aparência está em `Design/specs/03-sidebar.md` (campo de busca, Favoritos, Recentes) e `Design/specs/04-home.md` ("Continuar de onde parou"). A apresentação dos resultados da busca não está no protótipo (`Design/specs/10-known-issues.md`, item 6) e é definida no frontend.

## Busca

**Por nome do arquivo**, no workspace aberto. Não procura no conteúdo (FTS5 fica para depois da 0.1.0) nem em nomes de pastas.

### Normalização

O nome é comparado numa forma normalizada, para que "relatorio" encontre "Relatório Q1.pdf":

1. Nome do arquivo com a extensão (o último componente do caminho; nome que não é UTF-8 passa por `to_string_lossy`, spec 02).
2. Decomposição Unicode (NFD) e remoção das marcas combinantes: `ó` vira `o`, `ç` vira `c`.
3. Minúsculas.

A mesma função normaliza o nome guardado e o texto digitado. Ela vive no core (crate `unicode-normalization`) e é registrada em toda conexão do SQLite como função (`folio_normalize`, que recebe o caminho e normaliza o último componente), antes das migrações, para que a migração e o índice usem exatamente a mesma regra.

- `documents` ganha a coluna `name_norm`, por migração desta spec, preenchida para os documentos existentes com `folio_normalize`.
- Triggers `AFTER INSERT` e `AFTER UPDATE OF path` em `documents` recalculam `name_norm`. Assim toda operação que muda o caminho de um documento (indexar, renomear, mover, reconectar, renomear ou mover a pasta) atualiza `name_norm` na mesma transação, sem que cada operação precise lembrar disso.

### Correspondência

- O texto digitado é normalizado e dividido em palavras por espaço. **Todas as palavras** precisam aparecer no nome, em qualquer ordem e em qualquer posição: "trab v3" encontra "Trabalho v3.docx".
- Texto vazio (ou só espaços) não busca nada.
- `%` e `_` digitados são literais (escapados no `LIKE`).

**Ordem dos resultados:**

1. Nome que **começa** com a primeira palavra
2. Nome em que alguma palavra **começa** com a primeira palavra
3. Os demais

Uma palavra do nome começa depois de qualquer caractere que não é letra nem dígito (espaço, `-`, `_`, `.`, `(`...): em "Notas_trabalho.md", "trab" começa uma palavra.

Dentro de cada grupo: aberto mais recentemente primeiro (os nunca abertos no fim), depois o nome em ordem alfabética normalizada. No máximo 50 resultados.

### Filtros

Combinam entre si e com o texto (o texto pode ser vazio quando há filtro):

| Filtro | Valores | Observações |
|---|---|---|
| Formato | Um ou mais de PDF, DOCX, MD, TXT | |
| Projeto | Um projeto, ou "Sem projeto" | |
| Pasta | Um diretório | Inclui as subpastas: é um intervalo de bytes no caminho (spec 02) |

Documentos inválidos aparecem nos resultados, como na árvore (spec 04).

**Desempenho:** a busca é um `LIKE` sobre `name_norm`, sem índice de texto. Num workspace de 50.000 documentos, cada busca termina em menos de 50 ms, medido pelo exemplo `search_bench` do `folio-core` (como o `scan_bench` da spec 05). O frontend espera o usuário parar de digitar (150 ms) antes de chamar.

**Resultado (SSD, btrfs, 2026-11-15):** medido com `cargo run --release -p folio-core --example search_bench -- <diretório> 50000`, pior de 20 execuções por consulta. O pior caso é uma letra só, que casa com todos os documentos e põe todos no ranking: 22 a 25 ms. Prefixo de palavra fica em ~13 ms, duas palavras em ~10 ms, só filtro de formato em 11 a 14 ms, sem resultado em ~6 ms e texto com filtro de projeto abaixo de 1 ms. A função de ranking lê os argumentos sem copiá-los: com cópia, o pior caso passava de 60 ms.

## Favoritos

Documentos, pastas e projetos podem ser favoritados.

### `favorites`

| Coluna | Observações |
|---|---|
| `id` | UUID, PK |
| `document_id` | FK para `documents` (`ON DELETE CASCADE`), para favorito de documento |
| `path` | `RelativePath` do diretório, para favorito de pasta ou projeto |
| `added_at` | ms UTC |

Exatamente um entre `document_id` e `path` é preenchido (`CHECK`). Cada documento ou diretório aparece no máximo uma vez (índices únicos parciais).

- **Ordem:** a de inserção, como no protótipo. Não há reordenação na 0.1.0.
- **Favoritar de novo** o que já é favorito não muda nada, nem a ordem.
- **Documento:** ligado ao id, sobrevive a renomear e mover, inclusive por fora quando a indexação reconecta (spec 05). Fica oculto enquanto o documento está ausente e sai junto com ele depois de 30 dias.
- **Pasta ou projeto:** ligado ao caminho, como os ícones (spec 03). Renomear ou mover pelo Folio atualiza o caminho; por fora, o favorito fica órfão e é removido no escaneamento da abertura (spec 05).
- A consulta devolve, para cada favorito, o tipo, o nome e o id ou caminho, que é o que a sidebar precisa para abrir o documento ou expandir o diretório.

## Recentes

Documentos abertos no Reader do workspace atual. A lista de workspaces recentes é outra coisa e fica no banco global (spec 02).

### `recent_documents`

| Coluna | Observações |
|---|---|
| `document_id` | PK e FK para `documents` (`ON DELETE CASCADE`) |
| `opened_at` | ms UTC |
| `position` | Onde o usuário parou, opcional (ver abaixo) |

- **O que conta como abrir:** o documento ser exibido no Reader, inclusive o derivado que a conversão abre em seguida. Abrir um documento inválido, ou desistir na senha de um protegido, não conta.
- Abrir de novo atualiza `opened_at`: um documento aparece uma vez só, na posição do acesso mais recente.
- **A lista guarda os 100 mais recentes.** Ao inserir o 101º, o mais antigo sai, na mesma transação.
- A consulta recebe quantos itens quer (7 na sidebar e na Home, no protótipo) e devolve, para cada um, o id, o nome, o projeto, o formato, a validade, o tamanho e o `opened_at`. Tempo relativo e tamanho são formatados no frontend (`Design/specs/11-i18n.md`).
- Documento ausente fica oculto e sai da lista junto com ele depois de 30 dias (spec 05).

### Continuar de onde parou

O Reader guarda onde o usuário parou e, ao reabrir, volta para lá:

| Formato | `position` |
|---|---|
| PDF, DOCX | Número da página |
| MD, TXT | Fração da rolagem (0 a 1) |

- O frontend informa a posição ao sair do documento (trocar de documento, ir para a Home, fechar a janela). O core só guarda.
- O tipo da posição segue o formato: página (a partir de 1) para PDF e DOCX, fração de 0 a 1 para MD e TXT. Posição de outro tipo, ou fração fora do intervalo, é recusada.
- Ao abrir, o core devolve a posição salva junto com o conteúdo. Página que não existe mais (o documento mudou) cai na última página; essa correção depende do número de páginas e é feita com o Reader (spec 07).
- Documento protegido por senha também guarda a posição: o número da página não revela conteúdo.

## Critérios de aceite

- [ ] "relatorio" encontra "Relatório Q1.pdf"; "RELATÓRIO" também.
- [ ] "trab v3" encontra "Trabalho v3.docx"; "v3 trab" também.
- [ ] Um nome que começa com o texto vem antes de um que só o contém.
- [ ] Os filtros de formato, projeto e pasta funcionam sozinhos, combinados e com texto vazio; o de pasta inclui subpastas.
- [ ] Renomear um documento pelo Folio faz a busca encontrá-lo pelo nome novo e não pelo antigo.
- [ ] Busca em 50.000 documentos responde em menos de 50 ms.
- [ ] Favoritos aparecem na ordem de inserção; favoritar de novo não duplica.
- [ ] Favorito de documento sobrevive a renomear e mover, pelo Folio e por fora com reconexão.
- [ ] Favorito de pasta acompanha o renomear pelo Folio e some quando a pasta é renomeada por fora.
- [ ] Abrir um documento o coloca no topo dos recentes, sem duplicar; a lista não passa de 100.
- [ ] Reabrir um PDF volta à página em que o usuário parou.
- [ ] Documentos ausentes não aparecem na busca, nos favoritos nem nos recentes.
