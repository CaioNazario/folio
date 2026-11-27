# 02 · Persistência

O Folio usa dois bancos SQLite com papéis diferentes. Nenhum deles guarda documentos: se os dois sumirem, nenhum documento se perde (spec 01).

Esta spec define a infraestrutura dos bancos (onde ficam, como abrem, como migram, como guardam caminhos) e o banco global inteiro. As tabelas do banco do workspace são definidas pela spec dona de cada uma.

## Spikes

Resolver antes de implementar a parte que cada spike bloqueia. O resultado vai para a seção da spec que ele decide e para o "Resultado" do spike.

### S7 · Workspace em pasta sincronizada

**Bloqueia:** nada. Adiado (ver "Resultado").

**Pergunta:** o que acontece quando o workspace está numa pasta sincronizada (Dropbox, Google Drive, Syncthing, Nextcloud), onde o SQLite pode corromper? Detectar e avisar, mover o banco para fora, ou bloquear?

**Como testar:** um workspace dentro de uma pasta do Syncthing e do Nextcloud (os que rodam no Linux), com duas máquinas. Abrir no Folio, favoritar e converter em uma, sincronizar e abrir na outra. Observar:

1. Se o sincronizador copia `folio.db` com o `-wal` e o `-shm` pela metade ou fora de ordem
2. Se o `quick_check` da abertura detecta o problema (spec 01) e a recuperação funciona
3. Se dá para detectar a pasta sincronizada (marcadores como `.stfolder`, `.sync`, caminhos conhecidos)

**Resultado:** adiado, sem data. Decisão provisória: o banco do workspace fica em `<workspace>/.folio/folio.db`, como descrito abaixo. Uma corrupção causada pelo sincronizador é coberta pela recuperação da spec 01 (`quick_check` falha, o banco vira `folio.db.corrupt` e o workspace é reindexado): no pior caso perdem-se favoritos, ícones, recentes e relações, nunca documentos. Se o spike mostrar que o banco precisa sair do workspace, a mudança fica isolada na implementação do `WorkspaceStore`.

O `WorkspaceStore` e a tabela `meta` são implementados no início da spec 03, junto com a abertura do workspace, que define o formato da trait.

## Os dois bancos

| Banco | Arquivo | Guarda | Se sumir |
|---|---|---|---|
| Global | `<dados do app>/folio.db` | Workspaces conhecidos e recentes, preferências, caminho da Entrada | Volta o diálogo da primeira execução; perde a lista de workspaces e as preferências |
| Do workspace | `<workspace>/.folio/folio.db` | Ids, metadados, índice de busca, ícones, favoritos, recentes, relações de conversão | Reindexa a partir do disco; perde favoritos, ícones, recentes e relações |

**Diretório de dados do app:** fixo, com o nome do app, sem o identificador do bundle:

| Plataforma | Diretório |
|---|---|
| Linux | `$XDG_DATA_HOME/folio/`, com fallback `~/.local/share/folio/` |
| Windows | `%APPDATA%\Folio\` |
| Android | Storage interno do app (`app_data_dir` do Tauri) |

No desktop, o `src-tauri` parte do diretório de dados do usuário (`data_dir` do Tauri) e acrescenta `folio`/`Folio`; não usa o `app_data_dir`, que deriva do identificador `com.caionazario.folio`. O `src-tauri` resolve o caminho e o passa ao `folio-core`, que não conhece o Tauri.

**Por que o banco do workspace fica dentro dele:** copiar ou mover a pasta do workspace leva junto favoritos, ícones e relações, e o futuro repo git mora no mesmo `.folio/`.

## Abertura

Os dois bancos abrem do mesmo jeito:

1. Abre o arquivo com `rusqlite` (feature `bundled`: não depende do SQLite do sistema e já traz FTS5 para o futuro).
2. Aplica os pragmas: `journal_mode = WAL`, `synchronous = NORMAL`, `foreign_keys = ON`, `busy_timeout = 5000`.
3. Confere a identidade (abaixo) e roda o `quick_check` (spec 01).
4. Aplica as migrações pendentes.

**Identidade:** todo banco do Folio grava `PRAGMA application_id` com o valor `0x464F4C49` ("FOLI"). Um arquivo `folio.db` sem esse id não é um banco do Folio e segue a recuperação da spec 01 (vira `folio.db.corrupt`, nunca é sobrescrito). Um banco recém-criado, vazio, recebe o id.

**Conexão:** uma conexão por banco, atrás de um `Mutex`. Os commands do Tauri compartilham essa conexão; na 0.1.0 não há pool. Se a medição mostrar contenção, o pool entra depois, sem mudar a interface do core.

**Recuperação do banco global:** a mesma do workspace (spec 01): arquivo corrompido vira `folio.db.corrupt` e um banco novo é criado. Sem reindexação, porque o banco global não é derivado do disco.

## Migrações

- A versão do esquema fica em `PRAGMA user_version`.
- As migrações são SQL embutido no binário, numeradas e aplicadas em ordem, cada uma na própria transação, junto com o novo `user_version`.
- Uma migração nunca é editada depois de publicada numa versão: correção vira migração nova.
- **Banco de versão mais nova** (`user_version` maior que a última migração conhecida): foi aberto por um Folio mais novo. O Folio não mexe nele e não abre o workspace, com a mensagem "Este workspace foi aberto por uma versão mais nova do Folio. Atualize o app para abri-lo." Não é corrupção: o banco não vira `.corrupt`.
- Cada spec que cria uma tabela traz a migração dela.

## Tabelas

**Banco global (definidas aqui):**

| Tabela | Colunas | Observações |
|---|---|---|
| `workspaces` | `id` (UUID), `location` (`Location`), `added_at`, `opened_at` | `location` única. Os recentes são os de `opened_at` mais novo. O último aberto é o que abre ao iniciar o app |
| `preferences` | `key` (texto, PK), `value` (JSON) | Ex.: `inbox.location`, `theme`, `language`. Chave desconhecida é ignorada, não é erro. `language`: `"system"` (padrão), `"pt-BR"` ou `"en"`; `theme`: `"system"`, `"dark"` (padrão, como no design) ou `"light"`; `inbox.location` (spec 09): os bytes exatos do caminho, como lista de números, para que um nome que não é UTF-8 sobreviva. Um valor gravado que não dá para ler vale como o padrão. Lidas pelo command `get_preferences` e gravadas por `set_language` e `set_theme` |

**Banco do workspace (cada uma definida pela spec dona):**

| Tabela | Spec dona |
|---|---|
| `documents`, `document_metadata`, `conversions` | 04 (documentos) |
| `icons` | 03 (workspace) |
| `favorites`, `recent_documents` | 06 (busca, favoritos e recentes) |
| `last_destination` | 09 (entrada e importação) |
| `meta` (id do workspace, data de criação) | 02 (esta) |

Não há tabela de hierarquia de pastas: a hierarquia vem do disco.

**Convenções de todas as tabelas:**

- Ids são UUID v7 em texto, gerados por um `IdGenerator` injetado (ids previsíveis nos testes).
- Datas são `INTEGER` em milissegundos UTC desde a época Unix, vindas do `Clock` injetado.
- Nomes de tabelas e colunas em inglês, em `snake_case`.

## Caminhos

### No banco do workspace: `RelativePath`

Todo caminho no banco do workspace é relativo à raiz (`Faculdade/TCC/Trabalho.pdf`), num tipo próprio do core que só existe validado:

- Nunca absoluto, nunca vazio, nunca com `.` ou `..`.
- Separador sempre `/`, em qualquer plataforma; a conversão para o separador do sistema acontece só na borda, ao tocar o disco.
- Nunca dentro de `.folio/`.
- Guardado como os bytes exatos do disco, sem normalização Unicode: o nome é o que o disco tem.

**Nem todo nome é UTF-8.** O Linux aceita nomes de arquivo com bytes inválidos em UTF-8 (arquivos vindos de sistemas antigos, ZIPs em Latin-1, pendrives). Esses arquivos são indexados e exibidos como qualquer outro:

- **Armazenamento:** a coluna de caminho é `BLOB` com os bytes do caminho. Quando o nome é UTF-8 válido (o caso comum), os bytes são o próprio UTF-8, iguais em qualquer plataforma. Quando não é, são os bytes crus do Unix (`OsStrExt::as_bytes`), nunca o `as_encoded_bytes`, cuja codificação a std não garante entre versões. A forma no Windows (nomes UTF-16 malformados) será especificada com a versão Windows.
- **Busca por pasta:** como o separador é o byte `/`, "tudo dentro de `Faculdade/TCC/`" continua sendo um intervalo de bytes no índice.
- **Exibição:** o core entrega à interface o nome convertido com `to_string_lossy`: os bytes inválidos aparecem como `�`. O nome exibido nunca é usado para achar o arquivo: a interface fala com o core pelo id do documento (spec 04), e o core usa os bytes guardados.
- **Na ponte com a interface:** um `RelativePath` atravessa como a lista dos seus bytes (`number[]` no TypeScript), sempre ao lado do nome exibido. Pastas e projetos, que não têm id, voltam da interface pelos mesmos bytes, então um diretório com nome que não é UTF-8 também pode ser aberto, favoritado ou renomeado. Ao voltar, os bytes passam pelas mesmas validações do `RelativePath`.
- **Nome novo** (renomear, criar pasta, sufixo de colisão) vem sempre de texto digitado ou gerado, portanto é UTF-8.

### No banco global: `Location`

A referência absoluta a um diretório (workspace ou Downloads) é um tipo que aceita caminho (desktop) ou URI persistida (Android, SAF), nunca uma coluna de texto que assume `/home/...`. Na 0.1.0 só a variante de caminho é implementada, guardada em bytes como o `RelativePath`; a de URI chega com a versão Android.

Caminhos absolutos só existem no banco global.

## `WorkspaceStore`

O local do estado do workspace fica atrás de uma trait no core, para que a diferença entre plataformas seja detalhe de implementação:

- **`.folio/` no workspace** (Linux e Windows): o banco em `<workspace>/.folio/folio.db`, o temporário em `.folio/tmp/`, o lock em `.folio/lock`.
- **Storage interno** (futuro, Android com pasta externa via SAF): o banco no storage interno do app, indexado pelo id do workspace, porque o SQLite precisa de um caminho de arquivo real. Especificado com a versão Android.

Na 0.1.0 a única implementação é a de `.folio/` (`FolioDirStore`), testada em diretório temporário; as operações só de banco são testadas com SQLite em memória.

## Critérios de aceite

- [x] Os dois bancos abrem em WAL, com `foreign_keys` ligado e `application_id` do Folio.
- [x] Um `folio.db` que não é do Folio não é sobrescrito: vira `folio.db.corrupt`.
- [x] Migrações aplicam em ordem, cada uma numa transação; uma migração que falha deixa o banco na versão anterior.
- [x] Banco de versão mais nova não é alterado e o workspace não abre, com a mensagem de atualizar o app.
- [x] `RelativePath` rejeita caminho absoluto, vazio, com `..`, com `.` e dentro de `.folio/`, e guarda sempre com `/`.
- [x] Nenhuma coluna do banco do workspace guarda caminho absoluto.
- [ ] Um documento com nome que não é UTF-8 é indexado, aparece na interface com `�` no lugar dos bytes inválidos e abre, renomeia, move e vai para a lixeira como os outros.
- [x] O banco global fica em `~/.local/share/folio/` no Linux (ou `$XDG_DATA_HOME/folio/`).
- [x] Apagar os dois bancos não perde nenhum documento.
