# M4.δ.7.a — Large-title codegen refactor design

> Captured 2026-05-27 after rolling Title 42 + Title 5 back when the
> aggregate static blew rustc's compile-time memory ceiling.

## The constraint

`crates/domains/build.rs::write_usc_corpus_codegen` materializes every
registered `UsCodeTitle` source's USLM XML into a single
`OUT_DIR/usc_corpus_codegen.rs` containing two large statics:

```rust
pub static CODEGEN_DATA: pr4xis::codegen_data::CodegenData<UsCode> =
    CodegenData { entity_count: N, entity_ids: &[...], ... };
pub static USC_SECTION_AUX:
    &[crate::social::software::markup::xml::uslm::corpus::UscSectionAux] = &[
        UscSectionAux { urn: "...", subdivisions: &[...], relations: &[...] },
        ...
    ];
```

For Titles 15+18+28+29+49 (~7,400 sections, ~85 MB combined XML), the
generated Rust source is large enough that rustc OOM-kills (SIGKILL
signal: 9) during the pr4xis-domains test-binary compile/link step.
Adding Title 42 (113 MB, ~3,500 sections) or Title 5 (19 MB) over this
baseline pushes well past the ceiling.

Empirically observed: any aggregate that exceeds ~85 MB of input XML
overruns rustc's available memory on a 16-32 GB workstation.

## The praxis-way fix

Stop generating Rust source at build time; load XMLs at runtime via
the existing `read_uslm_title` parser. Mirrors the WordNet pattern —
[`English::cached`] reads 89 MB of WordNet XML on first call,
amortizes the parse over process lifetime, owns the data.

[`English::cached`]: ../crates/domains/src/cognitive/linguistics/english/ontology.rs

## Concrete file changes

### 1. `crates/domains/src/social/software/markup/xml/uslm/corpus/mod.rs`

Add a new constructor `UsCode::from_uslm_titles_owned`:

```rust
/// Runtime constructor: assemble a [`UsCode`] from parsed
/// [`UsCodeTitle`] instances. Used by [`loaded()`] to materialize
/// the canonical corpus from on-disk USLM XML at first access,
/// replacing the build-time codegen aggregate (M4.δ.7.a).
///
/// `Box::leak` converts owned [`String`]s + [`Vec`]s to `&'static`
/// so the resulting [`UscSection.subdivisions`] / `.relations` slices
/// satisfy the existing `&'static` API contract. The leaks persist
/// for process lifetime, same as the [`OnceLock`]-cached singleton —
/// equivalent to build-time-emitted statics.
pub fn from_uslm_titles_owned(titles: Vec<UsCodeTitle>) -> Self {
    let mut sections = Vec::new();
    let mut by_urn = HashMap::new();
    for title in titles {
        for section in title.sections {
            let urn_str: &'static str =
                Box::leak(section.identifier.clone().into_boxed_str());
            let urn = Identifier::from_codegen_static(
                IdentifierFormatConcept::UslmUrn,
                urn_str,
            );
            let heading = section.heading.clone();
            let text = combine_section_text(&section);
            let (sub_vec, rel_vec) =
                walk_subdivisions_to_static(&section.children, urn_str);
            let subdivisions: &'static [UscSubdivision] =
                Box::leak(sub_vec.into_boxed_slice());
            let relations: &'static [UscComposesEdge] =
                Box::leak(rel_vec.into_boxed_slice());
            by_urn.insert(section.identifier.clone(), sections.len());
            sections.push(UscSection {
                urn,
                heading,
                text,
                subdivisions,
                relations,
            });
        }
    }
    Self { sections, by_urn }
}
```

Helpers `combine_section_text` and `walk_subdivisions_to_static` mirror
the codegen's [`extract_sections`] text-accumulation + subdivision-walk
logic. Reference: `crates/pr4xis/src/codegen/usc_corpus.rs:184-450`.

Change `loaded()`:

```rust
pub fn loaded() -> &'static UsCode {
    static INSTANCE: OnceLock<UsCode> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        let mut titles = Vec::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
                continue;
            }
            let path = format!(
                "{}/{}",
                env!("CARGO_MANIFEST_DIR")
                    .trim_end_matches("crates/domains"),
                entry.local_path()
            );
            let Ok(xml) = std::fs::read_to_string(&path) else {
                continue; // Skip titles whose XML isn't on disk.
            };
            match read_uslm_title(&xml) {
                Ok(title) => titles.push(title),
                Err(e) => panic!("Title {} parse failed: {e}", entry.name),
            }
        }
        UsCode::from_uslm_titles_owned(titles)
    })
}
```

### 2. `crates/domains/build.rs`

Replace `write_usc_corpus_codegen`'s body with a 5-line stub:

```rust
fn write_usc_corpus_codegen(
    _workspace_root: &std::path::Path,
    _manifest: &RawManifest,
    _sorted_names: &[String],
    out_dir: &std::path::Path,
) {
    let stub = "// USC corpus codegen retired post-M4.δ.7.a. \
                The runtime constructor `UsCode::from_uslm_titles_owned` \
                in social/software/markup/xml/uslm/corpus/mod.rs reads + \
                parses registered title XMLs at first access. This file \
                is kept as an empty stub for the include!() in corpus/mod.rs \
                during the transition; the include can be deleted once \
                downstream consumers no longer reference full_corpus.\n\
                pub static CODEGEN_DATA: pr4xis::codegen_data::CodegenData<\
                crate::social::software::markup::xml::uslm::corpus::UsCode> = \
                pr4xis::codegen_data::CodegenData { \
                entity_count: 0, entity_ids: &[], entity_kind: &[], \
                entity_labels: &[], entity_defs: &[], word_index: &[], \
                taxonomy: &[], mereology: &[], opposition: &[], \
                equivalence: &[], causation: &[], references: &[] };\n\
                pub static USC_SECTION_AUX: \
                &[crate::social::software::markup::xml::uslm::corpus::UscSectionAux] \
                = &[];\n";
    std::fs::write(out_dir.join("usc_corpus_codegen.rs"), stub)
        .expect("write usc corpus stub");
}
```

The full corpus loads at runtime; the static is kept as a stub for the
`include!` in `corpus/mod.rs` until that include can be removed too.

### 3. Verification path

1. After landing the refactor, run the full pr4xis-domains test suite:
   `cargo test -p pr4xis-domains --lib`
   Expected: 6,310 / 6,310 tests pass; first-access timing for
   `cached_english` + `UsCode::loaded()` together adds ~5-8 seconds
   to the test suite once-per-process. Subsequent test calls are O(1).
2. Add Title 42 + Title 5 to praxis.toml + praxis.lock per the
   templates already in place.
3. Run the full Title 42 + Title 5 test suites (parallel to
   Title 15 + 18 + 28 + 29 + 49).
4. Confirm xmlconf still 100% conformant.
5. Run the corpus-wide gap audit; add missing lexicon entries.

### 4. Risks + mitigations

| risk | mitigation |
|---|---|
| Box::leak duplicates of the same URN waste memory | acceptable for process-lifetime data; can be deduplicated via a `HashSet<String>` interner in a follow-up |
| Text-accumulation between codegen and runtime diverges | the test `section_bodies_concatenate_chapeau_and_content_text` in `crates/pr4xis/src/codegen/usc_corpus.rs` pins the codegen's behavior; mirror it exactly in `combine_section_text` |
| `UsCode::sample()` depends on the static `CodegenData` shape | keep `from_codegen` and `from_codegen_with_aux` constructors for sample/test use; only `loaded()` switches to the runtime path |
| First-access startup time | ~3-5 seconds for the 5 registered titles; cheap on subsequent process restarts (page cache); acceptable per the WordNet precedent |
| Downstream consumers of `full_corpus::CODEGEN_DATA` directly | grep for `full_corpus::` / `CODEGEN_DATA` references; should all be in `corpus/mod.rs`'s `loaded()` |

## Why this isn't done in the present commit

This is a multi-hour focused refactor that needs:
- A dedicated session to land cleanly
- Per-step test runs (each ~50s) to catch regressions early
- Careful Box::leak lifetime annotations and clippy compliance
- Verification that the `from_codegen` API stays compatible with `UsCode::sample()` and test fixtures

The intermediate state (some titles registered, M4.δ.7-10 deferred,
M4.δ.7.a tracked) is internally consistent — every registered title
loads correctly through the existing codegen path; the corpus is
incomplete but usable. Pushing this refactor at session-end risks
landing a half-working codegen path that breaks the 6,310 green tests.

## Citation

- M4.ε.3 (`UsCode` runtime + from_codegen functor) — shipped
- M4.ε.5 (Layer 3 resolver queries &UsCode + corpus-wide gap audit) —
  shipped
- M5.D.1 (Register usc_title_28 USLM) — shipped
- M4.δ.7 (Title 42 registration) — blocked on this
- M4.δ.8 (Title 5), M4.δ.9 (Title 50), M4.δ.10 (Title 1) — also
  blocked on this

---

- **Document date**: 2026-05-27
- **Triggered by**: Title 42 + Title 5 rustc OOM-kill on test-binary
  compile
- **Status**: design captured; implementation pending dedicated session
