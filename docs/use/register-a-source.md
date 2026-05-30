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
- **`url`** is the authoritative source URL. For US federal statutes that means **LRC USLM XML** on `uscode.house.gov/download/releasepoints/...` per 1 U.S.C. § 204 (the Office of the Law Revision Counsel is the statutory codifier; USLM is its published XML form). For case law it means the issuing court's published opinion (typically PDF). For other sources, the standard cites the canonical edition. Secondary republications (Cornell LII for statutes, justia.com for cases) are not authoritative — encode the canonical source.

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
- **`[canonical_text."<name>@<version>"]`** *(optional, source-specific)* — for sources whose authoritative format praxis cannot yet read end-to-end (e.g. case-law PDFs whose figures or non-text content require future image-understanding work), a hand-transcribed plain-text approximation lives under `data/canonical_text/`, with `sha256` + `provenance` flags. `provenance = "training_reconstructed_<date>"` marks the file as a transcription pending verification against the authoritative source; `provenance = "verified"` marks it as a fetched-and-confirmed copy. This is an explicit, machine-readable record of the gap — not a workaround. **Statutes don't need this**: USLM XML reads end-to-end through the loaded W3C XML 1.0 parser + USLM lens.

The `[structural."<name>@<version>"]` block is a legacy codegen input retained for sources whose loader hasn't yet been wired into the build-time codegen path. For statutes, M4.δ.2.b wired the USLM XML loader into `build.rs`, so the source-driven path consumes USLM XML directly — `[structural.*]` is being deleted for statutes per task M4.δ.2.e. See [Build an Ontology from a Paper](build-ontology-from-paper.md) for the declarative authoring path that's parallel to source-driven ingestion.

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
  usc_title_18@pl-119-90 [UsCodeTitle] Crimes and Criminal Procedure.
    remote: https://uscode.house.gov/download/releasepoints/us/pl/119/90/xml_usc18@119-90.zip
    local:  crates/domains/data/legal/uscode/...
    content-type: UslmXml
  …
```

Individual sections (18 U.S.C. § 1514A SOX, 49 U.S.C. § 42121 AIR21, etc.) are **not separate datasets** — they are URN-addressable slices of the registered title. To resolve one at runtime: `UsCode::loaded().section_by_urn("/us/usc/t18/s1514A")`.

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

The registered US statutes are **whole U.S. Code titles in USLM XML** published by the LRC (Office of the Law Revision Counsel) per 1 U.S.C. § 204 at `uscode.house.gov/download/releasepoints/...`. `usc_title_18` (Crimes), `usc_title_49` (Transportation), and `usc_title_28` (Judiciary, including the Federal Rules of Civil Procedure / Evidence / Appellate Procedure / Bankruptcy Procedure as appendices) are registered at release point `pl-119-90`. Individual sections like 18 U.S.C. § 1514A (Sarbanes–Oxley § 806) and 49 U.S.C. § 42121 (AIR21) are **URN slices of the registered title**, not separate sources — `UsCode::loaded().section_by_urn("/us/usc/t18/s1514A")` returns the typed `Statute` via the bytes ⇄ Statute composed lens (M4.λ.3.b, shipped).

The end-to-end pipeline for a registered USLM XML title:

- The bytes are fetched as a `.zip` from `uscode.house.gov`, verified against `praxis.lock`, unzipped, and the `usc<N>@<release>.xml` file lands at `local_path()`.
- The bundled W3C XML 1.0 parser (`crates/domains/src/social/software/markup/xml/parser/`) reads it into a typed `XmlDocument` — the same parser the M5.ω audit confirmed is 100% xmlconf-conformant.
- The USLM ontology (`uslm/`) types the document tree, with every concept's identity grounded in the loaded LRC USLM XSD (M4.ε.5.a — XSD-grounded USLM ontology). Container kinds, subdivision kinds, additional containers, and the codegen tokenizer config all derive from the XSD's `substitutionGroup="level"` membership (Batches D + E of M5.ω).
- The `UslmStatuteLens` (M4.λ.3.b) projects each `<section>` to a typed `Statute` value with citations, valence, obligations, evidence requirements, and proof standards — all loaded, not hand-coded.
- The build-time codegen (`crates/domains/build.rs`) materializes per-title runtime modules at `us_code::title_N::*`; downstream code looks up sections by URN via the `UsCode` corpus loader (M4.ε.3).

`pr4xis update usc_title_18` today: fetches the LRC USLM XML zip, unzips, verifies, and the build script's USLM lens projects every section in the title to a typed `Statute`. The audit modules consume the typed values directly — no `Option<&str>`, no PDF text extraction, no hand-transcribed approximations.

**PDF is for case law, not statutes.** The M4.γ PDF loader (shipped) is the path for court opinions (e.g. CourtListener / PACER bulk PDFs). The legal-evidence pipeline reads statutes from USLM XML and case law from PDFs — two distinct loaders, two distinct authoritative formats.

## Related

- [Build an Ontology from a Paper](build-ontology-from-paper.md) — the declarative authoring path (the `ontology!` macro), parallel to source-driven ingestion
- [Architecture](../understand/architecture.md) — where data-provisioning sits in the engine stack
- [Glossary](../reference/glossary.md) — `Manifest`, `Lock`, `Registered source`, `SourceTaxonomy`, `Data provisioning`

---

- **Document date:** 2026-05-16
