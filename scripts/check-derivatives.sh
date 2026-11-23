#!/usr/bin/env bash
# Opens the derivatives that conversion_corpus wrote in the reference
# programs of each format (specs/08-conversao.md, "Critério mínimo de
# qualidade", item 3): LibreOffice for DOCX, pdftotext for PDF, and a UTF-8
# check for MD and TXT. Prints one line per file that fails.
#
#   scripts/check-derivatives.sh <output dir>/Folio <corpus dir>
set -u

workspace=$1
corpus=$2
failed=0
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

fail() {
  echo "FAILED: $1"
  failed=1
}

for file in "$workspace"/*; do
  name=$(basename "$file")
  # Only derivatives: the originals came from the corpus.
  [ -e "$corpus/$name" ] && continue
  case "$name" in
    *.docx)
      soffice --headless --convert-to pdf --outdir "$scratch" "$file" >/dev/null 2>&1
      [ -s "$scratch/${name%.docx}.pdf" ] || fail "$name: LibreOffice cannot open it" ;;
    *.pdf)
      pdftotext "$file" - >/dev/null 2>"$scratch/err" || fail "$name: pdftotext cannot read it"
      [ -s "$scratch/err" ] && fail "$name: pdftotext warns: $(head -1 "$scratch/err")" ;;
    *.md | *.txt)
      iconv -f UTF-8 -t UTF-8 "$file" >/dev/null 2>&1 || fail "$name: not valid UTF-8" ;;
  esac
done

[ "$failed" = 0 ] && echo "all derivatives open"
exit "$failed"
