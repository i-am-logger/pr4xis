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
| **B** | parser content-loop + Misc grammar-driven alternation via loaded grammar. | — | deferred (needs EBNF interpreter subproduction-positions extension) |
| **C** | PEDef-through-EBNF-interpreter (kills `reject_ndata_decl_on_pe`); requires `parse_entity_decl` refactor. | — | deferred (parser-side refactor) |
| **E** | USLM codegen `CONTAINER_TAGS` from loaded XSD; STag/ETag handler unification. | — | deferred (cross-crate: parse code is in pr4xis; XSD bytes only in pr4xis-domains — needs codegen reorganization) |
| **G** | IANA Character Sets registry for UTF-16 encoding aliases. | — | deferred (needs IANA registry registered as a new praxis source) |
| **H** | codegen string-template → typed AST projection. | — | deferred (stylistic; large refactor) |
| **I.2** | test-fixture-value derivation (uslm/tests.rs:77-79, 91-97, 123-131) + `is_statutory_term_of_art` lexicon-grounding. | — | deferred (low-priority cleanup) |

## Verification gate

Every batch keeps:
- xmlconf audit at 100% (`audit_runs_and_reports` + `XmlConfCorpusAuditPasses` axiom)
- 499/499 XML unit tests green
- All citation_audit existing entries (this branch's leftover failures from `english_adjunction/tests.rs:145, 167` predate this audit — separate fix)
- `cargo test -p pr4xis-domains` green except documented pre-existing fails
