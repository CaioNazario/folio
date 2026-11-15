# 02 · Shell / layout

Janela inteira: `display:flex; height:100vh; width:100%; overflow:hidden`. Fundo `--bg`, cor `--tx-2`, Plex Sans 14px. Preview de referência: 1440×900.

```
┌──────────────┬──────────────────────────────────────────────┐
│ Sidebar      │ Área principal                               │
│ min(272px,   │  ├─ Home (view = "home")                     │
│ 24vw)        │  └─ Documento (view = "doc")                 │
│ min 196px    │                                              │
│              │  Toast (absoluto, centro inferior)           │
└──────────────┴──────────────────────────────────────────────┘
```

## Sidebar
- `width: min(272px, 24vw)`, `flex: 0 0 min(272px, 24vw)`, `min-width: 196px`
- Coluna flex, fundo `--bg`, `border-right: 1px solid --surf-4`
- Ordem: header (46px fixo) → ações (auto) → área rolável (flex 1, overflow-y auto) → rodapé (auto)

## Área principal
- `flex: 1; min-width: 0`, coluna flex, `position: relative` (âncora do toast)
- Renderiza **uma** view por vez: Home ou Documento
- O toast fica sobreposto às duas

## Tema
Atributo `data-theme` no elemento raiz. No light, o `html`/`body` também recebe fundo #ffffff para não aparecer preto no overscroll.
