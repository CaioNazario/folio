# 08 · Dados

## Tipos
```ts
type Ext = 'DOC' | 'PDF' | 'XLS' | 'PPT' | 'MD' | 'TXT' | 'PNG' | 'JPG' | 'CSV';

interface FileEntry { name: string; ext: Ext; timelineId?: string }

interface Project { key: string; name: string; count: number; files: FileEntry[] }

interface RecentEntry { name: string; ext: Ext; when: string; meta: string }   // meta = "Projeto · Tamanho"

interface Block { text: string; kind: 'p' | 'h' }

interface TimelineItem {
  id: string;
  kind: 'version' | 'derived';
  ext: Ext;
  title: string;    // nome do arquivo
  sub: string;      // descrição ("Versão 3 · atual", "Convertido da versão 2")
  date: string;     // "18 mar, 21:30" ou "agora"
  blocks: Block[];
  docDate: string;  // data na linha de autoria ("18 de março")
}
```

## Projetos (mock)
- **Faculdade** (fac, 11): Trabalho.docx·v1, Trabalho v2.docx·v2, Trabalho v2.pdf·d1, Trabalho v3.docx·v3, Referências.md, Notas de aula.txt, Fichamento Tanenbaum.md, Slides defesa.pptx, Cronograma.xlsx, Artigo base.pdf, Diagrama NFS.png
- **Trabalho** (tra, 9): Relatório Q1.xlsx, Proposta.pptx, Ata reunião 12-03.md, Orçamento 2026.xlsx, Escopo do projeto.docx, Contrato cliente.pdf, Métricas.csv, Roadmap.txt, Logo aprovada.png
- **Pessoal** (pes, 7): Contrato.pdf, Comprovante aluguel.pdf, Currículo.docx, Currículo.pdf, Receitas.md, Gastos mensais.xlsx, Passaporte.jpg
- **Arquivo** (pro, 6): Backup 2024.csv, Backup 2023.csv, Notas fiscais 2024.pdf, Projeto antigo.docx, Fotos evento.jpg, Leia-me.txt

## Recentes (mock)
| Nome | Meta | Quando |
|---|---|---|
| Trabalho v3.docx | Faculdade · 42 KB | há 2 d |
| Trabalho v2.pdf | Faculdade · 310 KB | há 6 d |
| Relatório Q1.xlsx | Trabalho · 88 KB | há 6 d |
| Proposta.pptx | Trabalho · 4,1 MB | há 9 d |
| Fichamento Tanenbaum.md | Faculdade · 12 KB | há 11 d |
| Orçamento 2026.xlsx | Trabalho · 64 KB | há 12 d |
| Currículo.pdf | Pessoal · 180 KB | há 20 d |

## Timeline (mock)
| id | kind | ext | title | sub | date | docDate |
|---|---|---|---|---|---|---|
| v1 | version | DOC | Trabalho.docx | Documento original | 12 mar, 14:02 | 12 de março |
| v2 | version | DOC | Trabalho v2.docx | Versão 2 · detectada no disco | 14 mar, 09:40 | 14 de março |
| d1 | derived | PDF | Trabalho v2.pdf | Convertido da versão 2 | 14 mar, 09:41 | 14 de março |
| v3 | version | DOC | Trabalho v3.docx | Versão 3 · atual | 18 mar, 21:30 | 18 de março |

O texto completo dos blocos (BASE, V2, V3) está no protótipo, nas constantes `BASE`, `V2` e `V3` do script.

## Badge de versão
- `kind === 'derived'` → "derivado"
- Caso contrário → "v" + número do id ("v1", "v2", "v3")

## Formatação
- Tamanhos com vírgula decimal pt-BR ("4,1 MB", "3,2 GB") e ponto de milhar ("1.284 arquivos")
- Tempo relativo curto: "há N d"
- Datas: "18 mar, 21:30"

## Cores por tipo de arquivo (reservado)
O protótipo define uma tabela `TINT` [fundo, tinta] por extensão, que **não é usada na v5**: DOC #26333f/#8ab4d8, PDF #3b2a2a/#d89a94, XLS e CSV #24352c/#8fc4a5, PPT #3a3125/#d9b070, MD #2f2b3a/#b0a6d8, TXT/PNG/JPG --line-4/#a9b0b8. Na v5 todos os chips são neutros.
