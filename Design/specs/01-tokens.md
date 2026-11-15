# 01 · Design tokens

Todos os tokens são CSS custom properties aplicadas no elemento raiz via `data-theme="dark" | "light"`. Dark é o padrão.

## Cores

### Superfícies e linhas
| Token | Dark | Light | Uso |
|---|---|---|---|
| --bg | #000000 | #ffffff | Fundo do app, sidebar e área principal |
| --surf-1 | #0a0a0a | #f3f2ef | Campo de busca, fundo do palco do leitor |
| --surf-2 | #0d0d0d | #f6f5f2 | Botões, cards, linhas da lista da home, toolbar do palco, grupo do tema |
| --surf-4 | #1a1a1a | #eae8e3 | Borda direita da sidebar, borda da busca, fundo do arquivo selecionado, badge de versão, chip de formato, segmento de tema ativo |
| --pop | #141414 | #ffffff | Dropdowns, toast, botão de fullscreen |
| --hover | #101010 | #ecebe7 | Hover das linhas da sidebar |
| --hover-2 | #151515 | #eeece8 | Hover das linhas da lista da home |
| --hover-3 | #1e1e1e | #ecebe7 | Hover de itens de menu e do botão de fullscreen |
| --line-1 | #121212 | #e6e4df | Borda inferior do header da sidebar, borda superior do rodapé |
| --line-2 | #161616 | #e8e6e1 | Divisor entre Favoritos e Projetos |
| --line-3 | #1c1c1c | #e4e2dd | Guia da árvore, bordas do cabeçalho do doc e da toolbar, divisores da lista, hover de botões |
| --line-4 | #212121 | #dedcd6 | Bordas de cards, da lista e do palco |
| --line-5 | #242424 | #dedcd6 | Fundo do avatar |
| --line-6 | #282828 | #d6d4ce | Borda de botões, do grupo de tema e do botão fullscreen |
| --line-9 | #3a3a3a | #b7b4ac | Borda em hover (botões e cards) |
| --bar | #2e2e2e | #cbc8c1 | Borda de dropdowns e toast, divisor interno de dropdown, thumb da scrollbar |
| --bar-hover | #454545 | #a9a59d | Hover do thumb da scrollbar |
| --avatar-tx | #a9b0b8 | #5f636a | Letra do avatar |

### Texto
| Token | Dark | Light | Uso |
|---|---|---|---|
| --tx-1 | #ffffff | #16171a | Texto primário (nomes, títulos, contagens) |
| --tx-2 | #e6e8ea | #1b1c20 | Cor base do app, texto do botão "Converter" |
| --tx-3 | #c3c8ce | #3a3d43 | Ícones de botões (⋮, fullscreen) |
| --tx-4 | #b9bfc6 | #44474d | Item atual do breadcrumb |
| --tx-5 | #afafaf | #67696e | Rótulos de seção da sidebar, ícones de favoritos, texto da toolbar do palco |
| --tx-6 | #9aa1a9 | #67696e | Breadcrumb |
| --tx-7 | #8b9299 | #6d7075 | Subtítulo da home, chevron do "Converter" |
| --tx-8 | #7d848c | #74777c | Descrições, metadados, "Converter para", pin inativo |
| --tx-9 | #6b727b | #7f8288 | Rótulos de seção da home, nota de rodapé do dropdown |
| --tx-10 | #5f666f | #8a8d92 | Texto vazio de favoritos, contagens mono da home, segmento de tema inativo |

### Accent e semânticas
| Token | Dark | Light | Uso |
|---|---|---|---|
| --accent | #ffffff | #16171a | Links, hover do pin |
| --accent-2 | #e6e6e6 | #000000 | Hover de links |
| --nav-accent | #ffffff | #16171a | Barra do arquivo selecionado, pasta aberta, pin ativo, ícone do tema ativo |
| --plus | #ffffff | #16171a | Ícone "+" do botão Importar |
| --icon | #787f88 | #7a7d83 | Ícones de cards, pasta fechada |
| --teal | #6fb3a8 | #2c7a6e | Ponto do toast (sucesso) |
| --danger | #c98b7f | #a33b2c | "Mover para lixeira" |

### Página (papel)
| Token | Dark | Light | Uso |
|---|---|---|---|
| --paper | #efece5 | #ffffff | Fundo da página |
| --paper-tx | #22242a | #22242a | Texto da página |
| --paper-mut | #8a857a | #8a857a | Kicker (instituição) |
| --paper-mut2 | #6c6960 | #6c6960 | Linha de autoria |
| --paper-rule | #d8d3c8 | #d8d3c8 | Régua sob o cabeçalho |
| --paper-edge | transparent | #e4e2dc | Borda da página (só no light) |

### Reservados (definidos, não usados na v5)
`--bg-rail`, `--surf-3`, `--line-7`, `--line-8`, `--sel`, `--tx-0`, `--tx-11`, `--ring`, `--icon-2`, `--teal-line`, `--acc-line`, `--add-bg/--add-tx`, `--del-bg/--del-tx`, `--mod-bg/--mod-tx`. Foram feitos para timeline de versões e diff. Os valores estão no bloco `<style>` do protótipo.

### Fora dos tokens (hardcoded)
- Html/body no light: `#ffffff`
- Hover do thumb da scrollbar e letra do avatar viraram os tokens `--bar-hover` e `--avatar-tx`, com valor no light (`10-known-issues.md`, item 3)

## Sombras
| Token | Dark | Light |
|---|---|---|
| --shadow-1 (página) | 0 10px 30px rgba(0,0,0,.45) | 0 6px 20px rgba(30,28,22,.10) |
| --shadow-2 (dropdown, toast) | 0 14px 34px rgba(0,0,0,.5) | 0 14px 34px rgba(30,28,22,.14) |

## Tipografia
- **IBM Plex Sans** 400/500/600: UI
- **IBM Plex Mono** 400/500: contagens, datas, breadcrumb, badges, toolbar
- **Georgia**, 'Times New Roman', serif: corpo da página do documento
- **Source Serif 4** 600: só o wordmark "Folio" (não carregado no protótipo, cai para Georgia; carregar ou converter o logo em path)
- Base: 14px, `-webkit-font-smoothing: antialiased`

| Estilo | Fonte | Tamanho/peso | Extras |
|---|---|---|---|
| Título da home | Sans | 26 / 600 | letter-spacing -0.01em |
| Título do documento | Sans | 15 / 600 | letter-spacing -0.005em |
| Corpo / subtítulo | Sans | 14 / 400 | |
| Linhas de nav, menus, títulos de card | Sans | 13 / 400–500 | |
| Botão Converter, toast | Sans | 12.5 / 400 | |
| Descrições, meta, caminho do rodapé | Sans | 12 / 400 | |
| Rótulo de seção | Sans | 11 / 600 | uppercase, letter-spacing 0.08em |
| Contagem e data (home) | Mono | 11 / 400 | |
| Breadcrumb | Mono | 10.5 / 400 | |
| Contagens, datas e badge (sidebar) | Mono | 10 / 400 | |
| Chip de formato | Mono | 9 / 500 | |

## Raios
| Valor | Uso |
|---|---|
| 7px | Página, chip de formato |
| 8px | Botão de pin, badge de versão |
| 10px | Avatar, segmento de tema, botão de fullscreen |
| 12px | Linhas da sidebar, botões do cabeçalho, itens de menu |
| 14px | Botão Importar, busca, grupo do tema |
| 16px | Palco do leitor |
| 18px | Cards, lista da home, dropdowns, toast |
| 50% | Ponto do toast |

## Espaçamento
Sem escala formal. Valores recorrentes: 2, 5, 6, 7, 8, 9, 10, 12, 14, 18, 20, 32, 36, 40, 72. Gaps internos de linha: 8–9px. Padding de cards: 14px.

## Motion
| Duração | Easing | Onde |
|---|---|---|
| 150ms | ease | Opacidade/cor do pin, fundo/cor do segmento de tema |
| 160ms | ease | Crossfade dos ícones maximizar/minimizar |
| 180ms | ease | Cor do ícone da pasta |
| 200ms | ease | Opacidade dos ícones de pasta, raio do palco, padding da área de leitura |
| 260ms | cubic-bezier(.32,.72,.34,1) | Escala dos ícones de pasta (0.9 ↔ 1) |
| 0ms | — | Rotação do chevron (sem transição) |
| 2600ms | — | Auto-dismiss do toast |

## Scrollbar
11px, thumb `--bar` com raio 14 e borda transparente de 3px (`background-clip: content-box`), hover do thumb `--bar-hover`, track transparente. Firefox: `scrollbar-width: thin; scrollbar-color: var(--bar) transparent`.

## Links
`a { color: var(--accent); text-decoration: none }` · `a:hover { color: var(--accent-2) }`
