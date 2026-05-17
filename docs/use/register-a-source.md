# Register a Source

This page is the operator's guide for telling praxis about an **external source** — a published artifact the engine should reason about (a statute text, a lexicon dump, a regulation) — and managing its on-disk copy from the command line.

A registered source has three pieces, all at the workspace root:

- A **manifest entry** in `praxis.toml` that names the source, its version, its [`SourceTaxonomy`](../reference/glossary.md#sourcetaxonomy) type, and the authoritative URL.
- A **lock entry** in `praxis.lock` that pins the source's sha256 so any drift between the registered identity and the on-disk bytes is detected at startup by the `LockManifestAgreement` axiom.
- The **on-disk artifact** at the path `RegistryEntry::local_path()` derives from the type — e.g. `crates/domains/data/wordnet/english-wordnet-2025.xml` for the registered Language source.

The runtime side is the engine's data-provisioning subsystem (`pr4xis_domains::applied::data_provisioning`); the operator side is the `pr4xis update` CLI.

## The manifest — `praxis.toml`

One `[sources.<name>]` block per registered source. Schema:

```toml
[sources.<name>]
version     = "<publication identifier>"
type        = "<SourceTaxonomy leaf concept>"
url         = "<https URL of the authoritative source>"
description = "<one-line human description>"
```

- **`name`** is a snake-case identifier. Convention: `<short>_<section>` for statutes (`sox_1514a`, `air21_42121`), `<short>_<rule>` for procedural rules (`frcp_rule_17`), `<plaintiff>_v_<defendant>_<year>` for case law.
- **`version`** is free-form — calendar year, amendment cycle, edition. Not semver; legal corpora are publication-date identified.
- **`type`** must be a leaf concept name from the `SourceTaxonomy` ontology (`Language`, `UsFederalStatute`, `Regulation`, `ProceduralRule`, `CaseLaw`, …). Unknown types fail closed at startup.
- **`url`** is the authoritative source URL. For US federal statutes that means GPO-authenticated PDFs on `govinfo.gov` per Bluebook §18; for other sources, the standard cites the canonical edition. Secondary republications (Cornell LII for statutes, justia.com for cases) are not authoritative — encode the canonical source.

The reference example is the WordNet entry:

```toml
[sources.english_wordnet]
version = "2025"
type    = "Language"
url     = "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz"
```

## The lock — `praxis.lock`

Two layers of pinning:

- **`[hashes]`** — `<name>@<version> = "<sha256>"`. The hash of the bytes praxis expects on disk at `local_path()`. `LockManifestAgreement` verifies every manifest entry has a matching hash and that the local file (when present) matches.
- **`[canonical_text."<name>@<version>"]`** *(optional, source-specific)* — for sources whose authoritative format praxis cannot yet read end-to-end (PDFs awaiting M4.γ), a hand-transcribed plain-text approximation lives under `data/canonical_text/`, with `sha256` + `provenance` flags. `provenance = "training_reconstructed_<date>"` marks the file as a transcription pending verification against the authoritative source; `provenance = "verified"` marks it as a fetched-and-confirmed copy. This is an explicit, machine-readable record of the gap — not a workaround.

The `[structural."<name>@<version>"]` block is the codegen input — see [Build an Ontology from a Paper](build-ontology-from-paper.md) for the declarative path; the source-driven path that consumes it directly is forward work blocked on the PDF loader.

## The CLI — `pr4xis update`

The CLI's data-provisioning surface. All flags work the same for every registered source.

### `pr4xis update`

Fetches every registered source, verifies bytes against `praxis.lock`, writes verified output to `local_path()`. Runs through every entry regardless of per-entry failures so you get a full report. Re-running after a successful fetch short-circuits via local re-verification, so invocations are idempotent.

```bash
pr4xis update
```

Output is one line per source:

```text
  [ok]      english_wordnet: already verified
  [fetched] english_wordnet: 47 MB written to crates/domains/data/wordnet/...
  [fail]    foo_source: verification failed — sha256 mismatch
```

### `pr4xis update <name>`

Single dataset by registered name. Useful when only one source needs refreshing.

```bash
pr4xis update english_wordnet
```

Unknown names fail fast: `pr4xis update: unknown dataset: foo`.

### `pr4xis update --check`

Read-only mode: verify the current on-disk state against the lock without touching the network. `--check` always wins over `--force` — `pr4xis update --check --force` ignores `force` and only verifies. Useful in CI air-gapped steps and pre-commit hooks.

```bash
pr4xis update --check
```

### `pr4xis update --force`

Re-fetch even when a valid local copy already exists. Useful when the upstream URL has new content under the same version label (unusual for legal corpora, common for development manifests).

```bash
pr4xis update english_wordnet --force
```

### `pr4xis update --list`

Print every registered source with its current state. Includes the schema-derived information — version, taxonomy kind, content-type, on-disk path:

```text
Registered datasets:
  english_wordnet@2025 [Language] An open lexical database of English.
    remote: https://github.com/globalwordnet/english-wordnet/releases/...
    local:  crates/domains/data/lexicons/languages/english_wordnet/...
    content-type: XmlLmf
  sox_1514a@2002 [UsFederalStatute] 18 U.S.C. § 1514A — ...
    remote: https://www.govinfo.gov/.../sec1514A.pdf
    local:  crates/domains/data/legal/statutes/us_federal/sox_1514a/...
    content-type: Plaintext
  …
```

### `pr4xis update --offline`

Refuse to touch the network. Local files are still verified; an absent file reports as `MissingAndOffline` rather than fetching. Useful for air-gapped builds.

```bash
pr4xis update --offline
```

Flag precedence: `--check` (read-only) always wins over `--force`. `--offline` blocks network access; if a local file exists it is still verified, and verification failure is reported as `VerificationFailed` (not `MissingAndOffline`, which is reserved for actually-absent files).

### Planned: `pr4xis source add / remove / list` (M6)

A separate `source` subcommand for *mutating* the registry — adding a new entry, removing an existing one, or listing without going through `update` — is task **M6** in the milestone plan, not yet implemented. Today the registry is read-only from the CLI's perspective; `pr4xis update --list` prints it but cannot edit it, and new entries are added by hand-editing `praxis.toml`. When M6 lands, hand edits will remain valid (the file is the source of truth either way) and `pr4xis source add <name> --version <v> --type <leaf> --url <url>` will automate the common case.

## Adding a new source

End-to-end. The current workflow is hand-edit-then-verify; mutating subcommands (`pr4xis source add` / `remove` / `list`) are planned as **M6 — registry CLI mutators** and not yet implemented. Manual edits are the canonical path until then.

1. **Choose a leaf in `SourceTaxonomy`** under `crates/domains/src/formal/meta/source_taxonomy/ontology.rs`. If no existing leaf fits the jurisdictional or genre specificity of your source, **add a new leaf first** — the taxonomy is closed-world and unknown types fail registration at startup.
2. **Append a `[sources.<name>]` block to `praxis.toml`** with the four required fields. Use the authoritative URL. (M6 will let `pr4xis source add <name> --version <v> --type <leaf> --url <url>` do this; today it's a hand edit.)
3. **Compute the sha256** of the bytes you expect on disk and add the entry to `praxis.lock`'s `[hashes]` block.
4. **Run `pr4xis update <name>`** to fetch and verify. The CLI writes the verified bytes to `local_path()`. Subsequent runs are no-ops unless you pass `--force`.
5. **Verify with `cargo test`** — the data-provisioning ontology runs `LockManifestAgreement` (along with seven other axioms) over the registered set; any drift fails.

Note that the runtime registry is `OnceLock`-cached per process, so a running process only sees entries present at first load. After hand-editing `praxis.toml` you need to restart any long-running praxis process to pick up the new entry.

## What's automated end-to-end today

The reference instance is **WordNet** (`sources.english_wordnet`): registered → manifest+lock pinned → `pr4xis update` fetches `.xml.gz` from GitHub Releases → decompresses → writes `.xml` → verifies → engine consumes via build-time codegen. Every step is machine-driven and reproducible.

The two registered statutes (SOX § 1514A, AIR21 § 42121) point at GPO-authenticated PDFs on govinfo.gov and consume the **M4.γ PDF loader** (shipped):

- The PDF byte-stream reader (`crates/domains/src/social/software/binary/pdf/reader.rs`) decodes `%PDF-N.M` files into a typed `PdfDocument` per ISO 32000-2:2020.
- The content-stream interpreter (`content_stream.rs`) walks PostScript operator sequences and emits `TextShowEvent`s and `GraphicsEvent`s per §7.8.2 + §9.4.
- The font + encoding pipeline (`font.rs`) maps glyph bytes to Unicode via `WinAnsiEncoding` / `Identity-H` / typed-`Unsupported` for everything else, per §9.10.2.
- The image-flagging walker (`flagged.rs`) emits a `Vec<FlaggedContent>` for any non-text content per `feedback_pdf_text_only_until_image_understanding`.
- The extraction pipeline (`extract.rs`) composes the above and supports Bluebook §3.3.4 section-boundary slicing.
- The build-time codegen (`crates/domains/build.rs` + `build_helpers/extract_pdf.rs`) runs the extractor for every registered statute and emits a typed `PdfBuildExtraction` const into each statute's codegen module. Downstream code pattern-matches the variant — `Extracted { text, bytes_hash }`, `NotOnDisk`, `ParseFailed`, `Encrypted`, or `UnsupportedContentType`.

`pr4xis update sox_1514a` today: fetches the GPO PDF from the manifest URL, the build script's extractor materializes the text, and the codegen module emits `PDF_EXTRACTION = Extracted { text, bytes_hash }`. The audit modules (`sox_1514a/canonical_audit.rs`, `air21_42121/canonical_audit.rs`) consume the typed const directly.

If a PDF hasn't been fetched yet, `PDF_EXTRACTION = NotOnDisk` — content-specific audit assertions gate on `PDF_EXTRACTION.is_extracted()` and the typed state is the report. No `Option<&str>`, no hand-transcribed approximations.

## Related

- [Build an Ontology from a Paper](build-ontology-from-paper.md) — the declarative authoring path (the `ontology!` macro), parallel to source-driven ingestion
- [Architecture](../understand/architecture.md) — where data-provisioning sits in the engine stack
- [Glossary](../reference/glossary.md) — `Manifest`, `Lock`, `Registered source`, `SourceTaxonomy`, `Data provisioning`

---

- **Document date:** 2026-05-16
