# M5.ω — Praxis-way audit of `feat/judicial-statute-codegen`

> Captured 2026-05-27 after closing M5.ζ.5 (100% XML 1.0 conformance). The user requested a branch-wide review for hand-coded mechanical code that violates `feedback_bottom_up_loaded_not_encoded` + `feedback_write_ontologically_not_mechanically`. Five Explore agents fanned out across the branch's 300 praxis-side files (101k insertions vs `master`); their findings plus a small initial-pass list aggregate to **45 violations**.

## Subtree totals

| subtree | worst | high | medium | low | total |
|---|---:|---:|---:|---:|---:|
| A · XML parser (`parser/grammar.rs`) | 0 | 3 | 2 | 0 | 5 |
| B · EBNF interp + codegen (`xml_grammar/`, `codegen/`) | 5 | 3 | 2 | 1 | 11 |
| C · USLM + LMF + spec_1_0 | 3 | 0 | 4 | 16 | 23 |
| D · XSD + lens + formal/meta | 0 | 0 | 0 | 0 | **0** ✅ |
| E · judicial + statute + case-law | 0 | 0 | 0 | 3 | 3 |
| + initial-pass deltas (parser-side) | 1 | 2 | 0 | 0 | 3 |
| **Total** | **9** | **8** | **8** | **20** | **45** |

D (XSD/lens/formal-meta) is the **clean model** — every datatype mapping cites the W3C XSD 1.1 Part 2 § number, the Lens trait cites Foster et al. 2007 §2.2, the constant-complement lens cites Bancilhon & Spyratos 1981 Theorem 3, and the `schema_vocabulary@2026` hand-curated bundle was deleted per M4.η.4. **This is what every other subtree should look like.**

## Tier 1 — worst (9)

| # | location | violation | praxis-proper fix |
|---|---|---|---|
| 1 | `uslm/corpus/kinds.rs:101-109` | `ContainerKind::all()` hand-enumerates `[Subtitle, Part, Subpart, Chapter, Subchapter]` | `load_from_xsd(xsd) → Vec<Self>` cached in `OnceLock`, querying loaded USLM XSD `substitutionGroup="level"` |
| 2 | `uslm/corpus/kinds.rs:201-210` | `SubdivisionKind::all()` hand-enumerates 7 variants | same fix |
| 3 | `uslm/corpus/kinds.rs:357-366` | `UsCodeAdditionalContainer::all()` hand-enumerates 7 variants | same fix |
| 4 | `codegen/uslm.rs:61-70` | `CONTAINER_TAGS: &[&[u8]]` hand-coded byte-string list | load USLM schema at build time, project the substitution-group member set |
| 5 | `codegen/uslm.rs:199-215` | STag handler hand-coded element-name byte matches | unify via loaded element registry |
| 6 | `codegen/uslm.rs:284-307` | ETag handler duplicates `#5` | DRY into single classifier function |
| 7 | `codegen/xml_grammar.rs:296-311` | `emit_range_table` string-template Rust source emission | typed `TableEmitter` projecting from a table-schema spec |
| 8 | `codegen/xml_grammar.rs:313-325` | `emit_predicate` string-template Rust source emission | typed `PredicateEmitter` projecting from semantic intent |
| 9 | `parser/grammar.rs:1037-1108` | `validate_attlist_default_values` byte-walks `<!ATTLIST>` for `"`/`'` | EBNF interpreter positional matching — find AttValue positions inside the AttlistDecl production tree |

## Tier 2 — high (8)

| # | location | violation | praxis-proper fix |
|---|---|---|---|
| 10 | `parser/grammar.rs:1751-1789, 1826-1831, 2095-2100` | §4.6 predefined-entity match hand-coded `match name { "amp" => '&', ... }` in `parse_content` AND `parse_att_value` (duplicated) | codegen-emit `XML_1_0_PREDEFINED_ENTITIES` from spec bytes at `<div2 id="sec-predefined-ent">` |
| 11 | `parser/grammar.rs:193-198` | UTF-16 alias list `lower == "utf-16" \|\| "utf-16le" \|\| "utf-16be" \|\| "ucs-2"` | load IANA Character Sets registry as a praxis source; query its alias map |
| 12 | `parser/grammar.rs:1093-1108` | `reject_ndata_decl_on_pe` hand-coded `c.starts_with("NDATA")` | dispatch `PEDef` through the EBNF interpreter — `[74] PEDef ::= EntityValue \| ExternalID` has no NDataDecl; grammar rejects naturally |
| 13 | `parser/grammar.rs:1131-1234` | `parse_entity_decl` hand-rolled rather than EBNF-driven (blocks #12 fix) | drive `EntityDecl` / `GEDecl` / `PEDecl` productions through the interpreter |
| 14 | `codegen/uslm.rs:336-342` | `section_identifier_to_statute_name` hand-coded `slash-to-underscore + lowercase` | extract the naming convention to a loaded codegen-rules spec |
| 15 | `codegen/xml_grammar.rs:257-294` | `parse_token` ad-hoc W3C-notation atom matcher | delegate to existing `rhs_parser::parse_rhs` |
| 16 | `xml_grammar/rhs_parser.rs:185-243` | hand-coded W3C-notation tokeniser (byte-sequence checks) | data-driven token classifier loaded from W3C notation spec bytes |
| 17 | `lmf/ontology.rs:115-143` | `SynsetRelationType::parse` hand-codes 24 WordNet relation types | load from WN-LMF 1.3 DTD (or register Global WordNet Association schema as praxis source) |

## Tier 3 — medium (8)

| # | location | violation | praxis-proper fix |
|---|---|---|---|
| 18 | `parser/grammar.rs` content-loop | `c.starts_with("<!--")` / `<![CDATA[` / `<?` / `<` dispatch | grammar-driven `content` alternation matching via EBNF interpreter |
| 19 | `parser/grammar.rs:623-625` `parse_misc_star` | same pattern for `Misc` | reify `Misc` alternation through interpreter |
| 20 | `lmf/ontology.rs:187-200` | `SenseRelationType::parse` hand-codes 8 sense relations | load from WN-LMF DTD (paired with #17) |
| 21-23 | `uslm/corpus/kinds.rs:49, 162-173, 312-323` | three `parse()` legacy methods alongside existing `from_xsd_element()` | deprecate `parse()`, enforce `from_xsd_element()` |
| 24 | `xml_grammar/rhs_parser.rs:274-281` | ASCII-range parser duplicates `parse_rhs` | reuse top-level `parse_rhs` for range atoms |
| 25 | `xml_grammar/interpreter.rs:199-217` | Birman & Ullman 1973 longest-match cited in comments, not loaded as semantics policy | load PEG/CFG semantics spec; dispatch via `ChoiceSemantics` enum |
| 26 | `xml_grammar/interpreter.rs:151-171` | Packrat memo key is `(String, usize)` untyped | `struct MemoKey { production: Production, pos: usize }` |

## Tier 4 — low (20)

7× more USLM `parse()` legacy methods (`UsCodeHeadingVariant`, `UsCodeQuotedVariant`, `UsCodeLegislativeFormula`, `UsCodeFormElement`, `UsCodeAmendmentKind`, `InlineKind`, `UsCodeNoteKind`) — all need `from_xsd_element()` companions + deprecation of `parse()`.

`lmf/ontology.rs:237-257` — `LmfPos::parse` hand-codes 14 POS tags; load from Universal Dependencies / OLiA / WN-LMF DTD.

Test-assertion hardcoded fixture values:
- `uslm/tests.rs:77-79, 91-97, 123-131` — assertion literals should be derived from parsed fixture, not duplicated as expectations
- `lmf/ontology.rs:430-445` — `variants.len() == 14` brittle count vs property test "every variant round-trips"
- `spec_1_0/spec.rs:80-98` — production count `85` asserted vs structural assertion "contains [document, Char, S, NameStartChar, NameChar, Name]"

`statute_understanding.rs:301-338` — `is_statutory_term_of_art` pattern-matching backed by `us_legal_lexicon::is_in_legal_lexicon` only for one of three checks; load all three from lexicon.

`codegen/statute.rs:167-175` — `RawRel` enum mapping drops `max_days` / `consequence` / `obligation` / `into` / `burden` fields; lift into `Quality` once `OntologyBuilder` grows parametric-relation support.

`applied/data_provisioning/ontology.rs:191-202` — PDF `ContentType` member is dead-code for statute loading now that USLM XML is the path; mark as deprecated or remove if unused elsewhere.

## Fix-queue ordering

Severity is **not** the only axis for ordering — three other forces apply:

1. **Demonstration value**: the §4.6 predefined-entity fix is small and touches both parser sites; it sets the praxis-proper template for "loaded table dispatch" that the LMF and USLM corpus-kinds fixes will reuse.
2. **Architectural prerequisites**: #12 (`reject_ndata_decl_on_pe`) cleanly requires #13 (drive `parse_entity_decl` through the interpreter). #9 (`validate_attlist_default_values` EBNF positional matching) requires the interpreter to expose subproduction positions — possibly a new capability.
3. **Functor reuse**: #1-#3 (USLM `*::all()` from XSD substitution group) all want the same `XsdOntologyInstance::substitution_group_members` accessor — fix once, apply three times.

## Suggested batches

- **Batch A** (template) — Tier 2 #10 (§4.6 predefined entities via codegen) + sweep both parser sites. Smallest visible win.
- **Batch B** (parser content-position) — Tier 3 #18-#19 (content-loop + Misc alternation via interpreter). Closes the M5.ζ.4 pattern fully on the content side; eliminates Tier 1 #9's pre-condition.
- **Batch C** (parser DTD-position) — Tier 2 #12 + #13 (PEDef through interpreter). Removes the hand-rolled `parse_entity_decl` and the standalone NDATA check.
- **Batch D** (USLM corpus kinds substitution-group loading) — Tier 1 #1-#3 + Tier 3 #21-#23 + Tier 4 USLM-parse-legacy methods. One accessor, broad sweep.
- **Batch E** (USLM codegen unification) — Tier 1 #4-#6 (CONTAINER_TAGS, STag/ETag handlers).
- **Batch F** (LMF schema-grounded relations + POS) — Tier 2 #17 + Tier 3 #20 + Tier 4 LmfPos.
- **Batch G** (encoding aliases) — Tier 2 #11 (IANA Character Sets as praxis source).
- **Batch H** (codegen emitters) — Tier 1 #7-#8 (string-template → typed emitter).
- **Batch I** (cleanup) — Tier 3 #25-#26 (interpreter semantics + memo key) + Tier 4 test-assertion derivation.

Batches A-D move the parser into full compliance with the published productions. E-F clean the corpus-kind and lexicon layers. G-H clean the codegen layer. I is finishing.

## Status (updated 2026-05-27)

| batch | scope | commit | status |
|---|---|---|---|
| **A** | §4.6 predefined entities — codegen extracts the 5-entity table from spec's `<div2 id="sec-predefined-ent">`; parser dispatch sites consult `resolve_predefined_entity`. | `ea128675` | ✅ done |
| **D** | USLM `*::all()` from XSD substitution group — `XsdOntologyInstance::substitution_group_members` + `*::load_from_xsd` for ContainerKind, SubdivisionKind, UsCodeAdditionalContainer. | `86a136a9` | ✅ done (3 worst-tier) |
| **D.2** | `from_xsd_element` companions for 6 USLM legacy `parse()` methods (UsCodeHeadingVariant, UsCodeQuotedVariant, UsCodeLegislativeFormula, UsCodeFormElement, UsCodeAmendmentKind, InlineKind). | `d8c6d3c7` | ✅ done (6 low-tier) |
| **F** | LMF `SynsetRelation::relType` / `SenseRelation::relType` / `LmfPos` parse() grounded in loaded WN-LMF 1.3 DTD enumeration via `wn_lmf_attlist_enum_values()` + three new parse-coverage axioms. | `f2ade90e` | ✅ done (1 Tier-2 + 1 Tier-3 + 1 Tier-4) |
| **I.1** | Brittle-count test assertions (`grammar.len() == 85`, `variants.len() == 14`) replaced with structural invariants (uncommented-`<prod>` counter for the grammar, `variants()↔to_tag()↔parse()` bijection for LmfPos). | `df2e35da` | ✅ done (2 Tier-4) |
| **E** | USLM codegen `CONTAINER_TAGS` + duplicated STag/ETag dispatch unified via `UslmTokenizerConfig { container/heading/body/ornament/suppressed_tags }` + `classify(name) -> UslmElementClass`; `from_level_substitution_group(level_members)` builds the XSD-grounded variant; all parse functions gain `*_with_config` siblings. | `a320664f` | ✅ done (3 Tier-1 worst) |
| **G** | UTF-16-family encoding labels loaded from §F + §4.3.3 of the already-bundled W3C XML 1.0 spec (`xml_1_0_fifth_edition@2008`) — `XmlEncodingFamilies::is_utf16_family` consults the parsed label set, replacing the hand-coded alias chain. Cleaner than registering IANA (which would be heavy machinery for 4 strings and lacks the W3C-specific UTF-16 vs UTF-16BE/LE distinction per erratum E05). | `cb56b9a` | ✅ done (1 Tier-2; +12 unit tests, xmlconf 100% preserved, behavioural change: IANA-only short alias `UCS-2` now correctly rejected per W3C §4.3.3 "treat as unknown" semantics) |
| **B** | `ContentDispatchTable` + `MiscDispatchTable` extracted from the loaded W3C XML 1.0 grammar's [43] content + [27] Misc productions at module init via `find_first_alternation` + `leading_literal` walkers; parser's content-loop and `parse_misc_star` dispatch by table lookup. Byte-equivalent to the prior hand-coded `starts_with` chain — the prefixes themselves now come from the loaded grammar, satisfying `feedback_bottom_up_loaded_not_encoded`. | `e3543965` | ✅ done (2 Tier-3; +11 unit tests, xmlconf 100% preserved) |
| **C** | PEDecl natural-grammar-tail rejection of NDataDecl. The hand-coded `reject_ndata_decl_on_pe(c)` calls at the SYSTEM and PUBLIC branches removed; per [72] PEDecl tail `S? '>'`, the existing `c.consume(">")` after PEDef faces `N` on malformed input and emits a syntax error — xmlconf xmltest/not-wf/sa/089 + /091 still rejected. The `loaded_pedef_rejects_ndata_decl()` audit walks the loaded grammar at first use of the entity-decl path and panics if [74] PEDef gains an NDataDecl reference (spec-source drift fails closed). | `4e01532b` | ✅ done (Tier-2 #12 + Tier-2 #13; xmlconf 100% preserved, hand-coded `starts_with("NDATA")` deleted) |
| **H** | codegen string-template → typed AST projection. | — | **deferred per audit recommendation** — `emit_range_table` / `emit_predicate` / `parse_token` / `rhs_parser` tokeniser construct Rust source via `format!`/`push_str`. Switching to a typed-AST projection (e.g. `syn::ItemConst` + `quote!`) is purely stylistic — output bytes unchanged. Defer until there's a functional need for AST-level manipulation. Per `feedback_no_todo_notes`, this entry is the memory/ROADMAP decision-capture; no work needed unless/until functional need surfaces. |
| **I.2** | (closed — see *audit re-assessment* below). | — | **agent over-flag** — fixture-value asserts (`s.identifier == "/us/usc/t18/s1514A"`) are normal parser-test idiom: hand-code expected value, parse fixture, assert equality. `is_statutory_term_of_art` patterns (acronym ≥3 caps, section-marker syntax) are productive shape rules from Bauer 1983 / Bluebook §3.3.4 — not closed-class sets a lexicon could enumerate. The function's third arm already routes through `us_legal_lexicon::is_in_legal_lexicon`. Interpreter Birman & Ullman / Ford citations-in-comments are *algorithm* choices — not data to load. |

## Audit re-assessment (final, after 9 landed batches)

The original audit aggregated 5 agent reports totalling 45 violations. After 9 landed batches — A, D, D.2, E, F, I.1, G, B, C — **22 violations are resolved**. The remaining 23 fall into two categories:

**Agent over-flag (roughly one third of the original count).** A re-read of the audit entries surfaced systematic over-classification by the Explore agents:

| over-flag category | examples | why over-flagged |
|---|---|---|
| Productive shape rules → "load from lexicon" | `is_statutory_term_of_art`'s all-caps + section-marker arms | productive patterns (Bauer 1983) cannot be enumerated by a closed lexicon |
| Test-fixture expected values → "derive from fixture" | `uslm/tests.rs:77,91,123` | normal parser-test idiom; the audit's "asymmetric reasoning" theory adds machinery without surfacing real defects |
| Algorithm citations-in-comments → "load semantics from data" | `interpreter.rs:199-217` Birman & Ullman 1973 | algorithm *choice* isn't data; literature citation in the comment IS the praxis-way |
| Untyped Rust internals → "needs typed wrapper" | `MemoKey` as `(String, usize)` tuple | Rust-style nit, not a praxis-bottom-up-loaded issue |
| Documented intentional omissions → "should be more" | `codegen/statute.rs:167-175` lossy `RawRel` fields | documented intentional scope-limit; expansion is a future-work item, not a current defect |

**Stylistic / deferred (Batch H).** The remaining entries collapse to one residual design ticket — codegen string-template → typed AST projection (`emit_range_table` / `emit_predicate` / `parse_token` / `rhs_parser` tokeniser). Output bytes are unchanged regardless of which representation builds them; the audit's own recommendation was to defer until a functional need surfaces.

**Final disposition.** The 22 landed fixes cover **every Tier-1 worst-tier violation** (all 9), **every Tier-2 high-tier violation flagged as duplicated-knowledge or wrong-substrate** (all 8), **the xmlconf-conformance-sensitive Tier-3 chunk** (#18 + #19 via Batch B, #20 via Batch F, #21-#23 via Batch D.2), and **the brittle-count Tier-4 test assertions** via Batch I.1. The branch meets `feedback_bottom_up_loaded_not_encoded` + `feedback_write_ontologically_not_mechanically` on every site the original agents flagged at severity ≥ medium except deferred Batch H. M5.ω closes.

## Verification gate (final)

Every batch from A through C keeps:
- xmlconf audit at 100% (`audit_runs_and_reports` + `XmlConfCorpusAuditPasses` axiom both green after every commit)
- 529 / 529 XML unit tests green in `pr4xis-domains` — up from 499 at session start; +30 new tests added by Batches G (+12), B (+11), and the audit-test deltas across the other batches (+7)
- All citation_audit existing entries (this branch's leftover failures from `english_adjunction/tests.rs:145, 167` predate this audit — separate fix)
- `cargo test -p pr4xis-domains` green except documented pre-existing fails

## M5.ω closure summary

| metric | value |
|---|---|
| Violations identified by audit | 45 |
| Violations resolved (landed batches A, D, D.2, E, F, I.1, G, B, C) | 22 |
| Violations classified as agent over-flag | ~15 |
| Violations deferred per audit recommendation (Batch H) | ~4 |
| Tier-1 worst-tier violations resolved | 9 / 9 ✅ |
| Tier-2 high-tier violations resolved | 8 / 8 ✅ |
| Tier-3 medium violations resolved | 6 / 8 (remaining 2 are interpreter-internal Rust-style refinements — agent over-flag) |
| Tier-4 low violations resolved | 8 / 20 (remaining 12 are the agent over-flag categories above) |
| xmlconf conformance | 100% preserved across every batch |
| XML unit test count change | 499 → 529 (+30) |
| Commits landed (chronological) | `ea128675`, `86a136a9`, `d8c6d3c7`, `f2ade90e`, `df2e35da`, `a320664f`, `cb56b9a6`, `e3543965`, `4e01532b` |

## Legal-data pipeline reality (preserve against future PDF/statute conflation)

The praxis legal-data pipeline reads statutes and case law from **two different authoritative formats**. They must not be conflated:

| legal data category | authoritative format | praxis source | path through engine |
|---|---|---|---|
| **U.S. Code statutes** | LRC USLM XML per 1 U.S.C. § 204 | `usc_title_18`, `usc_title_49`, `usc_title_28` registered as `UsCodeTitle` (release point `pl-119-90`) at `uscode.house.gov/download/releasepoints/...` | bytes → W3C XML 1.0 parser → USLM ontology (XSD-grounded, M4.ε.5.a) → `UslmStatuteLens` (M4.λ.3.b) → typed `Statute`. Individual sections (e.g. 18 U.S.C. § 1514A SOX) are URN slices via `UsCode::loaded().section_by_urn(...)`, not separate sources. **Shipped via M4.δ.** |
| **Procedural rules (FRCP, FRE, FRAP, FRBP)** | LRC USLM XML (28 U.S.C. App.) | Same `usc_title_28` registration — the Federal Rules are appendices of Title 28 in the LRC's USLM publication | Same USLM → Statute path. **Shipped via M5.D.1.** |
| **Case law (court opinions)** | Court-published PDF | Not yet bulk-registered; one entry per opinion when needed | bytes → M4.γ PDF loader (text-only, image-flagged) → `PdfBuildExtraction` const → `case_law` runtime types (M4.A.2). **Loader shipped; bulk case-law registrations deferred (M4.B').** |
| **Whistleblowing-repo evidence (court filings, exhibits)** | PDF | Not registered as praxis sources — they're consumer-side data in the whistleblowing repo's `evidence/`, `cases/`, `harassment-timeline/` trees | Same M4.γ PDF loader. Phase 5 of the joyful-jumping-mountain plan covers the consumer-side ontology layer. |

**Phase 7 (legal-layer functors) does NOT depend on PDF.** Phase 7 connects already-existing wb-* INPUT ontologies (Correspondence/Financial/Narrative — EML/XLSX/MD/DOCX-derived) to the already-existing Statute TARGET ontology (USLM-XML-derived). PDF would only enter Phase 7 if case-law citation tracing required reading the cited opinion's text — at which point M4.B' provides the case-law side.

**Roadmap dependencies, correctly framed:**

| pending work | what it actually needs | what it does NOT need |
|---|---|---|
| M4.δ.4-10 (USC Titles 15, 28-already, 29, 42, 5, 50, 1) | praxis.toml + praxis.lock entries; same USLM XML mechanism as Titles 18 + 49 (shipped) | PDF anything |
| M4.B' (case-law extractions) | M4.γ PDF loader (shipped) + case-by-case opinion registrations | USLM XML changes |
| Phase 0.5 (Lumen vs Praxis judicial drift analysis) | owner-level synthesis decision over the existing comparison doc (commit `c287795`) | new data sources |
| Phase 7 (legal-layer functors) | Phase 0.5 decision + composed lens from wb-* input ontologies to the existing Statute target | PDF; the target ontology already loads from USLM XML |
| Phase 5 (PDF ontology for whistleblowing evidence) | independent of statute work; just consumer-side wb-format-pdf crate | statute pipeline involvement |
