# Statute-shape test fixtures

These are **test fixtures**, not canonical statute text. They share
the structural shape of SOX § 1514A and AIR21 § 42121 (subsection
markers, indentation, heading style) but were hand-transcribed
from training-data recall, not extracted from the authoritative
GPO source.

**They are not pinned in `praxis.lock`. They do not appear in the
data-provisioning registry. No `RegistryEntry::local_path()`
resolves to these files. They have no authority for citation
purposes.**

## What they're for

The `statute_structure/` parser, term-extractor, relation-
extractor, and statute-report all need real-shape statute text
to exercise their behavior against. These fixtures provide that
text with realistic structural complexity (Bluebook §3.3.4
markers, multi-level subsections, embedded cross-references)
without making any claim to being canonical.

Tests in `crates/domains/src/social/judicial/statute_structure/`,
`crates/domains/src/social/compliance/compositions/audit.rs`
load these fixtures via `include_str!()` and verify the parser
produces the expected `ClauseTree` shape, the extractor finds
the right relation candidates, etc.

## The canonical text consumers — not these

For production canonical-text access:

- `crates/domains/src/social/compliance/statutes/sox_1514a/canonical_audit.rs`
- `crates/domains/src/social/compliance/statutes/air21_42121/canonical_audit.rs`

Both consume `PDF_EXTRACTION: PdfBuildExtraction` — the typed
build-time const emitted by `crates/domains/build.rs` from the
authoritative PDF at `RegistryEntry::local_path()`. State today:
`NotOnDisk` until `pr4xis update sox_1514a` (and AIR21) lands
the GPO-authenticated PDF.

When PDFs land on disk, `PDF_EXTRACTION` flips to
`Extracted { text, bytes_hash }` and the audit modules run
against authoritative text. These shape fixtures stay where they
are, for the parser/extractor unit tests.

## Provenance

`provenance: hand_transcribed_for_parser_tests_2026-05-15`. Not
canonical, not authoritative, not citable. If a parser or
extractor test asserts something about *this fixture's content*
that isn't satisfied by the canonical statute, that's a test bug
to fix — not a hint at the real statute's shape.

## Praxis rule alignment

`feedback_push_back_on_unsupported_file_types` — never
approximate from secondary sources. These fixtures are not
secondary-source approximations of statutes; they are
explicit, named test data with the same structural grammar.
The rule applies to production canonical-text claims, not to
explicitly-labeled test data.
