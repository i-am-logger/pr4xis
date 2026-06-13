# Praxis-way audit — function-words `.prx` + linguistics/meta mechanical residue (2026-06-12)

Triggered by a load-path asymmetry the user spotted: **OLiA loads from a committed
`.prx.gz`; `function-words/english.xml` still `include_str!`s + live-parses XML.**
Closing that gap pulled in two adjacent concerns the user named explicitly:

1. *"all answers must be researched, literature, ontological praxis level"* — every
   decision below is grounded in cited literature (see §2, §3).
2. *"we have mechanical crap in praxis and old stuff that needs updated"* — a repo-wide
   residue sweep (§4), adversarially verified.

Produced by three orchestrated workflows (understand → design+critique → research, and a
separate verified residue audit). The earlier `mω-praxis-way-audit-2026-05-27.md` closed
the XML/XSD/USLM/LMF **codegen** track; it never covered the cognitive/linguistics
projections or the new `.prx`/functor substrate. This is new ground.

---

## STATUS (updated 2026-06-12) — branch `feat/fw-prx-and-praxis-residue`

| Batch | State | Commit |
|---|---|---|
| **FW-A** function-words `.prx` + `ClosedClassLexicon` + D-8 `local_path` | ✅ done, verified | `e41237e` |
| **FW-B** `DeterminerKind` rename + `InterjectionKind`+`Conative` + cite fixes + D-14 | ✅ done, verified | `e6ae796` |
| **D-18** derive `is_leaf` from the loaded graph (source_taxonomy + artifact_identity is_leaf/is_family) | ✅ done, verified | `df6f3ec`, `2794f8c` |
| **D-10** theme `polarity` → `Option<Polarity>` | ✅ done, verified | `df6f3ec` |
| **D-19** `ContentHashIsInjective` derives its leaves from the subtree | ✅ done, verified | `2794f8c` |
| **D-20** `DomainOrder` rank from the loaded N⊂Z⊂Q⊂R⊂C chain | ✅ done, verified | `2794f8c` |
| **D-17** XSD `DatatypeEvolution` baseline derived (magic 46/4 removed) | ✅ done, verified | `2794f8c` |
| **D-7** relation→structural-property as loaded `HasProperty` edges | ✅ done, verified | `3ae5f61` |
| **D-11** delete dead `legacy` discourse module | ⏸ ON HOLD — it is test-covered runtime code; the "superseded by discourse/ontology.rs" claim is shaky (that is a discourse *ontology*, not a runtime conversation model). Needs explicit confirm-to-delete. |
| **MORPH D-1** irregular forms LOADED from the registered AGID source | ✅ done, verified | `94d7705` |
| **MORPH D-2 / D-12** (english_rules from CatVar + the len==13 count) | ▫ pending — the SECOND morphology source (CatVar / WordNet morphosemantic, Habash & Dorr 2003). The 13 productive affix rules map less directly to CatVar's derivational word-pairs than AGID's inflections did to irregulars; a focused follow-up. |
| **D-5, D-6, D-16** (XSD datatype groups / base_type) | ▫ pending — derive group membership + base_type from the loaded §3.4 Subsumption structure; removes the `BaseTypeAgreesWithCategory` dual-source-of-truth axiom. Interrelated + needs a transitive reduction — higher-risk, do as a focused trio. |
| **D-9, D-22** (registry) | ▫ pending — `family_dir_for` per-name arms → registry data (now partly enabled by the FW-A `local_path` field); `has_decoder_for` derivation (verifier flagged the fix as partly infeasible). |
| **D-13, D-15** (linguistics) | ▫ pending — `pos_to_olia_fragments` (partial: the canonical fragment is a legit anchor, the subclass enumeration is residue); `VerbTransitivity::from_frame_id` (needs the loaded WN frame vocabulary). |
| **D-21** scattered `variants().len()==N` counts | ◑ partial — the source_taxonomy leaf-count + the XSD baseline are structural now; the remaining meta-ontology inventory-count pins are cited-fact regressions (closed enums, no external source) and lower priority. |
| **D-3, D-4** (canonical_phrase / SynsetRelationType predicates) | ⏸ PARKED with Track C — both are the same `match → GeneratorAction::Functor` projection-as-data shape; D-4 specifically feeds `wn_builder_to_owned`. Do them with the Track-C lift, not piecemeal. |
| **Track C** projection-as-data lift | ⏸ PARKED per user → tracked below. |

### Track C — tracked issue (to file)
> **Title:** Lift the LMF→praxis projections onto `apply`/FreeExtension (projection-as-data)
> **Body:** `function_words_from_lmf` (`language.rs`), `wn_builder_to_owned` (`lmf/prx.rs:552`), the `SynsetRelationType::is_taxonomy/is_mereology/is_causal` predicates (D-4), `canonical_phrase` (D-3), and `RelationProperty` (now edges, D-7 — reference only) are source→praxis-kind projections. The #87 engine-bridge landed the substrate to carry such a projection AS `.prx` data — a `GeneratorAction::Functor` interpreted by `apply` (`9e59bb0`) — with the WordNet→praxis functor already carried that way (`f920898`, `bridge.rs:193-224`). Lift the WordNet projection AND the function-words projection together onto that interpreter so there is ONE projection mechanism, not a forked second one. NOTE: the 2026-06-12 audit's own adversarial verifiers ruled `wn_builder_to_owned`/`function_words_from_lmf` *codec-floor* (closed-enum→closed-enum), not the source-vocabulary→kind residue the functor supersedes — so this is a consistency/uniformity refactor, contested as strict residue. Do both projections together or neither.

All commits are unsigned (`--no-gpg-sign`); user re-signs on YubiKey. `fa9f154` (the
synset-codec fix) was cherry-picked forward — it never made it into the merged #201.

---

## 1. The function-words → `.prx` design (Track A)

**Goal:** give `crates/domains/data/function-words/english.xml` an OLiA-style committed
`.prx` fast-load, mirroring `lexicon/olia.rs:46 reference_model()`.

### The trap the design caught (load-bearing)
The compact succinct codec **mints synthetic synset ids** (`compact.rs:490/557`,
`format!("s{i}")`). The two feature decoders match *literal* ids
(`definiteness_from_synset` keys on `fw-definite-det`…, `interjection_kind_from_synset` on
`fw-greeting`…). Archiving function-words through the compact codec would silently collapse
**every determiner → `Indefinite`** and **every interjection → `Expressive`** — compiles,
loads, passes a count-only guard. Verified independently by all three design critics.

→ **Use the rkyv `WordNetPrxEnvelope`** (preserves the raw source; original `fw-*` ids
survive via `wn_reconstruct_source`), NOT the compact codec.

### Tier choice — graph-faithful, not floor (critic-3 correction)
The synthesis's "us_legal_lexicon is also floor-tier" precedent is **factually wrong**:
`us_legal_lexicon` registers `WordNetLmfLens` at `ByteExactGraphFaithful` with a durable
`[byte_exact_signatures]` pin (`lens.rs:176-181`). That lens is *source-agnostic*, and
`english.xml` is DTD-ordered — so registering it is nearly free and gives function-words
**integrity parity with its real sibling** (durable byte-exact identity) instead of a
weaker floor blob. Decision: register the lens (graph-faithful) — it also preserves the
`fw-*` ids (the rkyv-vs-compact distinction is orthogonal to floor-vs-graph-faithful).

### Taxonomy placement (research-resolved)
Add a cited leaf **`ClosedClassLexicon` as a sibling of `Language` under `Lexicon`**,
grounded in **Quirk et al. 1985 §2.34** (open vs closed class). The `Language` leaf
(WordNet) is *open-class only*; the closed-class function-word stratum is its **disjoint
complement** → a sibling, with an `Adjoins` edge to `Language` capturing the
complementarity. This reconciles the critic-2 objection: `us_legal_lexicon` sits under
`DomainLexicon` because it is *domain*-scoped; function words are domain-independent — an
orthogonal axis, not the same modeling problem.

### The loader gap
No existing loader returns the intermediate `WordNet` (all return materialized `English`).
Add **one** function:
```rust
#[cfg(feature = "prx")]
pub fn function_words_wordnet_from_prx(prx_gz: &[u8]) -> Result<WordNet, PrxError>
// gunzip → wordnet_envelope_from_bytes → wn_reconstruct_source (fail-closed) → read_wordnet
```
`build_english_function_words()` becomes the `reference_model()` twin: `#[cfg(feature="prx")]`
`include_bytes!` fast path → `function_words_wordnet_from_prx` → unchanged
`function_words_from_lmf`; XML fallback otherwise. Keeps the owned `HashMap` return (all
three callers move it).

### No new archive-axiom leg
Reuses the existing `WordNetPrxEnvelope` — it is the 2nd consumer of the **same** ontology
(USC #271 pattern), so `ontology_archive/axioms.rs` needs no edit. The floor/graph
reconstruct is already proven a byte-exact inverse (`prx.rs:1255`).

### Files (Track A)
- `lmf/prx.rs` — add `function_words_wordnet_from_prx` (+ register `WordNetLmfLens` for
  `english_function_words@2026`).
- `language.rs` — `build_english_function_words` → prx-fast-path + XML fallback.
- `data/function-words/english-function-words-2026.prx.gz` — NEW committed artifact.
- `source_taxonomy/ontology.rs` — `ClosedClassLexicon` concept + label + `is_a` + `Adjoins`
  + `parse_concept`/`concept_name` + `is_leaf`.
- `data_provisioning/ontology.rs` — `canonical_encoding` (XmlLmf) **and see Track D-8: move
  `local_path_override` row into praxis.toml rather than adding a 10th hand-row**.
- `praxis.toml` / `praxis.lock` — `[sources.english_function_words]` + pins.

### Tests (Track A) — stronger than OLiA's count-only guard
- `regenerate_english_function_words_prx` (`#[ignore]`) — writes the artifact.
- `bundled_function_words_prx_matches_the_xml` — **full-map** equality (key set + each
  `Vec<LexicalEntry>`), not entity-count.
- `decoders_survive_the_prx_roundtrip` — `the`→Definite, `this`→Demonstrative,
  `every`/`no`→Quantifier, `hello`→Greeting. The guardrail documenting why compact-codec is
  forbidden.
- `round_trip_recovers_source_features` — `(lemma, pos, synset_id, subcat)` preserved.

### Known must-fix test syncs (critic-1 blockers)
- `source_taxonomy/tests.rs:137` — `assert_eq!(leaves.len(), 17)` → 18 (+ contains check).
- `data_provisioning/tests.rs:400-405` — add `"english_function_words"` to `registered`.
- `completeness_meter()` / `all_sources_source_round_trip_byte_exact` now exercise the new
  source — confirm/adjust any floor-count or meter-length snapshot.
- `term_extractor.rs:107-193` independently `include_str!`s the same file for stopwords —
  the "ONE loaded source" doc claim (`language.rs:426`) becomes false unless that reader is
  routed through the shared loader. Track D or scope the doc claim honestly.

---

## 2. Inventory authority (research, high-confidence)

`english.xml` is a **hand-authored, partial** inventory (12 hand-picked determiners) that
is *citation-decorated*, not loaded-from-source. **Praxis-honesty flag:** "the source is
cited" ≠ "the enumeration is loaded". Making the inventory itself loaded (from a
machine-readable OLiA/UD closed-class resource) is separate praxis-debt (Track D / issue).

Citations to anchor `[sources.english_function_words]` + the XML header:
- **OLiA** (Chiarcos & Sukhareva 2015, doi:10.3233/SW-140167) — category backbone; already
  `[sources.olia]`. Every closed-class category the file uses exists under
  `olia-top:MorphosyntacticCategory`.
- **Quirk et al. 1985 CGEL** (ISBN 978-0-582-51734-9) — primary membership enumeration:
  Ch.5 determiners, Ch.6 pronouns, Ch.9 §9.1-9.7 prepositions, Ch.13 coordination, Ch.14
  subordination.
- **Huddleston & Pullum 2002 CamGEL** (ISBN 978-0-521-43146-0) — converging co-authority:
  Ch.3 aux/modal/copula, Ch.5 determinatives/pronouns, Ch.6 interrogative adverbs, Ch.7
  prepositions, Ch.12 relatives, **Ch.15 coordination**.
- **Biber et al. 1999 LGSWE** (ISBN 978-0-582-23725-4) — corroborating corpus.
- **Penn Treebank** (Santorini 1990, UPenn MS-CIS-90-47; Marcus et al. 1993,
  aclanthology.org/J93-2004) + **UD** closed-class POS — computational tagset.

**Fix three existing citation errors in the XML header:** conjunction cite "H&P Ch.8" →
"Ch.15 (Coordination and supplementation)"; the "Ch.16 §5" interjection cite (anchor on
LGSWE register data + OLiA:Interjection instead, or Ameka 1992 — see §3); the doubled
"English English Language" typo (~line 139).

---

## 3. Two literature-driven type-honesty flags (Track B)

Both in `pos.rs` / `language.rs` (the latter touched by `b77bafa`). High-confidence research.

**B-1. `Definiteness` is a misnomer.** Definiteness is binary ±definite (Lyons 1999,
doi:10.1017/CBO9780511605789); a demonstrative NP is *itself* definite and quantifiers
cross-cut the axis (Abbott 2010). The 4-way enum `{Definite, Indefinite, Demonstrative,
Quantifier}` conflates two axes — it is really a `DeterminerKind`/determiner-subclass
feature. Each *label* is citable (H&P Ch.5); the *type name* is not. → rename to
`DeterminerKind`, or document the conflation explicitly. Add Lyons 1999 + Abbott 2010 cites.

**B-2. `InterjectionKind` is missing Ameka 1992's CONATIVE class** (doi:10.1016/0378-2166(92)90048-G):
`sh!`, `psst`, summoning `hey!` silently default to `Expressive` — a real miscategorization.
Ameka's 3 top functions are Expressive / Conative / Phatic; the repo's
Greeting/Farewell/Response are Phatic subtypes, Politeness a phatic routine, Expressive
top-level — but Conative has no slot. → add a `Conative` variant + `fw-conative` synset, or
document the omission. Add Ameka 1992 / Wierzbicka 1992 / Wharton 2003 cites.

---

## 4. Mechanical residue sweep — 22 confirmed, 7 dismissed (Track D)

Each finding adversarially verified against the loaded source. The blessed target pattern:
a source→praxis-kind projection should be a `GeneratorAction::Functor` table interpreted by
`apply` (`pr4xis-runtime/src/apply.rs:79`), not a hardcoded Rust `match`. The **string
floor** (codec/wire decode, bootstrap `meta.rs`, generator-name/registry-key constants) is
exempt.

> **Nuance the verifiers established:** `wn_builder_to_owned` (`lmf/prx.rs:619`) and
> `function_words_from_lmf` (`language.rs:473`) were **dismissed** as closed-enum→closed-enum
> *codec floor*, NOT residue — the #87 functor relabels uniform Archive *kind strings*,
> not closed praxis wire-enums. So the projection-as-data lift of those is genuinely
> contested; it is a follow-up, not a violation. (Track C.)

### Tier 2 — high (1)
| # | Location | Finding | Fix |
|---|---|---|---|
| D-1 | `morphology/english/irregular.rs:1-158` | `english_irregulars()` ~110-entry hand-coded irregular-forms table; **AGID (Atkinson 2003)** exists but unregistered; untracked deferral | register AGID in praxis.toml + load via OnceLock (sibling of `english_function_words`) |

### Tier 3 — medium (8)
| # | Location | Finding | Fix |
|---|---|---|---|
| D-2 | `morphology/english/rules.rs:1-199` | `english_rules()` hand-enumerated affix rules; CatVar/WN-morphosemantic unregistered | register + OnceLock-load |
| D-3 | `xml/english_projection_v1.rs:61-80` | `canonical_phrase` match re-encodes W3C-Infoset `english_name` strings already loaded; self-titled "Functor" but compiled match | `GeneratorAction::Functor` over loaded `information_items()` |
| D-4 | `lmf/ontology.rs:370-391` | `SynsetRelationType::is_taxonomy/is_mereology/is_causal` — the WN-relType→praxis-kind partition as Rust `matches!` | carry as functor data (feeds the same projection) |
| D-5 | `xsd/datatypes/ontology.rs:562-607` | `BaseTypeAgreesWithCategory` axiom only exists to police drift between two encodings of one hierarchy (dual source of truth) | dissolves once D-16 lands |
| D-6 | `xsd/datatypes/ontology.rs:281-379` | datatype group membership (special/primitive/derived/list) hand-enumerated `[Concept;N]` + `.contains` | derive from loaded §3.4 Subsumption structure |
| D-7 | `formal/relations/ontology.rs:199-215` | `RelationProperty::get` match relation→structural-property; both already concepts | loaded typed edges in the `edges:` block |
| D-8 | `data_provisioning/ontology.rs:462-483` | `local_path_override` 9-row name→path table in Rust | add optional `local_path` to `[sources.*]`; **fold into Track A** so fw-prx adds data not a 10th hand-row |
| D-9 | `data_provisioning/ontology.rs:500-577` | `family_dir_for` per-source-NAME arms baking disk layout into a match | per-name arms → registry data; keep the kind-derived formula (floor) |

### Tier 4 — low (13)
| # | Location | Finding |
|---|---|---|
| D-10 | `applied/hmi/report/validator.rs` + `generator.rs` | `polarity` dark/light/unknown as bare `String`+`==`; `enum Polarity` already exists in base16 |
| D-11 | `pragmatics/discourse/legacy.rs` (73 lines) | dead `legacy` discourse module, re-exported, no live callers |
| D-12 | `morphology/english/rules.rs:205-212` | brittle `len()==13` |
| D-13 | `lexicon/olia.rs:201-226` | `pos_to_olia_fragments` hand-enumerated inverse of the loaded subsumption closure |
| **D-14** | `language.rs:547-548` | **stale doc-link to non-existent `super::lexicon::wh` module — introduced by `b77bafa`** |
| D-15 | `lmf/ontology.rs:596-603` | `VerbTransitivity::from_frame_id` `starts_with` substring tests over un-loaded frame vocab |
| D-16 | `xsd/datatypes/ontology.rs:385-434` | `base_type` match parallel to the `is_a:` edges (needs transitive reduction, not a flat filter) |
| D-17 | `xsd/datatypes/versioned.rs:100` | magic `46` datatype count inside an axiom's verify() |
| D-18 | `artifact_identity/ontology.rs:310-331` + `source_taxonomy is_leaf` | `is_leaf`/`is_family` hand `matches!` vs the loaded `ancestors_of`; **intersects Track A** (the `is_leaf` edit fw-prx needs) |
| D-19 | `artifact_identity/ontology.rs:399-408` | `ContentHashIsInjective` hardcodes 5 leaves instead of deriving the subtree |
| D-20 | `formal/math/ontology.rs:63-76` | `DomainOrder::get` hand-numbered u8 = depth of the loaded N⊂Z⊂Q⊂R⊂C chain |
| D-21 | meta ontologies (many) | brittle `variants().len()==N` inventory-count assertions |
| D-22 | `data_provisioning/decoders/mod.rs:27-40` | `has_decoder_for` hand-maintained `matches!` duplicating the `pub mod` set |

### Dismissed (7, correctly) — the string floor
`english/ontology.rs:568-619` (struct-loading codec); `language.rs:473-537`
(`function_words_from_lmf` closed-enum codec); `lmf/prx.rs:619-645` (`wn_builder_to_owned`
codec lowering); `xsd/from_xml` element-name ingest codec; `calculator/op.rs` interpreter
AST; `ins_gnss`/`sensor_fusion` rename `pub type` shims; `hmi/theming/schemes.rs`
closed-enum→canonical-name. None construct ontological knowledge.

---

## 5. Track C — projection-as-data (tracked follow-up, NOT this PR)
Lift `function_words_from_lmf` + `wn_builder_to_owned` (+ D-3, D-4, D-7) onto
`GeneratorAction::Functor` + `apply`, **jointly**, on the #87 engine-bridge line — so there
is one projection mechanism, not a forked second one. Contested by the audit's own verifiers
(codec-floor vs projection) → do both together or neither. References: `f920898`, `9e59bb0`,
`bridge.rs:193-224`, `prx.rs:552`.

---

## 6. Suggested batch ordering
- **Batch FW-A** (this effort) — function-words `.prx` load path (§1) + D-8 + D-18 folded in
  so registration *reduces* residue. Literature citations from §2.
- **Batch FW-B** (this effort or next) — type-honesty (§3): `Definiteness`→`DeterminerKind`,
  `InterjectionKind`+`Conative`, XML citation fixes, **D-14** (fix the stale doc-link I
  introduced).
- **Batch MORPH** — D-1 + D-2 + D-12 (register AGID/CatVar; load morphology tables).
- **Batch META-DERIVE** — D-5 + D-6 + D-16 + D-18 + D-19 + D-20 (derive from loaded
  morphisms) + D-21 (brittle counts).
- **Batch REGISTRY** — D-9 + D-22 (registry/decoder derivation).
- **Batch CLEANUP** — D-10 (Polarity) + D-11 (delete legacy) + D-13 + D-15 + D-17.
- **Batch C (engine-bridge)** — §5 projection-as-data lift.

## 7. Verification gate (per batch)
`dev-fmt` clean · `cargo clippy -p pr4xis-domains --features prx --lib` clean · the
relevant tests + the new structural guards · docs intra-doc links · `dev-ci` before push ·
commits unsigned (`--no-gpg-sign`), user re-signs on YubiKey · co-author awfmilton where
continuing #169/#201 lineage.
