// The page template of the PDF writer (spec 08, "Escritores"). The document
// arrives as data in `sys.inputs`; its text is only ever inserted as
// strings, never evaluated as Typst code.

#let doc = sys.inputs

// The date stays `auto`: the writer passes the creation date to the PDF
// export, and without one the PDF has none.
#set document(
  title: doc.at("title", default: none),
  author: doc.authors,
  description: doc.at("subject", default: none),
  keywords: doc.keywords,
)

// A4 with 2.5 cm margins; justified text without hyphenation, because the
// PDF reader keeps a line-end hyphen (spec 07). Justifying only stretches
// spaces: a shrunk one is too narrow for other programs to see a space.
#set page(paper: "a4", margin: 2.5cm)
#set text(font: "Libertinus Serif", size: 11pt, hyphenate: false)
#set par(justify: true, justification-limits: (spacing: (min: 100%, max: 150%)))
#show raw: set text(font: "DejaVu Sans Mono", size: 9pt)

// Each heading level its own size, all at least 1.16 times the body: the
// PDF reader takes 1.15 times the body as a heading (spec 07).
#let heading-sizes = (1.8em, 1.6em, 1.45em, 1.3em, 1.2em, 1.16em)
#show heading: it => block(above: 1.4em, below: 0.8em, text(size: heading-sizes.at(it.level - 1), weight: "bold", it.body))

// Typst closures cannot call each other, so each function only recurses
// into itself.
#let inline(node) = {
  let inner = () => node.content.map(inline).join()
  if node.kind == "text" { node.text }
  else if node.kind == "bold" { strong(inner()) }
  else if node.kind == "italic" { emph(inner()) }
  else if node.kind == "code" { raw(node.text) }
  else if node.kind == "break" { linebreak() }
  // The model does not mark where `#anchor` leads, so it stays text.
  else if node.kind == "link" and node.target.starts-with("#") { inner() }
  else if node.kind == "link" { link(node.target, inner()) }
}

#let inlines(nodes) = nodes.map(inline).join()

// The bullets the PDF reader recognizes (spec 07), one per level.
#set list(marker: ([•], [◦], [▪]))

// A grid with borders; the first row is a header repeated on each page.
#let table-of(node) = {
  let rows = node.rows.map(row => row.map(inlines))
  table(columns: node.columns, table.header(..rows.first()), ..rows.slice(1).flatten())
}

#let block-of(node) = {
  let all = nodes => nodes.map(block-of).join()
  if node.kind == "heading" { heading(level: node.level, inlines(node.content)) }
  else if node.kind == "paragraph" { par(inlines(node.content)) }
  else if node.kind == "list" and node.ordered { enum(..node.items.map(all)) }
  else if node.kind == "list" { list(..node.items.map(all)) }
  else if node.kind == "quote" { quote(block: true, all(node.blocks)) }
  else if node.kind == "table" { table-of(node) }
  else if node.kind == "separator" { line(length: 100%) }
  else if node.kind == "image" { image(node.bytes, alt: node.alt) }
  // Weak: a break on an empty page adds no blank page.
  else if node.kind == "pagebreak" { pagebreak(weak: true) }
  else if node.kind == "code" { raw(node.text, block: true, lang: node.at("language", default: none)) }
}

#doc.blocks.map(block-of).join()
