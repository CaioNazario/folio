# 04 · Home ("Documentos")

Ativada pelo botão "Importar documento". Container rolável (`flex:1; overflow-y:auto`). Conteúdo: `max-width: 880px; margin: 0 auto; padding: 72px 40px 60px`.

## Cabeçalho
- "Documentos": 26px / 600, letter-spacing -0.01em
- Subtítulo (margin-top 6, 14px, `--tx-7`): "Somente leitura e conversão. Tudo armazenado neste computador."

## Ações rápidas
Grid de 3 colunas iguais, gap 10, margin-top 32.

**Card de ação:** padding 14, borda 1px `--line-4`, raio 18, fundo `--surf-2`, cursor pointer. Hover: borda `--line-9`.
- Linha do título: flex, gap 9, 13/500. Ícone 15px `--icon`
- Descrição: margin-top 3, 12px, `--tx-8`

| Ícone (Lucide) | Título | Descrição |
|---|---|---|
| upload | Importar arquivos | Arraste ou selecione do disco |
| file-plus | Converter arquivo | PDF, Markdown, TXT ou imagem |
| folder-plus | Novo projeto | Agrupe documentos por contexto |

Os cards não têm ação no protótipo.

## Continuar de onde parou
- Cabeçalho (margin-top 40): flex, `align-items: baseline`, `space-between`
  - Rótulo "Continuar de onde parou": 11/600, uppercase, 0.08em, `--tx-9`
  - "Ver tudo": 12px `--tx-8`, cursor pointer, sem ação
- Lista (margin-top 12): borda 1px `--line-4`, raio 18, `overflow:hidden`
- Linha: flex, gap 12, `padding: 12px 14px`, `border-bottom: 1px solid --line-3`, fundo `--surf-2`. Hover: `--hover-2`
  - Nome 13px
  - Meta 12px `--tx-8` ("Faculdade · 42 KB")
  - Tempo em Mono 11px `--tx-10`, margin-left auto
- Usa a mesma lista de Recentes (7 itens). Clique: igual ao item de Recentes da sidebar

## Projetos
- Rótulo "Projetos" (margin-top 36, mesmo estilo de rótulo da home)
- Grid de 4 colunas, gap 10, margin-top 12
- Card: mesmo estilo do card de ação
  - Ícone `folder` 16px `--icon`
  - Nome (margin-top 12): 13/500
  - Contagem (margin-top 2): Mono 11px `--tx-10`, "{n} documentos"
- Clique: abre/fecha o projeto na sidebar (não muda de view)

## Responsividade
O protótipo usa colunas fixas (3 e 4). Na implementação, usar `repeat(auto-fill, minmax(200px, 1fr))` ou breakpoints para que os cards quebrem de linha em larguras estreitas.
