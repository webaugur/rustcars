// Book template for the rustcars manual. Built by docs/build-manual.sh (pandoc -> typst).
#let accent = rgb("#9C805A")
#let head-font = "Antonio"
#set document(title: "rustcars Developer Manual", author: "David Norris")
#set text(font: "Noto Serif", size: 10pt, lang: "en")
#set par(justify: true, leading: 0.62em)
#show raw: set text(font: "DejaVu Sans Mono", size: 7pt)
#show raw.where(block: true): it => block(fill: luma(245), inset: 6pt, radius: 2pt, width: 100%, it)
#show heading: set text(font: head-font, weight: "bold", fill: rgb("#3a3a3a"))
#show heading.where(level: 2): set text(size: 14pt)
#show heading.where(level: 3): set text(size: 11.5pt)
#set table(stroke: 0.4pt + luma(170), inset: 4pt)
#show table: set text(size: 8pt)
#show link: set text(fill: rgb("#3A4E66"))

// ---------------- title page
#set page(width: 6in, height: 9in)
#page(margin: 0.8in, header: none, footer: none)[
  #v(0.6in)
  #text(font: head-font, size: 44pt, weight: "bold", fill: accent)[RUSTCARS]
  #v(-0.25in)
  #text(font: head-font, size: 20pt)[DEVELOPER MANUAL]
  #v(0.15in)
  #text(size: 11pt)[Building LCARS-style interfaces with egui and Rust]
  #v(0.4in)
  #image("preview.png", width: 100%)
  #v(1fr)
  #text(font: head-font, size: 12pt)[DAVID NORRIS · 2026]
]

// ---------------- copyright page
#page(header: none, footer: none, margin: (inside: 0.875in, outside: 0.625in, y: 0.75in))[
  #v(1fr)
  #set text(size: 8pt)
  #set par(justify: false)
  *rustcars Developer Manual*. Copyright © 2026 David Norris. The rustcars source code is released under the MIT License (see `LICENSE`).

  The bundled typeface *Antonio* is Copyright © 2013 The Antonio Project Authors (github.com/googlefonts/antonioFont), licensed under the SIL Open Font License 1.1. The license text is in `assets/fonts/OFL.txt`; it must be distributed with the font.

  LCARS design guidance is adapted from Bracer Jack's _LCARS GuideLine_ and _The LCARS Manifesto_, republished at lcars-terminal.de. The structure of this manual is inspired by Apple's _Macintosh Human Interface Guidelines_ (1987/1992). It is inspiration only: no Apple text is reproduced, and Apple does not endorse this work. The paint-behind frame technique was inspired by paxel/dedup-rs (MIT).

  Star Trek and LCARS are trademarks and/or copyrights of their respective owners (Paramount/CBS). LCARS was designed by Michael Okuda. rustcars is an unofficial fan project and is not affiliated with or endorsed by those owners.

  Typeset with Typst. Body: Noto Serif. Headings: Antonio. Code: DejaVu Sans Mono.
]

// ---------------- contents
#page(header: none, margin: (inside: 0.875in, outside: 0.625in, y: 0.75in))[
  #text(font: head-font, size: 24pt, weight: "bold")[CONTENTS]
  #v(0.2in)
  #outline(title: none, depth: 2, indent: 1em)
]

// ---------------- body
#let chapter-title = state("chapter", "")
#set page(
  width: 6in, height: 9in,
  margin: (inside: 0.875in, outside: 0.625in, top: 0.8in, bottom: 0.75in),
  header: context {
    let p = here().page()
    // no running head on chapter-opening pages
    if query(heading.where(level: 1)).any(h => h.location().page() == p) { return }
    set text(font: head-font, size: 8.5pt, fill: luma(90))
    // chapter title of the last chapter that started on or before this page
    let chs = query(heading.where(level: 1)).filter(h => h.location().page() <= p)
    let title = if chs.len() > 0 { chs.last().body } else { [] }
    if calc.even(p) [RUSTCARS DEVELOPER MANUAL #h(1fr)]
    else [#h(1fr) #upper(title)]
    v(-6pt); line(length: 100%, stroke: 0.4pt + accent)
  },
  footer: context {
    let p = counter(page).display()
    set text(font: head-font, size: 9pt)
    if calc.even(here().page()) [#p #h(1fr)] else [#h(1fr) #p]
  },
)
#show heading.where(level: 1): it => {
  pagebreak(to: "odd", weak: true)
  chapter-title.update(it.body)
  v(0.9in)
  block(text(font: head-font, size: 26pt, weight: "bold", fill: accent, it.body))
  line(length: 35%, stroke: 2pt + accent)
  v(0.25in)
}
#counter(page).update(1)

#include "build/body.typ"
