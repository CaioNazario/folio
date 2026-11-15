# 05 · Visualização de documento

View padrão (`view = "doc"`). Coluna flex: cabeçalho (56px) + corpo (flex 1).

## 5.1 Cabeçalho
`height: 56px; padding: 0 18px; gap: 14px; border-bottom: 1px solid --line-3`

**Esquerda** (`min-width:0`)
- Breadcrumb: Mono 10.5px `--tx-6`, gap 7. "Faculdade" / "{nome do doc}" (o último em `--tx-4`)
- Linha do título (margin-top 2, gap 9):
  - Nome do documento: 15/600, letter-spacing -0.005em
  - Badge de versão: Mono 10px, `padding: 2px 5px`, raio 8, fundo `--surf-4`, `--tx-1`. Texto "v1", "v2", "v3"… ou "derivado" para conversões

**Direita** (margin-left auto, gap 6, `position:relative`, âncora dos menus)
- **Botão "Converter"**: flex, gap 6, `padding: 6px 12px`, borda 1px `--line-6`, raio 12, fundo `--surf-2`, `--tx-2` 12.5px. Ícone `chevron-down` 13px `--tx-7` depois do texto. Hover: fundo `--line-3`. Clique: abre/fecha o dropdown Converter e fecha o outro menu
- **Botão "Mais"** (title "Mais"): 30×29, borda 1px `--line-6`, raio 12, fundo `--surf-2`, ícone `ellipsis-vertical` 15px `--tx-3`. Hover `--line-3`. Clique: abre/fecha o menu Mais e fecha o Converter

## 5.2 Dropdown "Converter"
Posição: absoluta, `top: 38px; right: 66px`, largura 196, z-index 40. Estilo base em [06-components](06-components.md#dropdown).
- Cabeçalho: "Converter para" (`padding: 7px 9px 6px`, 11px `--tx-8`)
- Itens (`padding: 7px 9px`, raio 12, gap 9, hover `--hover-3`):
  - Chip de extensão: Mono 9px/500, `padding: 2px 4px`, raio 7, fundo `--surf-4`, `--tx-1`
  - Rótulo 13px

| Chip | Rótulo |
|---|---|
| PDF | PDF |
| MD | Markdown |
| TXT | Texto simples |
| PNG | Imagem PNG |
| JPG | Imagem JPG |

- Nota de rodapé: `margin: 5px 9px 4px; padding-top: 7px; border-top: 1px solid --bar`, 11px `--tx-9`, line-height 1.45. Texto: "O original é preservado. O novo arquivo entra na timeline como derivado."
- Clique num item: ver [fluxo de conversão](07-state-interactions.md#conversão)

## 5.3 Menu "Mais"
Posição: absoluta, `top: 38px; right: 0`, largura 186, z-index 40. Itens `padding: 7px 9px`, raio 12, 13px, hover `--hover-3`:
1. Renomear
2. Adicionar tag
3. Abrir pasta no sistema
4. Propriedades (não está no protótipo; conteúdo em `specs/07-reader.md`)
5. Mover para lixeira (cor `--danger`)

Nenhum item tem ação no protótipo.

## 5.4 Palco (stage)
Wrapper: `flex:1; min-width:0; padding: 20px` (0 no fullscreen), coluna, `overflow:hidden`.

Palco: `flex:1`, coluna, borda 1px `--line-4` (nenhuma no fullscreen), raio 16 (0 no fullscreen, transição 200ms), fundo `--surf-1`, `overflow:hidden`.

### Toolbar
`padding: 8px 12px; gap: 10px; border-bottom: 1px solid --line-3`, fundo `--surf-2`.
- Texto em Mono 10px `--tx-5`: "{Preview | Fullscreen}" · "página 1 de 4 · 100%"
- Botão fullscreen (margin-left auto): 26×24, borda 1px `--line-6`, raio 10, fundo `--pop`, `--tx-3`, hover `--hover-3`
  - Ícones `maximize` e `minimize` (13px) sobrepostos em crossfade de 160ms
  - title: "Fullscreen" ou "Sair do fullscreen"

### Área de leitura
`flex:1`, centraliza a página, `padding: 8px` (4px no fullscreen, transição 200ms), `overflow:hidden`.

### Fullscreen
Não é fullscreen do navegador: apenas remove o padding, a borda e o raio do palco. Sidebar e cabeçalho continuam visíveis. Abrir ou fechar também fecha os menus.

## 5.5 Página do documento
A página se ajusta ao palco e toda a tipografia escala com a largura (container queries).

- `container-type: inline-size`, largura 97%, altura 97%, coluna flex
- Fundo `--paper`, texto `--paper-tx`, raio 7, sombra `--shadow-1`, borda 1px `--paper-edge`
- `padding: 3.6cqw 4.6cqw`, Georgia, `overflow:hidden`

| Elemento | Estilo |
|---|---|
| Kicker | Mono 2.1cqw, letter-spacing .1em, uppercase, `--paper-mut`. "Universidade Federal · Sistemas de Informação" |
| Título (h1) | 3.9cqw / 600, line-height 1.25, `margin: 2cqw 0 .8cqw`, text-wrap pretty |
| Autoria | 2.25cqw itálico `--paper-mut2`. "Caio Nazareno · Trabalho final · {data do doc}" |
| Régua | 1px `--paper-rule`, `margin: 2.3cqw 0` |
| Parágrafo | 2.55cqw, line-height 1.62, `margin: 0 0 1.8cqw` |
| Subtítulo de seção | 2.75cqw / 600, line-height 1.4, `margin: 2.6cqw 0 1.1cqw` |

### Encaixe na página (fitPage)
O fluxo de texto (`flex:1; min-height:0; overflow:hidden`) **nunca corta um bloco no meio**: depois de cada render e de cada resize, qualquer bloco cuja base passe da altura do container é escondido (`display:none`). Recalcular em resize da janela, troca de documento, troca de tema e entrada/saída do fullscreen. Na implementação, um ResizeObserver no container é melhor que o resize da janela.
