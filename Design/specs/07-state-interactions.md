# 07 · Estado e interações

## Estado
| Chave | Tipo | Inicial | Descrição |
|---|---|---|---|
| view | 'home' \| 'doc' | 'doc' | View da área principal. No produto começa em `home`: `doc` só com um documento aberto |
| sel | string | 'v3' | Id do item aberto na timeline |
| full | boolean | false | Modo fullscreen do palco |
| convertOpen | boolean | false | Dropdown Converter aberto |
| menuOpen | boolean | false | Menu Mais aberto |
| toast | string \| null | null | Mensagem do toast |
| open | Record<projectKey, boolean> | { fac: true } | Pastas expandidas |
| pinned | Record<nome, 'file' \| 'folder'> | { 'Trabalho v3.docx': 'file', 'Faculdade': 'folder' } | Favoritos, na ordem de inserção |
| hover | string \| null | null | Linha em hover ("rec:", "tree:", "proj:" ou "fav:" + nome) |
| theme | 'light' \| 'dark' \| 'system' | 'dark' | Preferência de tema |
| sysDark | boolean | matchMedia | Resultado de `prefers-color-scheme: dark` |
| tl | TimelineItem[] | 4 itens | Versões e derivados do documento (ver [08-data](08-data.md)) |

**Derivado:** `cur = tl.find(id === sel) ?? último item de tl`.

## Navegação
- **Importar documento** → `view = 'home'` e fecha os menus. No produto: abre o seletor de arquivos (`specs/09-entrada.md`), e quem leva à Home é o **logo** da sidebar
- **Arquivo da árvore ligado à timeline** (Trabalho.docx → v1, Trabalho v2.docx → v2, Trabalho v2.pdf → d1, Trabalho v3.docx → v3) → `sel = id`, `view = 'doc'` e fecha os menus
- **Outros arquivos da árvore** → `sel = nome`. Como não existem na timeline, o leitor mostra o último item. No produto, todo arquivo deve abrir
- **Recentes** (sidebar e home): os dois primeiros abrem v3 e d1. Os outros mostram o toast "{nome} aberto"
- **Favoritos**: arquivo → mesmo comportamento da árvore (ou o toast "{nome} aberto"); pasta → alterna a pasta
- **Card de projeto (home)** → alterna a pasta na sidebar

## Menus
- Converter e Mais são **mutuamente exclusivos**: abrir um fecha o outro
- Os dois fecham ao selecionar um arquivo, ao ir para a Home e ao alternar o fullscreen
- O protótipo **não fecha** com clique fora nem com Esc. Implementar os dois

## Conversão
Ao clicar no formato X do dropdown:
1. `base` = título atual sem extensão; `nome` = `base + '.' + x.toLowerCase()`
2. Cria um TimelineItem `{ id: 'c' + timestamp, kind: 'derived', ext: X, title: nome, sub: 'Convertido de ' + título original, date: 'agora', blocks e docDate herdados }`
3. Insere logo **depois** do item atual em `tl`
4. `sel = novo id`, fecha o dropdown
5. Toast: "{nome} criado · {título original} preservado"
6. O badge de versão mostra "derivado"

## Favoritos
- O pin alterna `pinned[nome]`. A chave é o nome (no produto, usar id estável)
- A lista Favoritos é derivada de `pinned`. Vazia → mostra o texto vazio

## Pastas
- Clique na linha alterna `open[key]`
- Visual: rotação do chevron, crossfade do ícone de pasta e cor do ícone (ver [03-sidebar](03-sidebar.md))

## Tema
- Os 3 segmentos definem `theme`
- Tema resolvido = `theme === 'system' ? (sysDark ? 'dark' : 'light') : theme`
- Escutar `matchMedia('(prefers-color-scheme: dark)')` com 'change' e remover o listener no unmount
- Aplicar `data-theme` na raiz. No light, também `background:#fff` em html/body
- Persistência: não existe no protótipo. Recomendado salvar `theme` localmente

## Fullscreen
Alterna `full`. Afeta o padding do wrapper (20 → 0), a borda (1px → none), o raio (16 → 0), o padding da área de leitura (8 → 4), o rótulo da toolbar (Preview → Fullscreen), o ícone e o title do botão.

## Toast
`flash(msg)`: cancela o timer anterior, mostra a mensagem e esconde após 2600ms. Limpar o timer no unmount.
