# 09 · Ícones e assets

## Ícones
Todos são **Lucide** (lucide.dev): stroke 2 (exceto o plus), `stroke-linecap/linejoin: round`, `fill: none`, `currentColor`.

| Ícone | Tamanho | Onde |
|---|---|---|
| plus | 15 (stroke 3) | Importar documento |
| search | 13 | Busca |
| pin | 12 | Botão de pin |
| chevron-right | 12 | Expandir pasta |
| folder | 13 / 14 / 16 | Favoritos, árvore, cards de projeto |
| folder-open | 14 | Pasta aberta |
| file | 13 | Arquivo nos favoritos |
| sun / moon / monitor | 13 | Seletor de tema |
| upload | 15 | Card "Importar arquivos" |
| file-plus | 15 | Card "Converter arquivo" |
| folder-plus | 15 | Card "Novo projeto" |
| chevron-down | 13 | Botão Converter |
| ellipsis-vertical | 15 | Botão Mais |
| maximize / minimize | 13 | Fullscreen |
| minus / plus | 13 | Zoom do palco |
| languages / check | 13 | Seletor de idioma |

## Logo (wordmark)
SVG inline, viewBox `0 0 164 58`, renderizado com height 22, `fill: currentColor`:
```svg
<svg viewBox="0 0 164 58" height="22" xmlns="http://www.w3.org/2000/svg">
  <path d="M3 4H22V52H3A3 3 0 0 1 0 49V7A3 3 0 0 1 3 4Z" fill="currentColor"/>
  <path d="M26 0H45A3 3 0 0 1 48 3V45A3 3 0 0 1 45 48H26Z" fill="currentColor"/>
  <text x="62" y="46" font-family="'Source Serif 4','Source Serif Pro',Georgia,serif"
        font-size="46" font-weight="600" letter-spacing="-0.9" fill="currentColor"
        textLength="96" lengthAdjust="spacingAndGlyphs">Folio</text>
</svg>
```
O símbolo são duas folhas deslocadas (a esquerda 4px mais baixa). Converter o texto em path para não depender da fonte.

## Fontes
Google Fonts: `IBM Plex Sans` (400, 500, 600) e `IBM Plex Mono` (400, 500). Georgia é fonte do sistema. Source Serif 4 só se o logo continuar como texto.

## Imagens
Nenhuma.
