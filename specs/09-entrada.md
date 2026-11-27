# 09 · Entrada e importação

Duas maneiras de um documento entrar no workspace:

- **Entrada:** a visão dos documentos da pasta Downloads, com atualização ao vivo, de onde o usuário **move** o que quer guardar.
- **Importação:** trazer documentos de qualquer lugar do disco, pelo seletor de arquivos ou arrastando para a janela, **copiando** para o workspace.

Nenhuma das duas está desenhada no protótipo, exceto os botões de importar (`Design/specs/10-known-issues.md`, item 5). A interface segue a regra do que as specs não cobrem: tokens, componentes e ícones existentes, nenhuma linguagem visual nova.

## Entrada

A Entrada mostra os documentos da pasta Downloads sem transformá-la em workspace: o Folio não cria `.folio/` lá e não guarda nada dela no banco do workspace.

### Pasta da Entrada

- **Padrão:** `XDG_DOWNLOAD_DIR` (lido de `~/.config/user-dirs.dirs` pelo `download_dir` do Tauri, como a pasta de documentos da spec 03), com fallback `~/Downloads`.
- Guardada no banco global, na preferência `inbox.location` (spec 02): é o único caminho absoluto fora da lista de workspaces, porque pertence à máquina, não ao workspace.
- O usuário pode trocar a pasta pelo cabeçalho da Entrada ("Trocar pasta").
- **Pasta que não existe:** a Entrada mostra "Pasta não encontrada: `<caminho>`", com "Escolher pasta".
- **Pasta dentro do workspace aberto** (ou a própria raiz dele, comparando o caminho por componentes): a Entrada some da sidebar. Aqueles documentos já estão no workspace, e mostrá-los duas vezes confundiria.

### O que aparece

- **Só o primeiro nível** da pasta, sem entrar em subpastas: Downloads costuma ter pastas extraídas de ZIPs, com centenas de arquivos que não foram baixados como documento.
- **Só documentos**, com as regras da spec 04: os quatro formatos, documentos inválidos com aviso, protegidos por senha, links simbólicos para arquivo; ocultos e locks não aparecem.
- **Downloads em andamento não aparecem.** Os navegadores baixam para um nome temporário e renomeiam no fim; o Folio esconde:
  - arquivos com extensão de download em andamento: `.part` (Firefox), `.crdownload` (Chrome, Chromium, Brave, Edge), `.download` (Safari), `.partial`, `.opdownload` (Opera);
  - o arquivo `X` enquanto existir ao lado um `X.part` ou `X.crdownload`: o Firefox cria o arquivo final vazio antes de terminar, e ele apareceria como documento inválido.
- **Ordem:** o mais recente primeiro (`modified_at`), porque o que acabou de baixar é o que o usuário procura.

### Atualização ao vivo

- Um watcher do filesystem (crate `notify`, com o debouncer `notify-debouncer-full`) observa **só a pasta da Entrada**, sem recursão, enquanto o app está aberto.
- A cada lote de eventos (debounce de 500 ms), o core relista a pasta e avisa a interface por evento se algo mudou. Relistar um diretório é barato e evita interpretar evento por evento.
- O download que termina aparece sozinho: o rename final do navegador dispara o watcher.
- Ao abrir o app, a Entrada é listada do zero; não há estado guardado entre execuções.

Dentro do workspace não há watcher (spec 05): a Entrada é a única pasta observada em tempo real.

### Identidade

Documentos da Entrada não estão em nenhum índice e não têm id estável. A interface se refere a eles pelo nome do arquivo, que é único dentro da pasta. Antes de qualquer ação, o core confere que o nome é um nome de arquivo simples (sem `/`, sem ser `.` ou `..`, o que tiraria a ação da pasta; senão, `InvalidName`) e que o arquivo ainda existe; se não existe, devolve `InboxDocumentNotFound` e relista.

### Ações

| Ação | Comportamento |
|---|---|
| Abrir | Abre no Reader (spec 07), como pré-visualização: não entra nos recentes, não guarda posição, não tem favorito nem conversão. O Reader recebe a origem do documento: um documento do workspace, pelo id, ou um arquivo da Entrada, pelo nome |
| **Mover para…** | Escolhe um projeto ou pasta do workspace e **move** o arquivo (nunca copia, para não existir duas versões do mesmo documento). Usa a operação de mover da spec 04: sem sobrescrita, sufixo em colisão; entre filesystems diferentes, cópia conferida por hash antes de remover a origem (spec 01). O documento entra no índice com id novo, e o toast confirma "Movido para Faculdade/TCC" |
| Mostrar na pasta | Spec 10 |
| Mover para lixeira | Spec 01 |

O Folio não altera, renomeia nem converte arquivos na Entrada: para converter, o documento precisa estar no workspace.

### Interface

- **Sidebar:** seção "Entrada" entre Favoritos e Projetos, uma linha de navegação (`Design/specs/06-components.md`) com o ícone Lucide `inbox` e a contagem em Mono, no estilo da contagem de Projetos. Clique abre a Entrada na área principal.
- **Área principal:** uma view nova, com cabeçalho ("Entrada", o caminho da pasta em Mono e "Trocar pasta") e a lista no estilo de "Continuar de onde parou" da Home (`Design/specs/04-home.md`): nome, chip de formato, tamanho e tempo relativo. Cada linha tem "Mover para…" e o menu Mais com as outras ações.
- **Vazia:** "Nenhum documento novo em Downloads."
- **Mover para…:** um dropdown com os projetos e as pastas do workspace, com o último destino usado no topo.
- **Último destino:** guardado no banco do workspace, na tabela `last_destination` (uma linha só: o `RelativePath` da pasta), e atualizado a cada "Mover para…" e a cada importação. Segue a pasta quando o Folio a renomeia ou move; um destino que não existe mais não aparece no topo.

### Android

Fora da 0.1.0. Lá o acesso à Downloads passa por MediaStore/SAF e será especificado junto com a versão Android.

## Importação

Para documentos que estão em qualquer lugar do disco: um pendrive, outra pasta, o anexo salvo por outro app.

- **Pelo seletor:** "Importar documento" (sidebar) e o card "Importar arquivos" (Home) abrem o seletor de arquivos do sistema (`tauri-plugin-dialog`), com seleção múltipla e filtro pelos quatro formatos.
- **Arrastando:** soltar arquivos do gerenciador de arquivos na janela do Folio importa do mesmo jeito (evento de drag-and-drop do Tauri, que entrega os caminhos).
- **Destino:** depois da seleção, o mesmo dropdown do "Mover para…", com o último destino usado no topo. Soltar os arquivos sobre um projeto ou pasta da árvore já define o destino.
- **Copia, não move:** a origem pode ser um pendrive ou a pasta de outro app, e o Folio não mexe em arquivo fora do workspace sem o usuário pedir. A cópia segue a garantia 2 da spec 01: temporário, sincronização, publicação sem sobrescrita, sufixo em colisão.
- **Arquivos que não são documento** na seleção são ignorados, e o toast avisa: "2 arquivos ignorados: o Folio não abre esse formato".
- **Arquivo que já está no workspace:** não é copiado; o toast avisa "Este documento já está no workspace" e o Folio o seleciona na árvore.
- **Arquivo que está na pasta da Entrada:** é movido, como no "Mover para…" da Entrada, para não existir duas versões.
- Ao terminar, os documentos importados entram no índice com ids novos; com um só, ele abre no Reader.

## Erros

| Erro | Quando |
|---|---|
| `InboxNotFound` | A pasta da Entrada não existe |
| `InboxDocumentNotFound` | O arquivo da Entrada sumiu antes da ação (carrega o nome do arquivo: ele não tem id) |
| `AlreadyInWorkspace` | Importar um arquivo que já está no workspace |

Mais os das specs 01 e 04 (`AlreadyExists`, `CopyMismatch`, `DocumentNotFound`...).

## Critérios de aceite

- [ ] A Entrada lista os documentos do primeiro nível da pasta de downloads, do mais recente para o mais antigo, sem criar `.folio/` nem gravar nada lá.
- [ ] Um download que termina aparece na Entrada sem ação do usuário.
- [ ] Durante um download no Firefox e no Chrome, nem o arquivo temporário nem o arquivo final vazio aparecem.
- [ ] "Mover para…" tira o arquivo da Downloads e o coloca no destino, inclusive entre filesystems, sem sobrescrever.
- [ ] Abrir pela Entrada não cria recente nem posição.
- [ ] A Entrada some quando a pasta de downloads está dentro do workspace aberto.
- [ ] Importar pelo seletor e arrastando copia para o destino; a origem fica intacta.
- [ ] Importar ignora arquivos que não são documento e avisa quantos.
- [ ] Importar um arquivo da pasta da Entrada move, em vez de copiar.
