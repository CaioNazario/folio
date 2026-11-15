# 01 · Integridade

Nenhuma operação do Folio pode destruir ou corromper um documento do usuário. As garantias desta spec valem para todas as outras: cada spec seguinte cobre, nos próprios testes, as garantias que as operações dela usam.

## O que o Folio faz com arquivos do usuário

A lista é fechada. Qualquer operação nova sobre arquivos do usuário entra primeiro aqui.

| Operação | Usada por | Garantia |
|---|---|---|
| Ler | Reader, metadados, indexação, conversão | Só leitura; nunca abre o arquivo com permissão de escrita |
| Criar arquivo | Conversão, importação (spec 09) | Nasce num temporário e é publicado sem sobrescrita |
| Criar diretório | Projetos e pastas; pasta de imagens da conversão para MD (spec 08) | Falha se o nome já existir; a pasta de imagens nasce no temporário e é publicada sem sobrescrita |
| Renomear e mover | Workspace, Entrada, importação a partir da Entrada | Sem sobrescrita; disco antes do banco |
| Enviar para a lixeira | Workspace, Reader, Entrada | Nunca apaga permanentemente |

O Folio **nunca escreve dentro de um arquivo existente** do usuário: a 0.1.0 não edita documentos. Tudo o que o Folio grava por conta própria fica em `.folio/`.

## Garantias

### 1. Sem sobrescrita silenciosa

Nenhuma operação substitui um arquivo ou diretório existente.

- **Rename e move** usam o rename sem sobrescrita do sistema, que falha de forma atômica se o destino existir: `renameat2` com `RENAME_NOREPLACE` no Linux, via `rustix` (API segura, sem `unsafe`). Nunca "confere se existe e depois renomeia", que tem corrida.
- **Filesystem sem suporte** a `RENAME_NOREPLACE` (alguns FUSE e NFS): a operação falha com erro explicando o motivo, em vez de cair num rename que sobrescreve.
- **Publicar um arquivo novo** (conversão) usa o mesmo rename sem sobrescrita sobre o temporário do `tempfile`. Não usa o `persist_noclobber`, que, sem `RENAME_NOREPLACE`, cai para `link` + `unlink` em vez de falhar com `NoAtomicSupport`.

**Colisão de nome:**

| Situação | Comportamento |
|---|---|
| Conversão, mover documento (inclusive da Entrada) | Sufixo automático: `Trabalho (1).docx`, `Trabalho (2).docx`... O aviso de sucesso mostra o nome final |
| Renomear, criar projeto ou pasta | Erro na hora ("Já existe um item com esse nome"): o usuário escolheu o nome, o Folio não muda por ele |

O sufixo é calculado e tentado em laço: se `(1)` for criado por outro processo entre a escolha e o rename, a falha do rename leva ao `(2)`.

### 2. Arquivo novo nasce num temporário

1. O resultado é gravado num temporário em `.folio/tmp/`.
2. O temporário é sincronizado com o disco (`sync_all`).
3. É publicado no destino final sem sobrescrita (garantia 1), num rename atômico: quem olha o destino vê o arquivo inteiro ou nada.
4. Qualquer erro antes do passo 3 descarta o temporário. O destino nunca recebe um arquivo pela metade.

**Arquivo com pasta de imagens** (conversão para MD, spec 08): a pasta é publicada antes do arquivo, os dois sem sobrescrita; se o arquivo falhar, a pasta volta para `.folio/tmp/` e é descartada.

**Destino em outro filesystem** (um ponto de montagem dentro do workspace): o rename falha com `EXDEV`. O temporário é recriado no diretório de destino, com nome oculto `.folio-tmp-*`, e publicado de lá.

### 3. Mover entre filesystems

Mover da Entrada para um projeto pode cruzar filesystems (Downloads e workspace em discos diferentes), onde o rename não funciona. Nesse caso:

1. Copia para um temporário no diretório de destino e sincroniza.
2. Confere que o hash da cópia é igual ao da origem.
3. Publica sem sobrescrita.
4. Só então remove a origem.

Se qualquer passo falhar, a origem fica intacta e o temporário é descartado. Mover nunca deixa duas cópias nem nenhuma: no pior caso (queda entre 3 e 4), sobram as duas, e nenhuma se perde.

### 4. Exclusão vai para a lixeira

- Toda exclusão (documento, pasta, projeto) envia para a lixeira do sistema pelo crate `trash`, que segue a especificação freedesktop no Linux (inclusive a lixeira por ponto de montagem, `$topdir/.Trash-$uid`).
- Se a lixeira falhar ou não existir, a exclusão **falha com erro**. Nunca há fallback para apagar permanentemente.
- O Folio não tem "esvaziar lixeira" nem "excluir permanentemente".
- Android, que não tem lixeira do sistema, será especificado junto com a versão Android.

### 5. Disco antes do banco

Toda operação de arquivo segue a ordem:

1. Executa no disco.
2. Atualiza o índice.

Se o passo 2 falhar, a operação continua valendo (o disco é a verdade): o erro é registrado no log e o próximo escaneamento corrige o índice (spec 05). O contrário (índice atualizado, disco não) nunca acontece. Se o passo 1 falhar, o índice não é tocado.

### 6. Banco recuperável

- SQLite em modo **WAL**, com `synchronous = NORMAL` e `foreign_keys = ON`.
- **Toda escrita múltipla numa transação.** Uma escrita de várias linhas nunca fica pela metade.
- Na abertura, o banco do workspace passa por `PRAGMA quick_check`. Se o arquivo não abrir, falhar no check ou não for um banco do Folio:
  1. o banco é renomeado para `.folio/folio.db.corrupt` (substituindo um anterior), para não perder o que ainda der para recuperar à mão;
  2. um banco novo é criado e o workspace é reindexado a partir do disco (spec 05);
  3. o usuário é avisado de que favoritos, ícones, recentes e relações de conversão podem ter se perdido. Documentos, nunca.
- Banco ausente é o caso normal de primeira abertura: cria e indexa, sem aviso.

### 7. Temporários limpos

Na abertura do workspace, o Folio esvazia `.folio/tmp/`. Os `.folio-tmp-*` fora de `.folio/` (garantia 2) são removidos quando o escaneamento os encontra (spec 05), e nunca aparecem na interface.

Para a limpeza não apagar o temporário de outra janela, a abertura do workspace pega um lock exclusivo em `.folio/lock` (`File::try_lock` da std). Se o lock já estiver com outra janela, o Folio não abre o mesmo workspace duas vezes: foca a janela que já está com ele.

## Erros

O `folio-core` distingue, num tipo de erro próprio, as falhas que esta spec define, para que a interface explique o motivo em vez de mostrar um erro genérico:

| Erro | Quando |
|---|---|
| `AlreadyExists` | Rename, move ou criação com destino ocupado (onde não há sufixo automático) |
| `NoAtomicSupport` | Filesystem sem `RENAME_NOREPLACE` |
| `TrashUnavailable` | Lixeira do sistema falhou ou não existe |
| `CopyMismatch` | Hash da cópia diferente da origem ao mover entre filesystems |
| `WorkspaceInUse` | Lock de `.folio/lock` com outra janela |

Erros de I/O do sistema (permissão, disco cheio) passam adiante com o caminho relativo do arquivo.

## Testes

As garantias são testadas com diretórios temporários (`tempfile`) e sem estado global. O helper de teste compara o hash de todos os arquivos do workspace antes e depois de cada operação, para afirmar que nada além do esperado mudou.

## Critérios de aceite

- [ ] Rename, move, criação e publicação nunca sobrescrevem: com o destino ocupado, falham ou geram sufixo, conforme a tabela de colisão.
- [ ] Uma operação que falha no meio não deixa arquivo final parcial; o original e o destino ficam como estavam.
- [ ] Mover entre filesystems só remove a origem depois de a cópia ter o mesmo hash e estar publicada.
- [ ] Exclusão manda para a lixeira do sistema; com a lixeira indisponível, falha sem apagar nada.
- [ ] Falha ao atualizar o índice depois de uma operação de arquivo não desfaz a operação, e o próximo escaneamento corrige o índice.
- [ ] Banco corrompido é preservado como `folio.db.corrupt`, recriado e reindexado, sem perder documento.
- [ ] Abrir o workspace esvazia `.folio/tmp/`; uma segunda janela no mesmo workspace não abre e não apaga temporários da primeira.
- [ ] Nenhum código do `folio-core` abre arquivo do usuário com permissão de escrita, fora a publicação de arquivo novo.
