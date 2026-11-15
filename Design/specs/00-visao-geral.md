# 00 · Visão geral (frontend)

## O produto

O Folio é um leitor e gerenciador local de documentos (Tauri 2 + React). A interface tem duas colunas: a **sidebar** à esquerda, com navegação por favoritos, projetos e recentes, e a **área principal** à direita, que mostra uma view por vez: a **Home** ("Documentos") ou a **visualização de documento**, com o leitor. Não há edição: o usuário lê, organiza e converte. O tema é escuro por padrão, com claro e "seguir o sistema".

Estas specs descrevem o protótipo `Design/Folio v5.dc.html` e são a fonte da verdade da **aparência**. O **escopo** da 0.1.0 e o comportamento do backend estão em `specs/` (comece por `specs/00-visao-geral.md`).

## Regras

Toda tela, componente e estilo do Folio segue o que está em `Design/`.

1. **Fidelidade alta.** Cores, tipografia, espaçamentos, raios e transições destas specs são finais: recriar pixel a pixel.
2. **Tudo via tokens.** Cores e superfícies vêm das CSS custom properties de `01-tokens.md`, nunca de hex fixo no componente. O tema é o atributo `data-theme` no elemento raiz; dark é o padrão.
3. **O que estas specs não cobrem segue a mesma aparência.** Telas, estados e componentes novos (erro de conversão, loading, estados vazios, diálogos, Entrada, Propriedades) reutilizam os tokens, os componentes de `06-components.md`, os ícones Lucide, IBM Plex Sans/Mono e os mesmos raios e espaçamentos. Nenhuma linguagem visual nova.
4. **O protótipo define a aparência, não o escopo nem o código.** A lógica do HTML e o `support.js` não são portados, e funcionalidades do protótipo fora da 0.1.0 não são implementadas só por aparecerem nele.
5. **Pendências conhecidas** estão em `10-known-issues.md` e são resolvidas na implementação, seguindo as sugestões de lá.
6. **Android:** a mesma UI em layout responsivo, com a sidebar virando navegação em gaveta ou inferior. A plataforma é detectada via `@tauri-apps/plugin-os`.

## Glossário

| Termo | Significado |
|---|---|
| **Sidebar** | Coluna esquerda: logo, Importar, busca, Favoritos, Projetos, Recentes e rodapé (spec 03) |
| **Área principal** | Coluna direita; renderiza uma view por vez e ancora o toast (spec 02) |
| **View** | O que a área principal mostra: `home` ou `doc` (spec 07) |
| **Home** | Tela "Documentos": ações rápidas, "Continuar de onde parou" e cards de projeto (spec 04) |
| **Visualização de documento** | View `doc`: cabeçalho com breadcrumb, título, Converter e Mais, e o palco (spec 05) |
| **Palco** | Área do leitor, com toolbar (página, zoom, fullscreen) e a página renderizada (spec 05) |
| **Página** | O documento renderizado dentro do palco, com fundo `--paper` (spec 05) |
| **Fullscreen** | Modo do palco sem padding, borda nem raio; sidebar e cabeçalho continuam visíveis (spec 05) |
| **Árvore** | Lista de projetos da sidebar, que expandem e mostram seus arquivos (spec 03) |
| **Botão de pin** | Liga e desliga favorito em qualquer linha da sidebar (spec 06) |
| **Dropdown Converter** | Menu de formatos de destino da conversão (specs 05 e 07) |
| **Menu Mais** | Renomear, Mostrar na pasta, Copiar arquivo, Propriedades, Mover para lixeira (spec 05) |
| **Toast** | Aviso flutuante no centro inferior, que some após 2600ms (spec 06) |
| **Token** | CSS custom property de cor, sombra ou raio, com valores dark e light (spec 01) |
| **Chip de formato** | Etiqueta Mono com a extensão (PDF, MD...) nos menus (spec 05) |
| **Badge** | Etiqueta ao lado do título do documento; no protótipo, "v1/v2/v3" ou "derivado" (spec 05) |

## Mapa de specs

| # | Spec | Cobre |
|---|---|---|
| 01 | [tokens](01-tokens.md) | Cores dark e light, tipografia, raios, sombras, motion, scrollbar |
| 02 | [layout](02-layout.md) | Shell da janela: sidebar e área principal, tema na raiz |
| 03 | [sidebar](03-sidebar.md) | Logo, Importar, busca, Favoritos, Projetos, Recentes, rodapé e seletor de tema |
| 04 | [home](04-home.md) | Tela "Documentos": ações rápidas, continuar de onde parou, projetos |
| 05 | [document-view](05-document-view.md) | Cabeçalho, dropdown Converter, menu Mais, palco, página e fullscreen |
| 06 | [components](06-components.md) | Botões, linha de navegação, botão de pin, card, rótulo, dropdown, toast |
| 07 | [state-interactions](07-state-interactions.md) | Estado da UI, navegação, menus, conversão, favoritos, tema |
| 08 | [data](08-data.md) | Tipos e dados mock do protótipo |
| 09 | [assets](09-assets.md) | Ícones Lucide, logo (SVG) e fontes |
| 10 | [known-issues](10-known-issues.md) | Bugs, lacunas e decisões pendentes do protótipo |
| 11 | [i18n](11-i18n.md) | Português e inglês pelo idioma do sistema, dicionários tipados, formatação, erros do backend |

## Ordem de implementação

Ordem planejada. Cada etapa só consome dados reais quando o command do backend correspondente existir (`specs/`); até lá, usa `mockIPC` nos testes.

1. Base: tokens no `src/app/styles.css` (01), shell da janela (02), fontes e logo (09), dicionários e formatadores de idioma (11)
2. Componentes compartilhados (06): botões, linha de navegação, card, dropdown, toast, botão de pin
3. Sidebar (03): árvore de projetos, Favoritos, Recentes, rodapé e seletor de tema
4. Visualização de documento (05): cabeçalho, palco, toolbar e fullscreen, com o leitor de cada formato
5. Home (04)
6. Interações (07) ligadas ao backend: abrir documento, favoritos, recentes, conversão, tema persistido
7. Pendências de `10-known-issues.md`, resolvidas na spec da tela que cada uma afeta

O `08-data.md` não é implementado: os tipos reais vêm de `src/lib/bindings/` (ts-rs) e os dados, dos commands do Tauri.

## Divergências do protótipo com a 0.1.0

O protótipo mostra coisas que a 0.1.0 não entrega (ver "Fora do escopo da 0.1.0" em `specs/00-visao-geral.md`). A aparência vale; o comportamento abaixo não.

- **Versões e timeline:** o badge "v1/v2/v3", os tokens reservados de diff (`--add-*`, `--del-*`, `--mod-*`) e a nota "entra na timeline como derivado" do dropdown Converter dependem de versionamento, que está fora da 0.1.0. O badge só mostra "derivado".
- **Formatos de conversão:** o dropdown oferece PDF, MD, TXT, PNG e JPG. Na 0.1.0 os formatos são PDF, TXT, MD e DOCX, e o dropdown só lista as conversões disponíveis para o documento aberto.
- **Página do documento:** no protótipo é um layout fixo (kicker, título, autoria, blocos). No produto, o conteúdo vem da engine de cada formato; a moldura (fundo `--paper`, sombra, raio, borda) é a do protótipo.
- **Tags:** "Adicionar tag" do menu Mais fica fora da 0.1.0.
- **Dados mock:** nomes, contagens, tamanhos e o caminho "~/Documentos" do rodapé são ilustrativos.

## Fora de escopo

- Botão Editar e qualquer edição de documento
- Timeline de versões e diff
- Busca no conteúdo dos documentos (a busca da sidebar é só por nome)
- Layout de Android (gaveta ou navegação inferior): vem com a versão Android

## Critérios de aceite globais

- [ ] Cada tela bate com o protótipo nos temas escuro e claro, a 1440×900.
- [ ] Nenhuma cor fora dos tokens: nada de hex em componente nem cor da paleta padrão do Tailwind.
- [ ] Trocar o tema muda a interface inteira sem recarregar, e a escolha persiste entre aberturas.
- [ ] Menus fecham com clique fora e com Esc (`10-known-issues.md` #12).
- [ ] Validado no app (`npm run tauri dev`, WebKitGTK), não só no navegador.
- [ ] `just check` verde.
