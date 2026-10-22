#!/usr/bin/env bash
# Rebuild docs/rustcars-manual.pdf from docs/manual/*.md.
# Needs: pandoc (>= 3.1), typst (>= 0.11), fonts "Noto Serif" and "DejaVu Sans Mono" installed.
# Antonio is loaded from assets/fonts via --font-path.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p build
: > build/body.typ
for f in manual/[0-9]*.md; do
  pandoc -f gfm -t typst "$f" >> build/body.typ
  printf '\n' >> build/body.typ
done
# Image paths in chapters are relative to docs/manual; body.typ lives in docs/build.
sed -i 's#"\.\./preview\.png"#"../preview.png"#g' build/body.typ
typst compile --root . --font-path ../assets/fonts manual.typ rustcars-manual.pdf
echo "wrote docs/rustcars-manual.pdf ($(pdfinfo rustcars-manual.pdf 2>/dev/null | awk '/^Pages/{print $2}') pages)"
