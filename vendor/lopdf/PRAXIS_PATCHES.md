# Praxis vendor patches against lopdf 0.40.0

Reason this directory exists: lopdf 0.40.0 contains all the
machinery praxis's M4.γ PDF text-extraction pipeline needs
(`ToUnicodeCMap` parser, standard encoding tables, Adobe Glyph
List), but the **public API surface** hides the relevant types
and modules. Downstream crates can construct
`Encoding::OneByteEncoding(table)` and
`Encoding::SimpleEncoding(name)` but cannot construct
`Encoding::UnicodeMapEncoding(ToUnicodeCMap)` because the inner
type is in a private module.

Without the patch, our Phase 4 (`crates/domains/src/social/software/binary/pdf/font.rs`)
falls back to `Encoding::SimpleEncoding(b"WinAnsiEncoding")` for
every font, which produces the `Ð`-for-em-dash glyph
substitution we saw on the SOX 1514A govinfo PDF. With the
patch, Phase 4 can use the font's own `/ToUnicode` CMap
(Adobe Tech Note #5014; ISO 32000-2:2020 §9.10.2) — the
authoritative per-font glyph-code → Unicode map — and get
faithful text without any hand-coded substitution rules.

## What changed

Three pub-export additions:

```rust
// crates/lopdf/src/lib.rs — at line 43, alongside the existing
// `pub use encodings::{Encoding, encode_utf8, encode_utf16_be};`
pub use encodings::cmap;
pub use encodings::glyphnames;
pub use encodings::mappings;
```

```rust
// crates/lopdf/src/encodings/mod.rs — first two lines
pub mod cmap;        // already pub
pub mod glyphnames;  // was `mod glyphnames;`
pub mod mappings;    // was `mod mappings;`
```

That's it. Six pub keywords added; zero behavior changes; zero
new types or methods.

## Why these matter for downstream crates

| Now reachable | Why it matters |
|---|---|
| `lopdf::cmap::ToUnicodeCMap` | Lets downstream construct `Encoding::UnicodeMapEncoding(...)`. Without it, the variant is publicly visible but unusable. |
| `lopdf::cmap::BfRangeTarget`, `ReverseCMapEntry`, etc. | Lets downstream build / mutate CMaps programmatically (test fixtures, custom mappings). |
| `lopdf::glyphnames::name_to_unicode(name) -> Option<u16>` | Adobe Glyph List access. Required to resolve `/Differences` arrays per ISO 32000-2:2020 §9.6.5.4 (each diff names a glyph; we need its Unicode codepoint). |
| `lopdf::mappings::{WIN_ANSI_ENCODING, PDF_DOC_ENCODING, MAC_ROMAN_ENCODING, ...}` | The 256-entry standard encoding tables from Annex D. Lets downstream pass them to `Encoding::OneByteEncoding(&table)` for non-WinAnsi encodings. lopdf's internal `SimpleEncoding(b"WinAnsiEncoding")` dispatch only handles WinAnsi by name. |

## Upstream PR plan

One focused PR against `J-F-Liu/lopdf`:

- **Title:** Pub-export `cmap` / `glyphnames` / `mappings` modules
- **Diff:** the six pub-keyword additions documented above.
- **Body:** the rationale section above, with reference to merged
  PRs #314 and #328 which added the ToUnicode machinery (the
  functionality already exists — only the visibility was
  missing).
- **No tests needed** — these are pure visibility changes; no
  behavior change.

When the PR lands in a released version:

1. Drop the `[patch.crates-io]` block from the workspace root
   `Cargo.toml`.
2. Bump `lopdf = "..."` in `crates/domains/Cargo.toml` to the
   released version that includes the exports.
3. Remove this `vendor/lopdf/` directory.
4. The Phase 4 `font.rs` can drop its `Unsupported(ToUnicodeCmap)`
   fallback variant and decode the CMap directly.

## Open lopdf items relevant to praxis

- **PR #493** (open) — `/Differences` encoding implementation.
  Once merged, our Phase 4 can drop the
  `Unsupported(Differences)` fallback.
- **Issue #250** (open since 2023) — "Wrong letters in PDF".
  Likely the canonical case of "ToUnicode not applied when caller
  uses SimpleEncoding"; the visibility patch here is half of the
  fix.
- **Issue #463** (open Jan 2026) — Parse error when encoding is
  an indirect reference. Affects how `/Encoding` is resolved
  before reaching the lopdf encoding dispatcher.
