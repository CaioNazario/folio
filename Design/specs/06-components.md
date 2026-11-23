# 06 · Componentes compartilhados

## Botão secundário
Borda 1px `--line-6`, fundo `--surf-2`, fonte herdada. Hover: fundo `--line-3` (no botão Importar, também borda `--line-9`).
| Variante | Padding / tamanho | Raio | Texto |
|---|---|---|---|
| Importar (sidebar) | 8px 10px, 100% | 14 | 13/500 `--tx-1` |
| Converter | 6px 12px | 12 | 12.5 `--tx-2` |
| Ícone (Mais) | 30×29 | 12 | ícone 15 `--tx-3` |
| Ícone pequeno (fullscreen) | 26×24, fundo `--pop` | 10 | ícone 13 `--tx-3` |

## Campo de texto
Mesma família do botão secundário: borda 1px `--line-6`, fundo `--surf-2`, raio 12, `padding: 6px 10px`, 13px `--tx-1`, placeholder `--tx-8`. Foco: borda `--line-9`, sem anel. Com erro: borda `--danger`, e a mensagem abaixo do campo em 12px `--danger`.

## Linha de navegação (sidebar)
Raio 12, gap 8, cursor pointer, hover `--hover`. Padding vertical 5px (arquivos e recentes) ou 6px (pastas), horizontal 8px. Nome 13px com ellipsis.

## Botão de pin
Aparece em favoritos, pastas, arquivos da árvore e recentes.
- Margin-left auto, `padding: 2px`, raio 8, ícone Lucide `pin` 12px
- Cor: fixado `--nav-accent`, não fixado `--tx-8`
- Opacidade: fixado 1; não fixado e linha em hover 0.85; caso contrário 0
- Hover no próprio botão: opacidade 1, cor `--accent`
- Transição: opacidade e cor 150ms ease
- title: "Adicionar aos favoritos" ou "Remover dos favoritos"
- Clique: `stopPropagation` (não abre a linha) e alterna o favorito
- O hover é rastreado **por linha e por contexto**: o mesmo arquivo em Recentes e na árvore são linhas diferentes

## Card
Padding 14, borda 1px `--line-4`, raio 18, fundo `--surf-2`. Hover: borda `--line-9`.

## Rótulo de seção
11/600, uppercase, letter-spacing 0.08em. Sidebar `--tx-5`, home `--tx-9`.

## Dropdown
Fundo `--pop`, borda 1px `--bar`, raio 18, padding 5, sombra `--shadow-2`, z-index 40. Itens com raio 12, `padding: 7px 9px`, 13px, hover `--hover-3`. Abre 38px abaixo do topo do grupo de botões, sem animação.

## Diálogo
- Painel com a aparência do dropdown e do toast: fundo `--pop`, borda 1px `--bar`, raio 18, sombra `--shadow-2`
- Largura 440 (no máximo a janela menos 32), `padding: 18px`, altura máxima 80% da janela com rolagem interna, centralizado, z-index 50 (acima do dropdown, abaixo do toast)
- Fundo escurecido: `--bg` com 60% de opacidade
- Título no estilo do título do documento (15/600 `--tx-1`); à direita, o botão Fechar no estilo "Ícone pequeno" do botão secundário, com o ícone Lucide `x` e title "Fechar"
- Fecha com `Esc`, clique fora ou o botão Fechar; o foco volta para quem abriu
- Sem animação de entrada ou saída

## Toast
- Absoluto: `left: 50%; bottom: 26px; transform: translateX(-50%)`, z-index 60
- Flex, gap 11, `padding: 10px 14px`, raio 18, fundo `--pop`, borda 1px `--bar`, sombra `--shadow-2`
- Ponto 7×7 circular `--teal` + texto 12.5px
- Some sozinho após 2600ms. Um toast novo substitui o anterior e reinicia o timer
- **Fixo:** um toast fixo não some sozinho; fica até outro toast substituí-lo (a conversão em andamento, `specs/08-conversao.md`)
- **Ação:** um botão de texto opcional no fim ("Cancelar"): 12.5px/500 `--tx-1`, `padding: 2px 6px`, raio 10, hover e foco `--hover-3`
- Sem animação de entrada ou saída no protótipo

## Chip de formato / badge
- Chip: Mono 9/500, `padding: 2px 4px`, raio 7, fundo `--surf-4`, `--tx-1`
- Badge de versão: Mono 10, `padding: 2px 5px`, raio 8, mesmas cores
