# 10 · Compartilhamento

Como os documentos são arquivos reais fora de `.folio/`, o que se compartilha é sempre o próprio documento, nunca uma cópia escondida. Esta spec cobre como um documento sai do Folio para o sistema e para outros apps.

## Spikes

Resolver antes de implementar a parte que cada spike bloqueia. O resultado vai para a seção da spec que ele decide e para o "Resultado" do spike.

### S8 · "Copiar arquivo" no Wayland/Hyprland

**Bloqueia:** Copiar arquivo (abaixo).

**Pergunta:** colocar o arquivo no clipboard como `text/uri-list` funciona de forma confiável no Wayland/Hyprland, a ponto de colar no gerenciador de arquivos e em apps como o WhatsApp Web?

**Como testar:** `arboard` 3.7 com a feature `wayland-data-control` (`Set::file_list`), no Omarchy (Hyprland). Copiar um PDF pelo Folio e colar:

1. No Nautilus e no Thunar (o Nautilus pode exigir também `x-special/gnome-copied-files`; se exigir, testar `wl-clipboard-rs`, que aceita tipos MIME arbitrários)
2. No WhatsApp Web, no Chromium e no Firefox
3. No Telegram Desktop
4. Depois de trocar de janela e voltar (no Wayland, o conteúdo some se o app que copiou não estiver mais servindo o clipboard)
5. Repetir num ambiente X11, para o Linux fora do Omarchy

**Resultado:** pendente. Se não funcionar de forma confiável, "Copiar arquivo" sai da 0.1.0 no Linux.

### S9 · Arrastar para fora no WebKitGTK/Wayland

**Bloqueia:** Arrastar para fora (abaixo).

**Pergunta:** o crate `drag` 2.x (o mesmo do `tauri-plugin-drag`) inicia, a partir de um command do Rust, um arraste nativo de arquivo por cima do webview do WebKitGTK no Hyprland, e o destino recebe uma cópia?

**Como testar:** iniciar o arraste no `mousedown` + movimento de uma linha da árvore, chamando um command que roda `drag::start_drag` na thread principal com `DragItem::Files` e `DragMode::Copy`. Soltar:

1. No Nautilus e no Thunar: o arquivo é **copiado**, nunca movido, inclusive no mesmo filesystem
2. No WhatsApp Web e no Telegram Desktop: o arquivo vira anexo
3. De volta na janela do Folio: nada acontece
4. Conferir o ícone de arraste e o cancelamento com `Esc`

**Resultado:** pendente. Se não funcionar, "Arrastar para fora" sai da 0.1.0 no Linux.

## Ações

| Ação | Linux (0.1.0) | Android (futuro) |
|---|---|---|
| Mostrar na pasta | Sim | Não se aplica |
| Arrastar para fora | Sim, se o S9 passar | Não se aplica |
| Copiar arquivo | Sim, se o S8 passar | Não se aplica |
| Compartilhar via… | Não se aplica | Share sheet nativo com `FileProvider`, especificado com a versão Android |

**O frontend nunca recebe caminho.** Toda ação é um command que recebe o id do documento (ou o nome, na Entrada, spec 09); o core resolve o caminho e o `src-tauri` chama o plugin do lado do Rust. Por isso as capabilities do webview não liberam as APIs JavaScript do `plugin-opener` nem do clipboard.

**Nenhuma ação altera o documento.** Todas só entregam o caminho ao sistema; o que o outro app faz com o arquivo é com ele.

### Mostrar na pasta

- Abre o gerenciador de arquivos com o documento selecionado: `reveal_item_in_dir` do `tauri-plugin-opener`, que no Linux usa a interface D-Bus `org.freedesktop.FileManager1` (Nautilus, Thunar, Dolphin).
- Sem essa interface, abre a pasta do documento, sem seleção.
- Num link simbólico (spec 04), mostra o link, no workspace, não o arquivo real.
- Para projeto ou pasta, abre o próprio diretório.
- É o item "Abrir pasta no sistema" do menu Mais (`Design/specs/05-document-view.md`), com o texto trocado para "Mostrar na pasta", que diz melhor o que acontece.

### Arrastar para fora

- Arrastar um documento de uma linha da árvore, dos Favoritos, dos Recentes, dos resultados da busca ou da Entrada, ou o título do documento aberto, para outro app.
- **Sempre cópia:** o arraste anuncia só a operação de copiar. Soltar no gerenciador de arquivos nunca tira o documento do workspace.
- Arraste que começa e termina no Folio é ignorado: não dispara a importação da spec 09.
- Um documento protegido sai como está, cifrado; quem recebe precisa da senha.

### Copiar arquivo

- Coloca o documento no clipboard como `text/uri-list` (`file:///…`), pelo `arboard`, para colar no gerenciador de arquivos ou num app de mensagens.
- **No Wayland, o Folio precisa continuar servindo o clipboard:** o objeto do `arboard` vive no estado do app enquanto ele estiver aberto. Fechar o Folio esvazia o clipboard; o toast diz "Arquivo copiado".
- Item do menu Mais e atalho `Ctrl+C` com um documento selecionado na árvore (fora de campos de texto e sem texto selecionado no palco, onde `Ctrl+C` copia o texto).

## Onde as ações aparecem

- **Menu Mais** do documento aberto, na ordem: Renomear, Mostrar na pasta, Copiar arquivo, Propriedades, Mover para lixeira. ("Adicionar tag", do protótipo, fica fora da 0.1.0.)
- **Menu de contexto** (clique com o botão direito) nas linhas de documento da árvore, dos Favoritos, dos Recentes, dos resultados da busca e da Entrada, com os mesmos itens do menu Mais que fazem sentido ali. Usa o dropdown de `Design/specs/06-components.md`, aberto na posição do clique. Em linhas de projeto e pasta: Renomear, Mostrar na pasta, Ícone, Mover para lixeira.

## Erros

| Erro | Quando |
|---|---|
| `ClipboardIndisponivel` | Não foi possível servir o clipboard (compositor sem data-control) |

Mais `DocumentoNaoEncontrado` (spec 04), quando o arquivo sumiu antes da ação.

## Critérios de aceite

- [ ] "Mostrar na pasta" abre o gerenciador de arquivos com o documento selecionado; sem `FileManager1`, abre a pasta.
- [ ] Arrastar um documento para o Nautilus cria uma cópia; o original continua no workspace.
- [ ] Arrastar um documento para o WhatsApp Web anexa o arquivo.
- [ ] "Copiar arquivo" e colar no Nautilus cria uma cópia (se o S8 passar).
- [ ] O frontend não recebe caminho absoluto em nenhuma ação, e as capabilities do webview não liberam o `plugin-opener` nem o clipboard.
- [ ] O menu de contexto aparece nas linhas de documento e de diretório, com os itens de cada uma.
