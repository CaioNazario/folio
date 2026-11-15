# 03 · Sidebar

## 3.1 Header (logo)
- Altura 46px fixa, `padding: 0 14px`, flex com `align-items:center; gap:9px`
- `border-bottom: 1px solid --line-1`
- Logo: SVG inline com height 22px, viewBox 164×58, `color: --tx-1` (ver [09-assets](09-assets.md))
- Clique no logo: vai para a **Home** (`04-home.md`). Rótulo acessível "Ir para Documentos"

## 3.2 Ações
Container: `padding: 12px 12px 8px`, coluna, gap 8px.

### Botão "Importar documento"
- Largura 100%, `padding: 8px 10px`, gap 9px, alinhado à esquerda
- Borda 1px `--line-6`, raio 14, fundo `--surf-2`, texto `--tx-1` 13/500
- Hover: fundo `--line-3`, borda `--line-9`
- Ícone: Lucide `plus` 15px, stroke 3, cor `--plus`
- Clique: abre o seletor de arquivos da importação (`specs/09-entrada.md`); até o command existir, fica sem ação. No protótipo ia para a Home, que agora abre pelo logo (3.1)

### Campo de busca
- `padding: 7px 10px`, gap 8px, raio 14, fundo `--surf-1`, borda 1px `--surf-4`
- Ícone Lucide `search` 13px, `--tx-1`
- Input sem borda nem outline, fundo transparente, 13px `--tx-1`, `flex:1; min-width:0`
- Placeholder: "Pesquisar documentos"
- Sem comportamento no protótipo (ver [10-known-issues](10-known-issues.md))

## 3.3 Área rolável
`flex:1; min-height:0; overflow-y:auto; padding: 6px 8px 12px`

### Rótulo de seção
11px / 600, uppercase, letter-spacing 0.08em, `--tx-5`. Padding `10px 8px 6px`. "Recentes" usa `16px 8px 6px`.

### Favoritos
- Rótulo: "Favoritos"
- Vazio: "Fixe uma pasta ou documento com o pin para vê-lo aqui." (`padding: 2px 8px 4px`, 12px, `--tx-10`, line-height 1.45)
- Linha: `padding: 5px 8px`, raio 12, gap 8, cursor pointer, hover fundo `--hover`
  - Ícone 13px `--tx-5`: `folder` para pasta, `file` para arquivo
  - Nome 13px `--tx-1`, uma linha com ellipsis
  - [Botão de pin](06-components.md#botão-de-pin) à direita
- Clique: arquivo abre o documento; pasta expande/recolhe o projeto na árvore
- Ordem: ordem de inserção dos favoritos

### Divisor
1px, `margin: 12px 8px 2px`, `--line-2`

### Projetos
Cabeçalho: flex `space-between`, `padding: 10px 8px 6px`. Rótulo "Projetos" + contagem em Mono 11px `--tx-1` ("4").

**Linha de projeto (pasta)**
- `padding: 6px 8px`, raio 12, gap 8, `--tx-1`, hover fundo `--hover`, `margin-bottom: 1px`
- Chevron Lucide `chevron-right` 12px, em caixa de 12px. Aberto: `rotate(90deg)`, sem transição
- Ícone de pasta 14×14 com dois ícones sobrepostos (`folder` e `folder-open`) em crossfade:
  - Fechado: `folder` com opacidade 1 e scale 1; `folder-open` com opacidade 0 e scale .9
  - Aberto: o inverso
  - Opacidade 200ms ease; transform 260ms cubic-bezier(.32,.72,.34,1)
  - Cor: aberto `--nav-accent`, fechado `--icon` (180ms ease)
- Nome 13px
- Botão de pin (margin-left auto)
- Contagem Mono 10px `--tx-1`
- Clique na linha: abre/fecha

**Linha de arquivo (filho)**
- `padding: 5px 8px 5px 25px`, `margin-left: 7px`, `border-left: 1px solid --line-3` (guia vertical), gap 8
- Hover: fundo `--hover`
- **Selecionado:** overlay absoluto `inset: 0 0 0 -7px` (cobre a guia), raio 12, fundo `--surf-4`, `border-left: 2px solid --nav-accent`. O conteúdo fica acima (`position:relative`)
- Nome 13px `--tx-1`, ellipsis
- Botão de pin
- Clique: abre o documento (view "doc")

**Subpastas e documentos sem projeto** (não estão no protótipo): a raiz lista os projetos e, depois deles, os documentos fora de qualquer projeto, como linhas de arquivo. Uma pasta aberta mostra suas subpastas (mesma linha de projeto) e depois seus documentos, cada nível com a guia vertical e o mesmo recuo. O conteúdo de cada pasta é carregado ao abri-la (`specs/03-workspace.md`).

**Outro workspace dentro deste:** a pasta aparece na árvore e, aberta, mostra "Esta pasta é outro workspace" (12px `--tx-8`) no lugar do conteúdo.

Estado inicial: todas as pastas fechadas. A contagem ao lado de "Projetos" é o número de projetos na raiz.

Vazio (não está no protótipo): workspace sem projetos e sem documentos na raiz mostra "Este workspace ainda não tem documentos.", no estilo do vazio de Favoritos. O texto só aparece depois que a raiz foi listada, para não piscar durante o carregamento.

### Recentes
- Rótulo "Recentes"
- Linha: `padding: 5px 8px`, raio 12, gap 8, hover `--hover`
  - Nome 13px `--tx-1`, ellipsis
  - Botão de pin (margin-left auto)
  - Tempo relativo em Mono 10px `--tx-1` ("há 2 d")
- Sem ícone
- Vazio (não está no protótipo): "Os documentos que você abrir aparecem aqui.", no estilo do vazio de Favoritos, só depois da primeira resposta

## 3.4 Rodapé
`padding: 10px 14px`, `border-top: 1px solid --line-1`, flex, gap 9.

- **Avatar:** 22×22, raio 10, fundo `--line-5`, letra "C" 11/600 #a9b0b8, centralizada
- **Bloco de texto** (line-height 1.25):
  - "~/Documentos" 12px `--tx-1`
  - "1.284 arquivos · 3,2 GB" Mono 10px `--tx-1`
- **Seletor de idioma** (margin-left auto): botão com dropdown, à esquerda do seletor de tema (ver [11-i18n](11-i18n.md))
- **Seletor de tema**:
  - Grupo: flex, gap 2, padding 2, raio 14, fundo `--surf-2`, borda 1px `--line-6`
  - 3 segmentos de 24×20, raio 10, ícone 13px:
    - Lucide `sun`, title "Tema claro"
    - Lucide `moon`, title "Tema escuro"
    - Lucide `monitor`, title "Seguir o sistema"
  - Ativo: fundo `--surf-4`, ícone `--nav-accent`. Inativo: transparente, `--tx-10`
  - Transição: background e color 150ms ease
