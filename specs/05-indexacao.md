# 05 · Indexação

A indexação mantém o banco do workspace de acordo com o disco: descobre documentos novos, percebe os que mudaram ou sumiram e reconecta ao id os que foram movidos por fora do Folio. O disco é sempre a verdade; o índice só alcança o disco.

Na 0.1.0 não há monitoramento em tempo real dentro do workspace: mudanças externas são percebidas na abertura do workspace e ao navegar.

## Spikes

Resolver antes de implementar a parte que cada spike bloqueia. O resultado vai para a seção da spec que ele decide e para o "Resultado" do spike.

### S5 · Reconectar ao id um arquivo movido por fora

**Bloqueia:** a reconexão (abaixo).

**Pergunta:** a estratégia em duas etapas da seção "Reconexão" (identidade do arquivo no filesystem, depois hash entre candidatos do mesmo tamanho) reconecta os casos reais sem reconectar errado?

**Como testar:** testes de integração em diretório temporário, um por caso, verificando se o id se mantém:

1. `mv` no terminal, no mesmo diretório e entre diretórios
2. Mover e renomear no gerenciador de arquivos (Nautilus, Thunar)
3. Mover entre filesystems (copia e apaga: muda o inode)
4. Salvar por cima num editor que grava num temporário e renomeia (LibreOffice, editores de texto): muda o inode e o conteúdo, mantém o caminho
5. Apagar e criar outro arquivo, que pode reaproveitar o inode
6. Dois arquivos idênticos (mesmo hash) e um deles some
7. Enviar para a lixeira e restaurar

**Resultado:** a estratégia em duas etapas reconecta todos os casos, cobertos por testes de integração em `crates/folio-core/src/indexing/reconnect.rs` (2026-11-15):

| Caso | Como o teste reproduz | Resultado |
|---|---|---|
| 1. `mv` no mesmo diretório e entre diretórios | `rename` | Etapa 1 reconecta, inclusive quando o destino é reconciliado antes da origem |
| 2. Gerenciador de arquivos | O Nautilus e o Thunar movem com o mesmo `rename` do `mv` | Coberto pelo caso 1 |
| 3. Mover entre filesystems | Copiar e apagar no mesmo disco: inode e mtime novos | Etapa 2 reconecta pelo hash |
| 4. Editor que grava num temporário e renomeia | Temporário renomeado por cima do mesmo caminho | Mesmo caminho: o documento é atualizado e mantém o id, sem precisar reconectar |
| 5. Inode reaproveitado | Não dá para forçar o sistema a reaproveitar um inode; a regra da etapa 1 é testada como função pura | Inode igual com tamanho ou mtime diferente não é o mesmo arquivo |
| 6. Dois idênticos e um some | Dois ausentes com o mesmo hash; e um ausente só | Dois: nenhum é reconectado. Um: reconecta |
| 7. Lixeira e restauração | Arquivo movido para fora e de volta ao mesmo caminho | Volta com o mesmo id |

A etapa 2 só reconecta um ausente que já tinha hash calculado antes de sumir: o arquivo dele não existe mais, então o hash não pode ser calculado depois.

### S6 · Indexação rápida de workspace grande

**Bloqueia:** o escaneamento completo (abaixo).

**Pergunta:** quanto tempo levam o percurso, a checagem de assinatura e o hash num workspace grande, e a interface fica responsiva durante a primeira indexação?

**Como testar:** workspaces gerados com 1.000, 10.000 e 50.000 documentos (e o mesmo tanto de arquivos que não são documentos), em SSD e em HD externo. Medir separadamente:

1. Percurso dos diretórios com `stat` de cada entrada
2. Checagem de assinatura (abre cada documento e lê o começo; no DOCX, o diretório central do ZIP)
3. Hash BLAKE3 de todos os documentos
4. Escrita no banco em lotes de 100, 500 e 1.000 por transação
5. Tempo até a árvore aparecer na primeira abertura

**Decide:** o tamanho do lote, se a checagem de assinatura vai para segundo plano junto com o hash, e se o percurso usa paralelismo (`jwalk` ou similar).

**Resultado (SSD, btrfs, 2026-11-15):** medido com `cargo run --release -p folio-core --example scan_bench -- <diretório> 1000 10000 50000`. Cada workspace tem N documentos (PDF, DOCX, MD e TXT em partes iguais) e N arquivos que não são documentos, 100 por pasta.

| Etapa | 1.000 | 10.000 | 50.000 |
|---|---|---|---|
| Percurso com `stat` (só std) | 4 ms | 56 ms | 0,19 s |
| Checagem de assinatura | 21 ms | 0,27 s | 0,9 s |
| Primeiro escaneamento (índice vazio) | 70 ms | 1,0 s | 4,6 s |
| Segundo escaneamento (sem mudanças) | 17 ms | 0,29 s | 2,2 s |
| Hash de todos os documentos | 44 ms | 1,1 s | 8,3 s |

- **Lote:** uma transação por diretório. Gravar cada documento na própria transação custava 50% a mais; não há ganho em ajustar o tamanho do lote à parte.
- **Assinatura:** fica no escaneamento. Ela é uma fração pequena dele; só o hash vai para segundo plano.
- **Paralelismo:** não precisa. O percurso de 100.000 entradas leva 0,2 s.
- **A medição achou dois custos quadráticos, corrigidos:** a etapa 2 da reconexão lia todos os documentos do mesmo tamanho para cada arquivo novo (o primeiro escaneamento de 50.000 levava 129 s); agora só consulta documentos com hash, por um índice parcial. E as consultas por arquivo passaram a reaproveitar o SQL compilado.
- **Pendente:** a mesma medição num HD externo.

## Reconciliar um diretório

A operação básica da indexação é **reconciliar um diretório**: comparar o que o disco tem nele com o que o índice tem para ele, e aplicar a diferença.

1. Lista o diretório no disco, aplicando as regras do que não aparece (specs 03 e 04).
2. Para cada documento no disco:
   - **Já indexado, mesmo `size` e `modified_at`:** nada a fazer. A assinatura não é relida.
   - **Já indexado, `size` ou `modified_at` diferentes:** atualiza os dois e checa a assinatura de novo (`valid`, `protected`). O hash e o cache de metadados passam a valer como velhos, porque `hash_of` e `extracted_from` não batem mais (spec 04).
   - **Não indexado:** candidato a documento novo (ver "Reconexão").
3. Para cada documento indexado naquele diretório que não está mais no disco: candidato a ausente (ver "Reconexão").
4. Grava a diferença numa transação.

O escaneamento completo é a reconciliação de todos os diretórios do workspace, da raiz para baixo.

**Idempotente:** reconciliar duas vezes seguidas não muda nada na segunda. Como o disco é a verdade, um índice atrasado por qualquer motivo (falha depois de uma operação, spec 01; operação do usuário no meio de um escaneamento) se corrige na próxima reconciliação.

## Quando a indexação roda

| Momento | O que reconcilia |
|---|---|
| Abertura do workspace | Escaneamento completo, em segundo plano |
| Banco novo ou recriado (primeira abertura, banco corrompido, spec 01) | Escaneamento completo: a reindexação é só isso, com o índice vazio |
| Expandir um projeto ou pasta na árvore | Aquele diretório |
| Abrir um documento que sumiu do disco | O diretório dele, e o documento vira ausente; a interface mostra `DocumentNotFound` (spec 04). Se o diretório também sumiu, ele conta como vazio: os documentos dele viram ausentes |
| Janela volta ao foco | Os diretórios expandidos na árvore |
| Operação do próprio Folio (renomear, mover, converter, excluir) | Nada: a operação já atualiza o índice (spec 01) |

## Escaneamento completo

- **Em segundo plano**, numa thread própria; a interface nunca espera por ele. O core expõe o escaneamento como função bloqueante, e o `src-tauri` a roda na própria thread.
- A árvore aparece na hora a partir do índice que já existe. Na primeira abertura, com o índice vazio, os documentos aparecem conforme são encontrados.
- O core informa o progresso (diretórios e documentos vistos) a cada diretório, por um callback; o `src-tauri` o liga a um canal e repassa à interface como evento. O core não conhece o Tauri.
- Gravação em lotes: uma transação por diretório (S6).
- **Não entra em** diretórios ocultos, links simbólicos para diretório e workspaces aninhados, que viram pasta bloqueada (spec 03).
- **Remove os `.folio-tmp-*`** que encontrar (spec 01). Só o escaneamento da abertura remove: nesse momento nenhuma conversão está em andamento.
- **Remove ligações órfãs de diretório:** ícones (e favoritos de pasta, spec 06) cujo caminho não existe mais.

## Hash

O hash (BLAKE3, spec 04) serve só à reconexão. Ler todos os arquivos na abertura seria o passo mais caro da indexação, então:

- É calculado **em segundo plano, depois do escaneamento**, com prioridade baixa, para todo documento cujo `hash_of` não bate com `size` e `modified_at` atuais.
- Para um arquivo novo, também é calculado **na hora** quando há um ausente do mesmo tamanho, para tentar a reconexão.
- Documento inválido também tem hash: um download truncado que depois é completado continua sendo o mesmo documento.

## Reconexão

Quando um arquivo é renomeado ou movido por fora, o escaneamento vê um ausente (o caminho antigo) e um novo (o caminho novo). Antes de criar um id novo, o Folio tenta reconhecer que é o mesmo documento, para que favoritos, recentes e relações de conversão não se percam.

**Etapa 1, identidade no filesystem:** dispositivo e inode iguais, e mesmo `size` e `modified_at`. Cobre `mv` e o gerenciador de arquivos, que preservam os três. Exigir tamanho e data evita confundir um inode reaproveitado por outro arquivo.

**Etapa 2, conteúdo:** entre os ausentes com o mesmo `size` do novo, o hash igual. Cobre mover entre filesystems e cópias seguidas de exclusão. Se mais de um ausente tiver o mesmo hash (arquivos idênticos), não reconecta nenhum: na dúvida, o novo ganha id novo.

**Reconectar** atualiza o caminho e mantém o id, com tudo o que está ligado a ele. Um arquivo novo sem correspondência ganha id novo (spec 04).

A coluna de identidade no filesystem (`fs_id`: dispositivo e inode) entra em `documents` por migração desta spec. Ela não vale entre máquinas: num workspace copiado para outro computador, só a etapa 2 reconecta.

## Ausentes

Um documento que sumiu do disco não sai do índice na hora: fica **ausente** por 30 dias.

- `documents` tem `missing_since` (ms UTC, opcional), criada na migração da spec 04, que já precisava dela para a lixeira.
- **Ausente não aparece:** não está na árvore, na busca, nos favoritos, nos recentes nem nas contagens. As relações de conversão dele também ficam ocultas.
- **Volta se reaparecer:** qualquer reconciliação que reconectar o arquivo (no mesmo caminho ou em outro) limpa `missing_since`, e favoritos, recentes e relações voltam junto.
- **Sai do índice depois de 30 dias**, na abertura do workspace, com as relações, o favorito e o recente dele (`ON DELETE CASCADE`). É uma consulta (`DELETE ... WHERE missing_since < agora - 30 dias`, com o `Clock` injetado), não uma tarefa agendada: o SQLite não tem agendador e o Folio não roda processo em segundo plano. O prazo não precisa ser exato, porque ausente já não aparece na interface. Os arquivos nunca são afetados.
- **Excluir pelo Folio** (spec 01) marca o documento como ausente, como se tivesse sumido por fora. Restaurar da lixeira em até 30 dias devolve o documento com o mesmo id e com tudo o que estava ligado a ele.

Os 30 dias cobrem os casos em que o arquivo some e volta: lixeira restaurada, disco desmontado, projeto movido para fora do workspace e trazido de volta.

## Critérios de aceite

- [ ] Abrir um workspace com documentos que não estão no índice indexa todos, em segundo plano, sem travar a interface.
- [ ] Apagar o `folio.db` e reabrir reindexa o workspace inteiro a partir do disco.
- [ ] Reconciliar duas vezes seguidas não muda nada na segunda.
- [ ] Um documento editado por fora tem `size`, `modified_at` e validade atualizados, e hash e metadados marcados como velhos.
- [ ] Renomear ou mover por fora com `mv` preserva o id, os favoritos e as relações.
- [ ] Mover por fora entre filesystems preserva o id pelo hash.
- [ ] Dois ausentes com o mesmo hash não são reconectados a um arquivo novo.
- [ ] Excluir pelo Folio e restaurar da lixeira em até 30 dias devolve o mesmo id, com favorito e relações.
- [ ] Um ausente há mais de 30 dias sai do índice na abertura do workspace.
- [ ] Ausentes não aparecem na árvore, na busca, nos favoritos, nos recentes nem nas contagens.
- [ ] O escaneamento não entra em diretórios ocultos, links simbólicos para diretório nem workspaces aninhados.
