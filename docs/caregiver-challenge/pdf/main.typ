// Entry point for one ACL Caregiver AI Challenge narrative PDF. Compiled by
// `dev-caregiver-pdf` (devenv.nix):
//   1. pandoc converts the source markdown to a build/*.typ fragment
//      (docs/caregiver-challenge/pdf/pandoc-typst-fixups.lua turns each
//      \newpage raw-tex block into a Typst #pagebreak(), and a `---`
//      thematic break into a literal horizontal-rule expansion).
//   2. typst compile --input src=build/<name>.typ main.typ <name>.pdf
//      #include's that one fragment against the page/font setup below.
//
// Page/format spec is ACL's own Phase 1 requirement (acl.gov, fetched
// 2026-07-21): US Letter, 1-inch margins, >=11pt widely-available font
// (Times New Roman named as an example — Liberation Serif is its
// metric-compatible free substitute, loaded via TYPST_FONT_PATHS so this
// doesn't depend on a proprietary font being installed), single-spacing
// permitted, page numbers required.
//
// `#include` needs a literal path, not a dynamic expression, so the driver
// script copies whichever document is being built to build/fragment.typ
// (one at a time) right before invoking `typst compile main.typ`.

// Typst defaults to an anonymized epoch date (1979-12-31) for reproducible
// builds unless a real date is set explicitly -- set here from the driver
// script's --input build-date=YYYY-MM-DD (today's date at build time)
// rather than left looking like a file-metadata bug in a federal
// submission's PDF properties. No argless `datetime.today()` exists in
// Typst (same reproducible-build reasoning as the epoch default), so this
// has to come in as an input, not be computed here.
#let build-date = sys.inputs.at("build-date", default: "2026-01-01").split("-").map(int)
// /Title metadata is a standard 508/PDF-UA checker item -- passed per-document
// by the driver script (--input doc-title=...), since main.typ compiles a
// different fragment each invocation.
#let doc-title = sys.inputs.at("doc-title", default: "ACL Caregiver AI Challenge Application")
#set document(
  title: doc-title,
  date: datetime(year: build-date.at(0), month: build-date.at(1), day: build-date.at(2)),
)
#set page(
  paper: "us-letter",
  margin: 1in,
  numbering: "1",
  footer: context [
    #align(center, text(size: 9pt, fill: luma(40%))[#counter(page).display()])
  ],
)
// Academic-paper typography: justified text (flush left AND right, with
// hyphenation so justification doesn't open rivers), Typst's default
// comfortable leading, and clear blank-line separation between paragraphs
// (block-paragraph style, the grant-narrative convention). Sections flow
// continuously rather than each forcing a page break -- the page budget
// (ACL's 15-page narrative cap) is spent on content, not section-break
// whitespace. 11pt is ACL's floor and the academic standard.
#set text(font: "Liberation Serif", size: 11pt, lang: "en")
#set par(justify: true, leading: 0.55em, spacing: 0.9em)
#show heading.where(level: 1): it => [
  #set text(size: 14pt, weight: "bold")
  #block(above: 16pt, below: 9pt)[#it.body]
]
#show heading.where(level: 2): it => [
  #set text(size: 12pt, weight: "bold")
  #block(above: 12pt, below: 6pt)[#it.body]
]
#show link: it => text(fill: rgb("#1f6feb"))[#it]
// ACL format spec: body text >=11pt; monospace log/data content >=10pt
// (Tech Requirements Guide names Courier New/Consolas, minimum 10pt).
// Inline code sits in body prose, so it gets the body minimum; block code
// (verbatim response logs) gets the monospace minimum.
#show raw.where(block: false): it => text(font: "Liberation Mono", size: 11pt)[#it]
#show raw.where(block: true): it => text(font: "Liberation Mono", size: 10pt)[#it]

// Tables MUST be able to continue onto the next page.
//
// Pandoc's typst writer wraps every table as `#figure(align(center)[#table(…)])`,
// and a Typst figure is laid out in a NON-BREAKABLE block by default. A table
// taller than the space left on the page therefore does not continue — it
// overflows, and every remaining row is drawn on top of the others in a band at
// the page foot, illegible and printing over the page number.
//
// That is not hypothetical: it silently corrupted the last six rows of the
// criterion-map table in track2-phase1-appendix (16 rows, several of them long
// paragraphs), and the `Caregiver PDF page limit` CI job could not see it —
// that gate counts PAGES, and overlapping text costs no extra page. A reviewer
// reading the submitted PDF was the detector.
#show figure: set block(breakable: true)

#include "build/fragment.typ"
