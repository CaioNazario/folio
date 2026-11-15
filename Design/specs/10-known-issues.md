# 10 · Pendências e inconsistências

Pontos do protótipo para decidir ou corrigir na implementação.

## Bugs visuais
1. **Ícone "+" com cor fixa:** o SVG tem `stroke: #F6F6F6` e `stroke="#FFFFFF"` no inline style, o que ignora `--plus` e fica invisível no tema claro. Usar `currentColor`
2. **Hover dos arquivos da árvore sem raio:** a linha tem fundo de hover sem border-radius, mas a seleção tem raio 12. Sugestão: aplicar o mesmo overlay arredondado no hover
3. **Hover do thumb da scrollbar** (#454545) e **letra do avatar** (#a9b0b8) são hardcoded e não mudam no light

## Copy
4. O texto vazio de Favoritos fala em "estrela", mas o ícone é um **pin**. **Resolvido:** o texto fala em pin (`03-sidebar.md`)
5. O botão "Importar documento" só leva para a Home. **Resolvido:** abre o seletor de arquivos (`specs/09-entrada.md`, Importação)

## Sem comportamento no protótipo
6. Busca
7. Cards de ação da Home e "Ver tudo"
8. Itens do menu Mais (Renomear, Adicionar tag, Abrir pasta no sistema, Mover para lixeira). **Resolvido:** itens e comportamento em `05-document-view.md` e `specs/10-compartilhamento.md`; "Adicionar tag" fica fora da 0.1.0
9. Toolbar do palco: "página 1 de 4 · 100%" é estático (sem paginação nem zoom)
10. Breadcrumb fixo em "Faculdade"
11. Contagens de projeto e o número "4" ao lado de Projetos são fixos
12. Fechar menus com clique fora ou Esc
13. Persistência de tema, favoritos e pastas abertas

## Lacunas de fluxo
14. Arquivos fora da timeline não abrem: o leitor continua no último item
15. Derivados criados pela conversão não aparecem na árvore da sidebar
16. Não há UI de timeline nem de diff na v5, mas os tokens e o modelo de dados existem. Decidir se entra
17. Sem estados de loading, erro (ex.: conversão falhou) ou vazio para projetos e recentes
18. Sem animação de entrada ou saída em dropdowns e toast

## Responsividade
19. Grids da Home com colunas fixas (3 e 4): usar auto-fill
20. A sidebar encolhe até 196px, mas não colapsa. Definir o comportamento em janelas estreitas
