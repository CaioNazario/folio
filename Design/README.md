# Handoff: Folio — Workspace Documental

## Visão geral
Folio é um workspace desktop local para **ler, organizar e converter documentos** (sem edição). O usuário navega por projetos (pastas), fixa favoritos, abre um documento num leitor com página renderizada e converte para outros formatos. O arquivo original é sempre preservado; a conversão gera um "derivado".

## Sobre os arquivos
`Folio v5.dc.html` é uma **referência de design em HTML** (protótipo), não código de produção. A tarefa é **recriar este design no ambiente do seu projeto** (React, Vue, Svelte, Electron, Tauri etc.) usando os padrões e bibliotecas já existentes. Se ainda não houver stack, sugerimos React + CSS variables (os tokens já estão nesse formato).

Para abrir o protótipo: coloque `Folio v5.dc.html` e `support.js` na mesma pasta e abra no navegador (precisa de servidor local, ex. `npx serve`).

## Fidelidade
**Alta fidelidade (hi-fi).** Cores, tipografia, espaçamentos, raios e transições são finais. Recriar pixel a pixel.

## Índice das specs
0. [Visão geral](specs/00-visao-geral.md): mapa, glossário, ordem de implementação e divergências com a 0.1.0
1. [Tokens](specs/01-tokens.md): cores (dark/light), tipografia, raios, sombras, motion
2. [Shell / layout](specs/02-layout.md): estrutura da janela
3. [Sidebar](specs/03-sidebar.md): logo, ações, favoritos, projetos, recentes, rodapé e seletor de tema
4. [Home](specs/04-home.md): tela "Documentos"
5. [Visualização de documento](specs/05-document-view.md): cabeçalho, menus, palco e página
6. [Componentes compartilhados](specs/06-components.md): botões, linhas, dropdown, toast, botão de pin
7. [Estado e interações](specs/07-state-interactions.md): modelo de estado, fluxos e regras
8. [Dados](specs/08-data.md): modelo de dados e conteúdo mock
9. [Ícones e assets](specs/09-assets.md)
10. [Pendências e inconsistências](specs/10-known-issues.md): o que o protótipo não resolve

## Arquivos
- `Folio v5.dc.html`: protótipo de referência (template + lógica no mesmo arquivo)
- `support.js`: runtime do protótipo (só para abrir o HTML, não portar)
