//! THE MONOTONIC RATCHET for `defines_pointers`'s real-corpus recall —
//! mirrors `caregiver_capability_ratchet.rs`'s discipline, but in the
//! OPPOSITE direction: that file ratchets red-class CEILINGS down (fewer
//! failures is better); this one ratchets per-title pointer-count FLOORS up
//! (more genuinely-extracted statutory definitions is better). A change
//! may only move a title's count UP (then ratchet its floor up in the same
//! commit) — never silently down.
//!
//! Exists because `defines_pointers` (the VerbNet-grounded chart-parse
//! behind the whole USC defines-overlay, `crates/domains/src/social/
//! judicial/statute_structure/grounding.rs`) has no fixed "corpus of
//! questions" the way the caregiver/adversarial suites do — its input is
//! every lexical/prose candidate span across all 9 loaded USC titles
//! (hundreds of thousands of them), so "don't regress recall" has nothing
//! to check against without a REAL, full-corpus sweep. This test IS that
//! sweep: it calls `compute_defines_overlay` directly against `loaded()`
//! (every title, no `.cprx.gz` cache involved — the exact production
//! extraction, run in-process) and asserts the resulting per-title
//! `(urn, term)` pair count never drops below its committed floor.
//!
//! EXPENSIVE — `#[ignore]`d, the same discipline `pr4xis compile --defines
//! --lock` itself already documents (`prx.rs`'s own doc comment: "this is
//! the EXPENSIVE, hours-long regen"). Re-derive:
//! `cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml \
//!   --test defines_pointers_corpus_ratchet -- --ignored --nocapture`
//! and read the printed per-title breakdown.
//!
//! ## Baseline history
//!
//! **2026-07-21, initial floor** (this file's first commit): established
//! immediately after this session's two `defines_pointers` correctness
//! fixes landed — the sentence-splitting performance/recall fix
//! (`tokenize::split_into_sentences`, period+semicolon-aware, fixing a
//! catastrophic whole-blob-parse blowup that was ALSO silently losing
//! every definition in the largest candidates) and the trailing
//! "whether X or Y" exhaustive-conditional adjunct fix
//! (`is_trailing_alternative_adjunct_head`). A KNOWN, still-open gap
//! remained at this baseline (N-ary "or"-coordination containing a
//! lexically-ambiguous OOV compound) — the floors below are still all 0
//! (never yet re-measured against a real sweep), so this ratchet does not
//! yet enforce anything; it is a scaffold awaiting its first real
//! per-title baseline.
//!
//! **2026-07-21, coordination gap closed** (same day, later commit): the
//! N-ary "or"-coordination gap above is fixed —
//! `tokenize::collapse_medial_comma_adjuncts` now checks
//! `list_coordinator_commas` BEFORE either medial-supplement gate (see
//! that function's own doc for the full mechanism). The RED fixture this
//! paragraph used to cite by name has turned GREEN and been renamed:
//! `grounding.rs`'s
//! `recognizes_the_real_company_definition_combining_the_whether_adjunct_and_n_ary_or_coordination_fixes`.
//!
//! **2026-07-21, first real baseline** (same day, this commit): a THIRD
//! fix landed between the scaffold above and this measurement — the
//! trailing-whether adjunct drop gained a `is_predicate_leaf` guard
//! (adversarially found: the original drop could destroy an entire
//! sentence's propositional content when "whether" heads an embedded
//! interrogative COMPLEMENT rather than a true trailing adjunct, e.g.
//! "That is, whether there should be a separation ... or not." collapsing
//! to just "that is" — a real, corpus-attested construction, not
//! contrived). With all three fixes landed together, a full `--ignored`
//! sweep ran to completion: **4341 total `(urn, term)` pairs across all 9
//! titles, in 4227.8s (~70.5 min)**. Per-title breakdown, now the
//! committed floor for each: `{1: 3, 5: 309, 15: 732, 18: 192, 28: 54, 29:
//! 228, 42: 2130, 49: 432, 50: 180}`. Title 42 (2130 pairs, the largest
//! title by far — 268,604 candidates, ~3.7x Title 15's raw byte size) is
//! KNOWN to still carry unresolved pathological outliers beyond what this
//! session's three fixes reach (a separate investigation found 5 of its
//! largest candidates still time out past 60s, and 9 of the 10 that do
//! complete extract zero pointers despite 14-57 CPU-seconds each) — this
//! floor reflects the corpus AS THIS SWEEP ACTUALLY MEASURED IT, honestly,
//! not a ceiling on future improvement.

//! **2026-07-25, the floors are RE-BASELINED DOWNWARD — the only such move
//! in this file's history, and the reason it now enforces a second,
//! non-count invariant.** A raw `(urn, term)` count cannot tell a recovered
//! definition from a fabricated one, so a count-only floor rewards
//! fabrication and forbids precision work. That is exactly what had
//! happened: `defines_pointers` was reading its definiendum off whatever
//! filled the subject slot, without ever checking that the subject was a
//! metalinguistic MENTION (`“X” means …`) rather than an ordinary USED NP.
//! `grounding::definiendum_words` now requires
//! [`ExpressionUse::Mentioned`](pr4xis_domains::cognitive::linguistics::lambek::ExpressionUse),
//! carried from the tokenizer through the chart — see that function's own
//! doc and the `ADefiniendumIsMentionedNeverUsed` axiom.
//!
//! Measured, full in-process sweep (this same test, `--ignored`):
//! **2755 pairs in 603.2s**, per title `{1: 7, 5: 254, 15: 556, 18: 142,
//! 28: 44, 29: 91, 42: 1284, 49: 270, 50: 107}` — the floors below.
//!
//! The downward move is fabrication removal, adjudicated pair-by-pair, not
//! lost recall:
//!
//! - Against the immediately-preceding extraction (same corpus, same
//!   grammar, mention check absent), titles 5/29/42 dropped 249/271/2171
//!   pairs and GAINED 0. Of those 2,691 dropped pairs, **exactly one** had
//!   its term quoted anywhere in the provision it was attributed to
//!   (`(/us/usc/t5/s804/3, "rule")`), and that one was itself a
//!   coincidence: the derivation's subject was the USED "any rule" of the
//!   § 804(3) EXCLUSION clause under the predicate "relate"
//!   (`Prop{relating, [.., Concept{"rule", Used}]}`), not the quoted
//!   definiendum — § 804(3)'s actual definitional frame is "has the meaning
//!   given such term in section 551", which this grammar still does not
//!   cover at all.
//! - Of the pairs KEPT in those three titles (235 + 80 + 1156 = 1471),
//!   **100%** have their term quoted in the provision they are attributed
//!   to.
//! - Section-HEADING misparses, the fabrication class with its own
//!   structural signature, are gone: pairs keyed on a Form atom (a heading
//!   string, never a provision) went 138 → **0**, and section-root URN keys
//!   went 174 → **14**.
//! - Title 1, the hand-adjudicated title (30 quoted definienda across 17
//!   provisions): 10 pairs → 7, with all 6 reference-set recoveries intact
//!   and non-reference pairs 4 → 1.
//!
//! Title 1's floor is the one that moves UP (3 → 7).
//!
//! **2026-07-25, the partial-parse goal — every floor moves UP, precision
//! unchanged at 100%.** `defines_pointers` required the WHOLE candidate span
//! to reduce to one two-argument `Sem::Prop`, so a single construction the
//! chart could not attach ANYWHERE in a long statutory definiens silently
//! discarded a definiendum it had already fully analysed. Measured before this
//! change: of a seeded 400-span sample of the 29,416 candidates carrying the
//! canonical `The term “X” means …` frame, 69.5% yielded zero pointers.
//! `grounding::defines_pointers_single_span` now reads a PARTIAL parse — the
//! clause cover `reduce::clause_fragments_with_alternatives_and_table_and_width`
//! on the syntax side, `montague::interpret_maximal_spans_where` on the
//! semantic side — so a definitional clause is found wherever it sits inside
//! the provision. Nothing about WHICH readings count as definitional moved:
//! the two-argument `Sem::Prop`, the VerbNet-confirmed predicate and the
//! `ExpressionUse::Mentioned` subject are the same three checks, applied to
//! the clause instead of to the whole span. See the
//! `AnUnattachableAdjunctNeverHidesItsClause` axiom
//! (`lambek::reduce`) for the closure, and Abney (1991) *Parsing By Chunks*
//! §1–§3 for why constituency and attachment must be reported separately.
//!
//! Measured, full in-process sweep (this same test, `--ignored`):
//! **5443 pairs in 576.2s**, per title `{1: 12, 5: 501, 15: 1121, 18: 531,
//! 28: 125, 29: 190, 42: 2241, 49: 459, 50: 263}` — the floors below.
//!
//! Monotone, and NOT bought with fabrication:
//!
//! - Against the immediately-preceding extraction (same corpus, same
//!   definitional checks, whole-string goal): **0 pairs lost, 2,688 gained**
//!   — the new pair set is a strict superset of the 2,755 committed above.
//! - The precision half below passes on all 5,443: every extracted term is
//!   marked `ExpressionUse::Mentioned` by the tokenizer in the provision it
//!   is attributed to. Independently, an out-of-tree curly-quote check over
//!   the same 5,443 found 0 whose term is not quoted in its own provision.
//! - Section-HEADING misparses stay gone: Form-atom keys remain 0, and
//!   section-root URN keys are 80 (of which 0 fail the quote check) — all of
//!   them subdivision-less sections whose body IS the provision.
//! - Title 1, the hand-adjudicated title (30 quoted definienda across 17
//!   provisions): 7 pairs → 12, reference-set recall 6/30 → **11/30**
//!   (adding `oath`, `officer`, `subscription`, `writing` at § 1 and `text`
//!   at § 112b(k)(7)(A)), with non-reference pairs unchanged at 1.

const TITLE_1_FLOOR: usize = 12;
const TITLE_5_FLOOR: usize = 501;
const TITLE_15_FLOOR: usize = 1121;
const TITLE_18_FLOOR: usize = 531;
const TITLE_28_FLOOR: usize = 125;
const TITLE_29_FLOOR: usize = 190;
const TITLE_42_FLOOR: usize = 2241;
const TITLE_49_FLOOR: usize = 459;
const TITLE_50_FLOOR: usize = 263;

#[test]
#[ignore]
fn defines_pointers_never_regresses_the_committed_per_title_floors() {
    use pr4xis_domains::cognitive::linguistics::english::english_loaded;
    use pr4xis_domains::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::compute_defines_overlay;
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::{
        loaded, title_number_of_urn,
    };
    use std::collections::BTreeMap;
    use std::time::Instant;

    let usc = loaded();
    let en = english_loaded();
    let vn = verbnet_classes_loaded();
    let mint_domain = pr4xis::ontology::meta::OntologyName::new_static("ratchet_mint");

    let started = Instant::now();
    let pairs = compute_defines_overlay(usc, en, vn, &mint_domain);
    eprintln!(
        "DEFINES_POINTERS CORPUS RATCHET: {} total (urn, term) pairs across all titles in {:.1}s",
        pairs.len(),
        started.elapsed().as_secs_f64()
    );

    let mut by_title: BTreeMap<u32, usize> = BTreeMap::new();
    for (urn, _term) in &pairs {
        if let Some(title) = title_number_of_urn(urn) {
            *by_title.entry(title).or_insert(0) += 1;
        }
    }
    eprintln!("  per-title breakdown: {by_title:?}");

    for (title, floor) in [
        (1u32, TITLE_1_FLOOR),
        (5, TITLE_5_FLOOR),
        (15, TITLE_15_FLOOR),
        (18, TITLE_18_FLOOR),
        (28, TITLE_28_FLOOR),
        (29, TITLE_29_FLOOR),
        (42, TITLE_42_FLOOR),
        (49, TITLE_49_FLOOR),
        (50, TITLE_50_FLOOR),
    ] {
        let count = by_title.get(&title).copied().unwrap_or(0);
        assert!(
            count >= floor,
            "REGRESSION: Title {title}'s extracted-definition count DROPPED to {count} \
             (committed floor {floor}).\nA change may only move a title's count UP (then \
             ratchet the floor up in the same commit) — monotonic-or-nothing.\n\
             Full breakdown: {by_title:?}",
        );
    }

    // THE PRECISION HALF — without it, the floors above can be satisfied by
    // fabrication, which is exactly how they came to sit above a corpus that
    // was 60% junk (see this file's 2026-07-25 baseline note).
    //
    // Every extracted pair must be one the corpus itself MENTIONS in the
    // provision it is attributed to: re-tokenize that provision's own
    // candidate spans and require a token the TOKENIZER marks
    // `ExpressionUse::Mentioned` whose surface is the extracted term. Not a
    // quote-glyph search over the text — the same mention marking the
    // extraction itself runs on, asked again independently at the corpus
    // scale the per-sentence axiom (`ADefiniendumIsMentionedNeverUsed`)
    // cannot reach.
    //
    // Only the provisions that actually produced a pointer are re-tokenized
    // (~2k spans), not the corpus's ~700k candidates, so this adds seconds
    // to a ten-minute sweep.
    //
    // Through `split_into_sentences` FIRST, because that is what
    // `defines_pointers` itself does, unconditionally, before it tokenizes
    // anything (see its own doc for why). Asking the question of a different
    // input than the extraction saw does not make the check independent, it
    // makes it wrong: quoted-span collapsing pairs quote glyphs within the
    // text it is handed, so a multi-clause provision tokenized as ONE blob
    // pairs the CLOSING quote of one clause with the OPENING quote of the
    // next and reports a mention set that is not merely incomplete but
    // shifted. Measured on 1 U.S.C. § 1's body (ten semicolon-separated
    // definitional clauses): as one blob the tokenizer marks
    // `[insane, insane person, person, whoever, subscription, sworn]`, and
    // sentence by sentence it marks `[insane, insane person] [person,
    // whoever] [officer] [signature, subscription] [oath, sworn] [writing]`
    // — the real definienda. Likewise 18 U.S.C. § 2311, where the blob
    // reading finds only `[security]` and the sentence readings find
    // `cattle`, `livestock`, `money`, `motor vehicle`, `securities`,
    // `tax stamp` and `value`. Both provisions were flagged by this check
    // while the extraction was correct.
    use pr4xis_domains::cognitive::linguistics::lambek::ExpressionUse;
    use pr4xis_domains::cognitive::linguistics::lambek::tokenize::{
        split_into_sentences, tokenize_with_alternatives,
    };
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::{
        dangling_chapeau_reassembly_index, defines_prose_index, usc_archive,
    };
    use std::collections::{BTreeSet, HashMap};

    let operators = pr4xis_domains::cognitive::linguistics::lambek::operators::vocabulary();
    let dashes = pr4xis_domains::cognitive::linguistics::symbols::dash_punctuation::vocabulary();

    let archive = usc_archive(usc);
    let shadowed = defines_prose_index(usc);
    let reassembled = dangling_chapeau_reassembly_index(usc);
    let mut spans: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &archive.nodes {
        if let Some(text) = node.lexical.as_deref() {
            spans.entry(node.name.as_str()).or_default().push(text);
        }
    }
    for (urn, prose) in &shadowed {
        spans.entry(urn.as_str()).or_default().push(prose.as_str());
    }
    for (urn, candidates) in &reassembled {
        for candidate in candidates {
            spans
                .entry(urn.as_str())
                .or_default()
                .push(candidate.as_str());
        }
    }

    let mut unmentioned: Vec<(String, String)> = Vec::new();
    let mut mentions_of: HashMap<&str, BTreeSet<String>> = HashMap::new();
    for (urn, term) in &pairs {
        let mentioned = mentions_of.entry(urn.as_str()).or_insert_with(|| {
            spans
                .get(urn.as_str())
                .into_iter()
                .flatten()
                .flat_map(|text| split_into_sentences(text, operators, dashes, en))
                .flat_map(|sentence| tokenize_with_alternatives(&sentence, en).0)
                .filter(|t| t.expression_use == ExpressionUse::Mentioned)
                .map(|t| t.word)
                .collect()
        });
        if !mentioned.contains(term) {
            unmentioned.push((urn.clone(), term.clone()));
        }
    }
    eprintln!(
        "  mention-grounded: {}/{} pairs",
        pairs.len() - unmentioned.len(),
        pairs.len()
    );
    assert!(
        unmentioned.is_empty(),
        "PRECISION REGRESSION: {} of {} extracted pairs name a term their own provision \
         never MENTIONS — a definiendum is talked about, not used (Quine 1940 §4; see \
         `grounding::definiendum_words`). First 20: {:?}",
        unmentioned.len(),
        pairs.len(),
        &unmentioned[..unmentioned.len().min(20)],
    );
}
