//! Word-sense disambiguation for the two-entity relation-query path — which
//! `(child, parent)` sense PAIR answers "is X \[relation\] Y?" when a
//! surface resolves to more than one WordNet sense.
//!
//! # Where this applies, and where it deliberately does not
//!
//! [`LexicalReasoner::reaches`]-based relation queries (`crates/chat/src/lib.rs`'s `answer_question`, the
//! two-entity path) are the ONLY place in the pipeline that commits to a
//! single sense pair today — and it does so by first-hit iteration order
//! over `child_ids × parent_ids`, not by any plausibility signal. This
//! module replaces that blind first-hit with [`best_reaching_pair`], scored
//! by BARE gloss-word overlap (Lesk 1986's original algorithm).
//!
//! # Why bare overlap, not Banerjee & Pedersen (2002)'s relation-expanded gloss
//!
//! An earlier version of this module expanded each candidate's gloss with
//! its taxonomy neighbors' glosses (Banerjee & Pedersen 2002's one-hop
//! adaptation, designed for GENERAL WSD over two a-priori-unrelated words).
//! That expansion is DEGENERATE here specifically because `best_reaching_pair`
//! only ever compares candidates that are ALREADY directly related by the
//! very edge `reaches` just confirmed: expanding `cid` via its parents
//! trivially re-absorbs `pid`'s own gloss (the common case is `pid` IS
//! `cid`'s parent), swamping the direct signal with a guaranteed match
//! that says nothing about which SENSE is right; expanding `pid` via its
//! children leaks a SIBLING candidate's own gloss words back into the
//! comparison target, inflating whichever sibling has more distinctive
//! vocabulary regardless of actual relatedness. Both were caught empirically
//! (`TwoEntityPathPrefersGlossOverlapAmongReachingPairs`'s own fixture, which
//! this module's tests reproduce) before landing, not assumed away. Bare
//! gloss overlap has neither failure mode: it compares only `cid`'s own
//! definitions against `pid`'s own definitions, nothing borrowed from either
//! side's neighbors.
//!
//! Two other candidate sites were considered and explicitly rejected:
//! - The Lambek chart's derivation-preference key
//!   ([`chart_reduce`](crate::cognitive::linguistics::lambek::reduce::chart_reduce))
//!   is syntax-only (fewest type-changing steps, leftmost split, cost,
//!   structural order) and never tracks sense identity — sense resolution
//!   happens strictly AFTER the chart commits, so there is nothing to
//!   disambiguate there.
//! - `define_word` deliberately enumerates EVERY sense
//!   (`DefineEnumeratesTheLoadedSenseInventory`,
//!   `crates/chat/src/lib.rs`) — sense selection there is a settled,
//!   axiom-protected policy, not an open question this module touches.
//!
//! # Why gloss overlap, not WordNet's sense-number ordinality
//!
//! WordNet's own documentation (`cntlist(5WN)`) states sense ordering
//! "should not be construed as an accurate indicator of frequency of use"
//! for post-2001 releases — the empirical SemCor tag counts it was built
//! from were never updated. A most-frequent-sense baseline keyed on sense
//! rank would rest on data its own source disclaims. Gloss overlap needs no
//! such assumption: it reads only [`definitions`](super::concept_store::ConceptView::definitions)
//! and the taxonomy edges already loaded and already used elsewhere.
//!
//! # The lone-hit corroboration gate
//!
//! Gloss overlap only has something to discriminate AMONG when more than one
//! candidate pair reaches. When exactly ONE pair reaches, there is nothing
//! to rank — but a lone hit can still be a genuinely misleading answer: e.g.
//! "is cut an end?" resolves 70 senses of "cut" against 18 senses of "end",
//! and among the 1,260 candidate pairs exactly ONE satisfies `reaches` — an
//! obscure 4-hop verb-sense chain (cut "cease, stop" → break up → break →
//! end "bring to an end or halt") that IS real, loaded WordNet data (nothing
//! is guessed), but is not the reading a person would default to. A
//! calibration pass (run against this codebase's own corpus ratchet) proved
//! that no hop-count/path-length threshold over WordNet's OWN graph can
//! separate this shape from genuinely correct deep relations without a
//! catastrophic false-negative rate — the failure hop-counts sit in the
//! densest part of the genuine-relation distribution, not a separable gap.
//!
//! [`best_reaching_pair`] therefore consults FIVE INDEPENDENT signals —
//! [`crate::cognitive::linguistics::verbnet`] (Kipper, Korhonen, Ryant &
//! Palmer 2008), whose syntactic-semantic verb classification is derived
//! from Levin's (1993) alternation diagnostics,
//! [`crate::cognitive::linguistics::conceptnet`] (Speer, Chin & Havasi 2017),
//! a crowd- and dataset-sourced commonsense association graph,
//! [`crate::cognitive::linguistics::framenet`] (Baker, Fillmore & Lowe
//! 1998), a hand-curated semantic-frame lexicon,
//! [`crate::cognitive::linguistics::sumo`] (Niles & Pease 2001, 2003), a
//! formal upper-ontology class crosswalk, and
//! [`crate::cognitive::linguistics::propbank`] (Palmer, Gildea & Kingsbury
//! 2005), a cross-part-of-speech predicate argument-structure lexicon — none
//! derived from WordNet's own gloss/hypernym construction — ONLY for the
//! lone-hit case (never for the already-scored multi-hit case above), and
//! ONLY for relation kinds each source's signal is actually evidence FOR or
//! AGAINST.
//!
//! ## Why corroboration only gates `Similarity`/`Equivalence`, never `Subsumption`
//!
//! This scope is not incidental — it is the load-bearing correction of a
//! REAL regression a first version of this mechanism produced. Levin's
//! (1993) own "semantic coherence hypothesis" states class comembership
//! tracks verbs that "share at least some aspect of meaning" — a
//! COMPONENTIAL/similarity claim, never a specificity (is-a) claim (Levin
//! 1993 ch. 1; VerbNet's own annotation guidelines describe classes as
//! sharing "core semantic and syntactic properties", the same componential
//! framing). Olsen, Dorr & Clark (1997, AMTA/SIG-IL) had to IMPORT WordNet
//! sense tags to impose a hierarchy onto Levin classes in the first place —
//! direct evidence VerbNet's own class structure carries no native
//! hypernymy signal to check WordNet's hypernymy against. Baker &
//! Ruppenhofer (2002, BLS 28) find the same class-vs-hierarchy mismatch
//! against FrameNet's frame structure too, so this is not WordNet-specific.
//!
//! Consulting VerbNet for a `Subsumption` (is-a) query was tried and
//! measured: it moved this codebase's committed corpus is-a class from 4
//! failures to 47, entirely FALSE negatives — genuinely true WordNet
//! hypernym pairs (e.g. "is coughing a kind of eliminating?" — coughing IS a
//! specific way of eliminating/discharging, per WordNet) that VerbNet
//! correctly places in unrelated classes (`cough` in `breathe-40.1.2`;
//! `eliminate` in `remove-10.1`/`murder-42.1` — verified directly against
//! the loaded VerbNet 3.3 data), because "unrelated syntactic-alternation
//! behavior" says nothing about whether one word is a MORE SPECIFIC KIND of
//! the other. This is VerbNet working exactly as designed, not a data gap or
//! a crosswalk bug — `cough` and `eliminate` genuinely aren't near-synonyms
//! or alternation-mates, even though one really is a hyponym of the other.
//!
//! Notably, this scoping ALSO means no source gates the original
//! motivating "is cut an end?" case: that query's relation kind is
//! `Subsumption` (a bare copula "is" question), so it is out of scope
//! regardless of what any source says about cut/end specifically. (For
//! the record: VerbNet independently PLACES cut and end in the same class,
//! `stop-55.4` — confirming, not refuting, that the relation is real —
//! which matches this codebase's own earlier design-panel conclusion that
//! cut/end is a true, attested-but-unintuitive fact, not a spurious one, and
//! that no gating mechanism was ever going to make it answer differently
//! without breaking something else.) The mechanism below is real, tested,
//! and will engage the moment this pipeline generates a `Similarity`- or
//! `Equivalence`-kind two-entity query — it is honestly DORMANT today
//! because the loaded corpus generator does not yet produce that construct,
//! not because the code path doesn't work.
//!
//! ### ConceptNet and FrameNet get the same scope restriction, despite carrying `IsA`/hierarchical signal
//!
//! ConceptNet's 34 relation types (Speer, Chin & Havasi 2017; see the
//! [`crate::cognitive::linguistics::conceptnet`] module doc) include an
//! explicit `IsA` relation, which — unlike anything in VerbNet — genuinely IS
//! a hypernymy claim. This mechanism does not exploit that: every ConceptNet
//! relation type is mapped GENERICALLY onto the existing `Association`
//! relation kind (SKOS `related`), never distinguished by type (see
//! `ConceptNetEdge::relation`'s doc comment for why — treating a
//! heterogeneous bag of crowd-sourced `RelatedTo` mentions and curated `IsA`
//! assertions as uniformly-trustworthy evidence for a `Subsumption` claim is
//! exactly the kind of unprincipled per-source special-casing the VerbNet
//! regression above already proved dangerous; a principled `IsA`-specific
//! Subsumption corroboration path is future work, not this one).
//!
//! FrameNet's frame-to-frame relations (Ruppenhofer et al. 2016; see the
//! [`crate::cognitive::linguistics::framenet`] module doc) similarly include
//! `Inheritance`, a genuinely hierarchical relation between frames — but
//! frame INHERITANCE is not the same claim as a WORD being a hyponym of
//! another (two lexical units can evoke frames related by Inheritance while
//! bearing no hypernymy relation to each other at all), and `corroborate_lone_hit`
//! doesn't distinguish it from the other 8 relation types for the identical
//! reason ConceptNet's `IsA` isn't specially trusted. So `corroborate_lone_hit`
//! gates ALL THREE sources on `kind` identically, before any is ever
//! consulted.
//!
//! ## Composing corroboration sources
//!
//! Four outcomes, per [`ReachingPairOutcome`], for a `Similarity`/
//! `Equivalence`-kind lone hit, composing VerbNet, ConceptNet, FrameNet,
//! SUMO, and PropBank under two rules:
//! - **Rule 1 (any source corroborates):** VerbNet class-sharing OR a
//!   ConceptNet association OR a FrameNet frame-family match OR a SUMO
//!   class match OR a PropBank cross-POS roleset match agrees: `Trusted` —
//!   one independently-derived fact agreeing is enough, exactly like the
//!   multi-hit gloss-overlap winner needs only one signal, not unanimity
//!   across every candidate.
//! - **Rule 2 (coverage-with-no-match anywhere flags uncorroborated):**
//!   NO source corroborates, but AT LEAST ONE has data for BOTH
//!   concepts: real negative evidence, not silence — `Uncorroborated`. The
//!   caller (`answer_question`) must not fall through to an UNSCOPED
//!   cross-product negation check on this outcome (that would reintroduce
//!   the exact any-sense-pair bug this mechanism exists to retire) — only a
//!   check scoped to this SPECIFIC `(cid, pid)` pair is licensed.
//! - **No coverage anywhere:** no source has data for both concepts, OR
//!   `kind` is anything other than `Similarity`/`Equivalence` (including
//!   `Subsumption`): no signal either way — `Trusted`, preserving
//!   default-trust behavior. A concept absent from one or more sources is
//!   not evidence against it; every source's coverage is necessarily
//!   partial, so "no data" and "queried, found nothing" are kept
//!   epistemically distinct
//!   ([`crate::cognitive::linguistics::verbnet::store::VerbNetStore::has_coverage`],
//!   [`crate::cognitive::linguistics::conceptnet::store::ConceptNetStore::has_coverage`],
//!   [`crate::cognitive::linguistics::framenet::store::FrameNetStore::has_coverage`],
//!   [`crate::cognitive::linguistics::sumo::store::SumoStore::has_coverage`],
//!   [`crate::cognitive::linguistics::propbank::store::PropBankStore::has_coverage`]),
//!   never collapsed into one `Option`.
//! - **`NoPath`** (zero `reaches()` hits) and the multi-hit `Trusted` case
//!   are unaffected by any source — this composition only ever applies to
//!   the lone-hit case.

use hashbrown::HashSet;

use alloc::string::String;
use alloc::vec::Vec;

use pr4xis_runtime::ontology::ConceptRef;

use super::ontology::{ConceptId, LexicalReasoner};
use crate::cognitive::linguistics::conceptnet::store::ConceptNetStore;
use crate::cognitive::linguistics::framenet::store::FrameNetStore;
use crate::cognitive::linguistics::orthography::case_folding;
use crate::cognitive::linguistics::propbank::store::PropBankStore;
use crate::cognitive::linguistics::sumo::store::SumoStore;
use crate::cognitive::linguistics::verbnet::store::VerbNetStore;
use crate::formal::relations::ontology::{equivalence_relation_kind, similarity_relation_kind};
#[cfg(test)]
use pr4xis_runtime::ontology::subsumption_kind;

/// The result of [`best_reaching_pair`] — which of the candidate
/// `(child, parent)` pairs, if any, the two-entity relation path may trust,
/// and (for the risky lone-hit case) whether an independent source found
/// real evidence against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachingPairOutcome {
    /// No candidate pair satisfies `reaches` at all.
    NoPath,
    /// Exactly one candidate pair satisfies `reaches`, and the independent
    /// corroboration check found real negative evidence against it (both
    /// concepts ARE known to the corroboration source, but share no common
    /// classification with each other). The caller must not treat this as
    /// equivalent to `NoPath` — it is a SPECIFIC pair to check, not license
    /// for an unscoped any-sense-pair scan.
    Uncorroborated(ConceptId, ConceptId),
    /// A trustworthy pair — the unambiguous single hit, a lone hit with no
    /// negative evidence against it, or the gloss-overlap-selected winner
    /// among multiple hits.
    Trusted(ConceptId, ConceptId),
}

/// The bag of case-folded, function-word-filtered content words drawn from
/// `id`'s own gloss(es) — no relation expansion (see the module docs for
/// why: expanding via taxonomy neighbors is degenerate for a candidate
/// already known to be directly related to the thing it's compared against).
fn gloss_words(en: &dyn LexicalReasoner, id: ConceptId) -> HashSet<String> {
    let mut words = HashSet::new();
    push_gloss_words(en, id, &mut words);
    words
}

/// Splits `id`'s definitions (already loaded, [`ConceptView::definitions`](super::concept_store::ConceptView::definitions))
/// on whitespace, case-folds each token through the loaded Unicode simple
/// case-folding table ([`case_folding`], the same source
/// [`lookup_case_folded`](LexicalReasoner::lookup_case_folded) uses — never
/// `str::to_lowercase`), strips a token to its alphanumeric core, drops it
/// if [`is_function_word`](LexicalReasoner::is_function_word) says it's
/// closed-class, and inserts the rest. A bare whitespace split (not a full
/// chart pass) is deliberate: glosses are short definitional phrases, and
/// simplified/bag-of-words overlap is exactly what Kilgarriff & Rosenzweig
/// (2000) specify — running the Lambek chart over gloss text to disambiguate
/// the chart's own input would be circular.
fn push_gloss_words(en: &dyn LexicalReasoner, id: ConceptId, out: &mut HashSet<String>) {
    let Some(view) = en.concept(id) else {
        return;
    };
    let folder = case_folding::table();
    for gloss in view.definitions() {
        for raw in gloss.split_whitespace() {
            let trimmed = raw.trim_matches(|c: char| !c.is_alphanumeric());
            if trimmed.is_empty() {
                continue;
            }
            let folded = folder.fold(trimmed);
            if en.is_function_word(&folded) {
                continue;
            }
            out.insert(folded);
        }
    }
}

/// Lesk (1986) word-overlap score between two concepts' own glosses.
fn gloss_overlap_score(en: &dyn LexicalReasoner, cid: ConceptId, pid: ConceptId) -> usize {
    let a = gloss_words(en, cid);
    let b = gloss_words(en, pid);
    a.intersection(&b).count()
}

/// Do the independent sources corroborate a lone `reaches()` hit, or find
/// real evidence against it? See the module doc's "lone-hit corroboration
/// gate" section for the full rationale, including WHY this only ever
/// engages for `Similarity`/`Equivalence` — a measured regression (this
/// codebase's own committed corpus is-a class, 4 → 47 failures, entirely
/// false negatives) proved VerbNet class-sharing is not valid evidence for
/// `Subsumption` (is-a) queries, and the epistemic distinction between "no
/// coverage" and "queried, no connection" for the kinds it IS valid for.
///
/// Five independent sources compose under the same two rules, applied
/// across ALL FIVE rather than any one alone (the module doc's "composing
/// corroboration sources" section):
/// - Rule 1 (any corroborating source trusts): if VerbNet class-sharing OR
///   a ConceptNet association OR a FrameNet frame-family match OR a SUMO
///   class match OR a PropBank cross-POS roleset match corroborates the
///   pair, `Trusted` — one independently-derived agreement is enough,
///   exactly like the multi-hit gloss-overlap winner needs only one signal.
/// - Rule 2 (coverage-with-no-match anywhere flags uncorroborated): only if
///   NO source corroborates AND at least one source has DATA for BOTH
///   concepts (real negative evidence exists somewhere) is the pair
///   `Uncorroborated`. If no consulted source covers both concepts, default
///   trust is preserved — partial coverage in one source is never license to
///   flag a pair no source actually examined.
///
/// `pub`, not private: [`best_reaching_pair`] only reaches this via a real
/// `reaches()` hit, but `English`'s default `reaches()` implementation only
/// supports `Subsumption` — a synthetic `English` fixture can never produce a
/// lone `Similarity`/`Equivalence` hit through the full path. Callers
/// exercising THIS function's own Rule 1/Rule 2 composition behavior for
/// those kinds (this module's own unit tests, and
/// `crates/chat/src/lib.rs`'s `ConceptNetCorroborationComposesWithVerbNet`
/// axiom) call it directly, bypassing the `reaches()` gate.
#[allow(clippy::too_many_arguments)]
pub fn corroborate_lone_hit(
    en: &dyn LexicalReasoner,
    verbnet: &VerbNetStore,
    conceptnet: &ConceptNetStore,
    framenet: &FrameNetStore,
    sumo: &SumoStore,
    propbank: &PropBankStore,
    kind: &ConceptRef,
    cid: ConceptId,
    pid: ConceptId,
) -> ReachingPairOutcome {
    if *kind != similarity_relation_kind() && *kind != equivalence_relation_kind() {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    if verbnet.shares_class_family(cid, pid).is_some() {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    if conceptnet.shares_association(en, cid, pid) {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    if framenet.shares_frame_family(en, cid, pid) {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    if sumo.shares_sumo_class(cid, pid) {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    if propbank.shares_roleset(en, cid, pid) {
        return ReachingPairOutcome::Trusted(cid, pid);
    }
    let verbnet_covers_both = verbnet.has_coverage(cid) && verbnet.has_coverage(pid);
    let conceptnet_covers_both =
        conceptnet.has_coverage(en, cid) && conceptnet.has_coverage(en, pid);
    let framenet_covers_both = framenet.has_coverage(en, cid) && framenet.has_coverage(en, pid);
    let sumo_covers_both = sumo.has_coverage(cid) && sumo.has_coverage(pid);
    let propbank_covers_both = propbank.has_coverage(en, cid) && propbank.has_coverage(en, pid);
    if verbnet_covers_both
        || conceptnet_covers_both
        || framenet_covers_both
        || sumo_covers_both
        || propbank_covers_both
    {
        return ReachingPairOutcome::Uncorroborated(cid, pid);
    }
    ReachingPairOutcome::Trusted(cid, pid)
}

/// Replaces blind first-hit iteration: collects every `(cid, pid)` in
/// `child_ids × parent_ids` for which `en.reaches(cid, pid, kind)` holds.
///
/// - Zero hits: [`ReachingPairOutcome::NoPath`].
/// - Exactly one hit: the RISKY case (see module doc) — passed through
///   [`corroborate_lone_hit`], which is the ONLY place `verbnet` is
///   consulted.
/// - More than one hit: returns the one with the highest
///   `gloss_overlap_score` as `Trusted`, unconditionally (a second
///   independently-derived source is not consulted here — gloss overlap
///   among multiple already-true candidates is a different, already-solved
///   problem, not the lone-hit salience question); ties keep the
///   FIRST-encountered pair (a `<=` guard, not [`Iterator::max_by_key`] —
///   which returns the LAST max on ties and would reintroduce
///   nondeterminism relative to today's load-order-stable behavior).
#[allow(clippy::too_many_arguments)]
pub fn best_reaching_pair(
    en: &dyn LexicalReasoner,
    verbnet: &VerbNetStore,
    conceptnet: &ConceptNetStore,
    framenet: &FrameNetStore,
    sumo: &SumoStore,
    propbank: &PropBankStore,
    child_ids: &[ConceptId],
    parent_ids: &[ConceptId],
    kind: &ConceptRef,
) -> ReachingPairOutcome {
    let mut hits: Vec<(ConceptId, ConceptId)> = Vec::new();
    for &cid in child_ids {
        for &pid in parent_ids {
            if en.reaches(cid, pid, kind) {
                hits.push((cid, pid));
            }
        }
    }
    match hits.len() {
        0 => ReachingPairOutcome::NoPath,
        1 => {
            let (cid, pid) = hits[0];
            corroborate_lone_hit(
                en, verbnet, conceptnet, framenet, sumo, propbank, kind, cid, pid,
            )
        }
        _ => {
            let mut best: Option<((ConceptId, ConceptId), usize)> = None;
            for (cid, pid) in hits {
                let score = gloss_overlap_score(en, cid, pid);
                match &best {
                    Some((_, best_score)) if score <= *best_score => {}
                    _ => best = Some(((cid, pid), score)),
                }
            }
            let (cid, pid) = best.map(|(pair, _)| pair).expect("hits.len() > 1");
            ReachingPairOutcome::Trusted(cid, pid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::conceptnet::ontology::{ConceptNet, ConceptNetEdge};
    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::framenet::ontology::{FrameNet, FrameNetLexicalUnit};
    use crate::cognitive::linguistics::verbnet::ontology::{VerbNet, VerbNetClass, VerbNetMember};
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;
    use alloc::collections::BTreeMap;

    /// An empty VerbNet store — has coverage for nothing, so every lone hit
    /// falls to `Trusted` via the "no signal either way" branch, preserving
    /// pre-corroboration test expectations for fixtures that aren't
    /// exercising the corroboration mechanism itself.
    fn no_coverage_verbnet() -> VerbNetStore {
        VerbNetStore::from_verbnet_and_crosswalk(&VerbNet::default(), &BTreeMap::new())
    }

    /// An empty ConceptNet store — mirrors [`no_coverage_verbnet`]'s role for
    /// the second corroboration source.
    fn no_coverage_conceptnet() -> ConceptNetStore {
        ConceptNetStore::from_conceptnet(&ConceptNet::default())
    }

    /// An empty FrameNet store — mirrors [`no_coverage_verbnet`]'s role for
    /// the third corroboration source.
    fn no_coverage_framenet() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet::default())
    }

    /// An empty SUMO store — mirrors [`no_coverage_verbnet`]'s role for the
    /// fourth corroboration source.
    fn no_coverage_sumo() -> SumoStore {
        SumoStore::from_sumo(&crate::cognitive::linguistics::sumo::ontology::Sumo::default())
    }

    /// An empty PropBank store — mirrors [`no_coverage_verbnet`]'s role for
    /// the fifth corroboration source.
    fn no_coverage_propbank() -> PropBankStore {
        PropBankStore::from_propbank(
            &crate::cognitive::linguistics::propbank::ontology::PropBank::default(),
        )
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_single_reaching_pair_returns_unscored() {
        // "dog" (one sense) is-a "mammal" (one sense) — no ambiguity, no
        // gloss comparison needed to answer.
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-1" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="mammal-n-1" synset="s-mammal"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-mammal" ili="i2" partOfSpeech="n"><Definition>a warm-blooded vertebrate</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));
        let dog = en.lookup("dog")[0];
        let mammal = en.lookup("mammal")[0];
        let kind = subsumption_kind();
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        assert_eq!(
            best_reaching_pair(
                &en,
                &verbnet,
                &conceptnet,
                &framenet,
                &sumo,
                &propbank,
                &[dog],
                &[mammal],
                &kind
            ),
            ReachingPairOutcome::Trusted(dog, mammal)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn prefers_the_reaching_pair_with_higher_gloss_overlap() {
        // "cut" has two senses: a NOUN "a wound" (real gloss overlap with
        // the correct "injury" parent's gloss) and a VERB-flavored decoy
        // "a reduction" (no gloss overlap with "injury", but STILL reaches
        // it — a deliberately unrelated but structurally-true edge, mirroring
        // the real "cut"/"end" corpus failure this module fixes). Both
        // "cut" senses are hyponyms of BOTH "harm" candidates so both pairs
        // satisfy `reaches`; only the gloss overlap should discriminate.
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-cut-n">
      <Lemma writtenForm="cut" partOfSpeech="n"/>
      <Sense id="cut-n-wound" synset="s-cut-wound"/>
      <Sense id="cut-n-reduction" synset="s-cut-reduction"/>
    </LexicalEntry>
    <LexicalEntry id="e-harm-n">
      <Lemma writtenForm="harm" partOfSpeech="n"/>
      <Sense id="harm-n-injury" synset="s-harm-injury"/>
      <Sense id="harm-n-decrease" synset="s-harm-decrease"/>
    </LexicalEntry>
    <Synset id="s-cut-wound" ili="i1" partOfSpeech="n">
      <Definition>an injury from a sharp incision</Definition>
      <SynsetRelation relType="hypernym" target="s-harm-injury"/>
      <SynsetRelation relType="hypernym" target="s-harm-decrease"/>
    </Synset>
    <Synset id="s-cut-reduction" ili="i2" partOfSpeech="n">
      <Definition>an amount subtracted or removed</Definition>
      <SynsetRelation relType="hypernym" target="s-harm-injury"/>
      <SynsetRelation relType="hypernym" target="s-harm-decrease"/>
    </Synset>
    <Synset id="s-harm-injury" ili="i3" partOfSpeech="n"><Definition>physical injury or wound</Definition></Synset>
    <Synset id="s-harm-decrease" ili="i4" partOfSpeech="n"><Definition>a reduction or amount removed</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));
        let cut_ids = en.lookup("cut");
        let harm_ids = en.lookup("harm");
        assert_eq!(cut_ids.len(), 2);
        assert_eq!(harm_ids.len(), 2);
        let cut_wound = en.concept_by_synset("s-cut-wound").unwrap().id();
        let harm_injury = en.concept_by_synset("s-harm-injury").unwrap().id();
        let harm_decrease = en.concept_by_synset("s-harm-decrease").unwrap().id();

        let kind = subsumption_kind();
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        // "wound" overlaps "injury" via "injury"/"wound" — direct gloss
        // overlap correctly discriminates when only the injury parent is
        // offered.
        let winner = best_reaching_pair(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            cut_ids,
            &[harm_injury],
            &kind,
        );
        assert_eq!(winner, ReachingPairOutcome::Trusted(cut_wound, harm_injury));
        // Sanity: the reduction sense genuinely reaches BOTH parents too (the
        // fixture's existential search is real, not vacuous) — it just loses
        // on gloss overlap for the injury parent above.
        let cut_reduction = en.concept_by_synset("s-cut-reduction").unwrap().id();
        assert!(en.reaches(cut_reduction, harm_decrease, &kind));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn ties_keep_the_first_encountered_pair() {
        // A fixture with two structurally-identical-gloss-shape concepts, so
        // both candidate pairs score 0 overlap and the FIRST pair must win.
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-x-n">
      <Lemma writtenForm="x" partOfSpeech="n"/>
      <Sense id="x-n-1" synset="s-x1"/>
      <Sense id="x-n-2" synset="s-x2"/>
    </LexicalEntry>
    <LexicalEntry id="e-y-n"><Lemma writtenForm="y" partOfSpeech="n"/><Sense id="y-n-1" synset="s-y"/></LexicalEntry>
    <Synset id="s-x1" ili="i1" partOfSpeech="n"><Definition>alpha</Definition><SynsetRelation relType="hypernym" target="s-y"/></Synset>
    <Synset id="s-x2" ili="i2" partOfSpeech="n"><Definition>beta</Definition><SynsetRelation relType="hypernym" target="s-y"/></Synset>
    <Synset id="s-y" ili="i3" partOfSpeech="n"><Definition>gamma</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));
        let x_ids = en.lookup("x");
        let y = en.lookup("y")[0];
        assert_eq!(x_ids.len(), 2);
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        // Neither x-sense's gloss overlaps y's gloss at all — a true 0-0 tie.
        assert_eq!(
            best_reaching_pair(
                &en,
                &verbnet,
                &conceptnet,
                &framenet,
                &sumo,
                &propbank,
                x_ids,
                &[y],
                &subsumption_kind()
            ),
            ReachingPairOutcome::Trusted(x_ids[0], y)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn function_words_are_stripped_from_gloss_overlap() {
        // Two glosses share ONLY function words ("of", "a", "the") — overlap
        // must be 0 once stopwords are filtered, so the tie-break (not a
        // spurious "high overlap") decides.
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-a-n"><Lemma writtenForm="a" partOfSpeech="n"/><Sense id="a-n-1" synset="s-a"/></LexicalEntry>
    <LexicalEntry id="e-b-n"><Lemma writtenForm="b" partOfSpeech="n"/><Sense id="b-n-1" synset="s-b"/></LexicalEntry>
    <Synset id="s-a" ili="i1" partOfSpeech="n"><Definition>of a the</Definition></Synset>
    <Synset id="s-b" ili="i2" partOfSpeech="n"><Definition>the of a</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));
        let a = en.lookup("a")[0];
        let b = en.lookup("b")[0];
        assert_eq!(gloss_overlap_score(&en, a, b), 0);
    }

    /// Two-sense WordNet fixture (`cut` verb, `end` verb — one sense each,
    /// so `reaches` has exactly one candidate pair to find) mirroring the
    /// real cut/end shape closely enough to exercise the corroboration gate
    /// without needing the full real corpus.
    fn cut_end_lone_hit_fixture() -> English {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition><SynsetRelation relType="hypernym" target="s-end"/></Synset>
    <Synset id="s-end" ili="i2" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"))
    }

    /// Two-class VerbNet fixture: `cut` and `end` share `stop-55.4` (mirrors
    /// the real, verified stop-55.4 class this whole mechanism is grounded
    /// in — see the module doc's regression-history section).
    fn shared_class_verbnet_fixture(cut: ConceptId, end: ConceptId) -> VerbNetStore {
        let vn = VerbNet {
            classes: alloc::vec![VerbNetClass {
                id: "stop-55.4".into(),
                members: alloc::vec![VerbNetMember {
                    name: "cut".into(),
                    wn_sense_keys: alloc::vec!["cut%2:30:00".into()],
                }],
                subclasses: alloc::vec![VerbNetClass {
                    id: "stop-55.4-1-1".into(),
                    members: alloc::vec![VerbNetMember {
                        name: "end".into(),
                        wn_sense_keys: alloc::vec!["end%2:30:01".into()],
                    }],
                    subclasses: Vec::new(),
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                }],
                theme_roles: Vec::new(),
                frames: Vec::new(),
            }],
        };
        let crosswalk: BTreeMap<String, u64> = [
            ("cut%2:30:00".to_string(), cut.value()),
            ("end%2:30:01".to_string(), end.value()),
        ]
        .into_iter()
        .collect();
        VerbNetStore::from_verbnet_and_crosswalk(&vn, &crosswalk)
    }

    /// Two-class VerbNet fixture: `cut` and `end` in UNRELATED classes — both
    /// have coverage, neither shares a class family.
    fn unrelated_class_verbnet_fixture(cut: ConceptId, end: ConceptId) -> VerbNetStore {
        let vn = VerbNet {
            classes: alloc::vec![
                VerbNetClass {
                    id: "stop-55.4".into(),
                    members: alloc::vec![VerbNetMember {
                        name: "cut".into(),
                        wn_sense_keys: alloc::vec!["cut%2:30:00".into()],
                    }],
                    subclasses: Vec::new(),
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                },
                VerbNetClass {
                    id: "unrelated-99.9".into(),
                    members: alloc::vec![VerbNetMember {
                        name: "end".into(),
                        wn_sense_keys: alloc::vec!["end%2:30:01".into()],
                    }],
                    subclasses: Vec::new(),
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                },
            ],
        };
        let crosswalk: BTreeMap<String, u64> = [
            ("cut%2:30:00".to_string(), cut.value()),
            ("end%2:30:01".to_string(), end.value()),
        ]
        .into_iter()
        .collect();
        VerbNetStore::from_verbnet_and_crosswalk(&vn, &crosswalk)
    }

    /// A ConceptNet fixture with a `cut` <-> `end` association edge.
    fn shared_association_conceptnet_fixture() -> ConceptNetStore {
        ConceptNetStore::from_conceptnet(&ConceptNet {
            edges: alloc::vec![ConceptNetEdge {
                relation: "RelatedTo".to_string(),
                start_lemma: "cut".to_string(),
                end_lemma: "end".to_string(),
                weight: 1.0,
            }],
        })
    }

    /// A ConceptNet fixture where BOTH `cut` and `end` have coverage (other
    /// edges), but no edge between the two of them.
    fn unrelated_association_conceptnet_fixture() -> ConceptNetStore {
        ConceptNetStore::from_conceptnet(&ConceptNet {
            edges: alloc::vec![
                ConceptNetEdge {
                    relation: "RelatedTo".to_string(),
                    start_lemma: "cut".to_string(),
                    end_lemma: "sever".to_string(),
                    weight: 1.0,
                },
                ConceptNetEdge {
                    relation: "RelatedTo".to_string(),
                    start_lemma: "end".to_string(),
                    end_lemma: "finish".to_string(),
                    weight: 1.0,
                },
            ],
        })
    }

    /// A FrameNet fixture with `cut` and `end` evoking the SAME frame.
    fn shared_frame_framenet_fixture() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet {
            lexical_units: alloc::vec![
                FrameNetLexicalUnit {
                    lemma: "cut".to_string(),
                    pos: crate::social::software::markup::xml::lmf::LmfPos::Verb,
                    frame: "Cause_to_end".to_string(),
                },
                FrameNetLexicalUnit {
                    lemma: "end".to_string(),
                    pos: crate::social::software::markup::xml::lmf::LmfPos::Verb,
                    frame: "Cause_to_end".to_string(),
                },
            ],
            relations: Vec::new(),
        })
    }

    /// A FrameNet fixture where BOTH `cut` and `end` have coverage (other
    /// frames), but no shared or related frame.
    fn unrelated_frame_framenet_fixture() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet {
            lexical_units: alloc::vec![
                FrameNetLexicalUnit {
                    lemma: "cut".to_string(),
                    pos: crate::social::software::markup::xml::lmf::LmfPos::Verb,
                    frame: "Cutting".to_string(),
                },
                FrameNetLexicalUnit {
                    lemma: "end".to_string(),
                    pos: crate::social::software::markup::xml::lmf::LmfPos::Verb,
                    frame: "Process_completed_state".to_string(),
                },
            ],
            relations: Vec::new(),
        })
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn framenet_corroborates_a_lone_hit_with_no_verbnet_or_conceptnet_signal() {
        // Rule 1: neither VerbNet nor ConceptNet has coverage, but FrameNet
        // independently finds a shared frame — Trusted.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = shared_frame_framenet_fixture();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn all_three_sources_covering_but_none_corroborating_is_uncorroborated() {
        // Rule 2 across all three: every source has data for the pair and
        // every source agrees there's no connection.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = unrelated_association_conceptnet_fixture();
        let framenet = unrelated_frame_framenet_fixture();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Uncorroborated(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn framenet_never_downgrades_a_subsumption_query_even_with_unrelated_frames() {
        // THE REGRESSION-GUARD TEST for FrameNet specifically: the exact
        // same unrelated-frames data that produces Uncorroborated for
        // Similarity must stay Trusted for Subsumption.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = unrelated_frame_framenet_fixture();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &subsumption_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn verbnet_corroborates_a_lone_hit_that_shares_a_class_family() {
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = shared_class_verbnet_fixture(cut, end);
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        // Direct unit test of the corroboration function itself (not routed
        // through best_reaching_pair's reaches() gate, which English's
        // default impl only supports for Subsumption — see module doc for
        // why this mechanism is scoped to Similarity/Equivalence).
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn conceptnet_corroborates_a_lone_hit_with_no_verbnet_signal() {
        // Rule 1 (either source corroborates): VerbNet has no coverage at
        // all, but ConceptNet finds a direct association — Trusted.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = no_coverage_verbnet();
        let conceptnet = shared_association_conceptnet_fixture();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn verbnet_flags_a_lone_hit_with_no_shared_class_as_uncorroborated_for_similarity() {
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        // Both concepts have VerbNet coverage, but in UNRELATED classes —
        // real negative evidence, not silence — for a Similarity-kind query.
        // ConceptNet has no coverage at all, so it contributes nothing here.
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Uncorroborated(cut, end));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn both_sources_covering_but_neither_corroborating_is_uncorroborated() {
        // Rule 2 requires coverage AND no match from EITHER source — this is
        // the case where BOTH sources have data for the pair and BOTH agree
        // there's no connection, the strongest form of real negative
        // evidence this mechanism can find.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = unrelated_association_conceptnet_fixture();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Uncorroborated(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn conceptnet_corroboration_overrides_verbnet_silence_on_no_match() {
        // Rule 1 again, from the other direction: VerbNet has coverage for
        // both concepts and finds no shared class (real VerbNet-side
        // negative evidence), but ConceptNet independently finds a direct
        // association — ONE corroborating source is enough, so the overall
        // outcome is Trusted, not Uncorroborated.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = shared_association_conceptnet_fixture();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn verbnet_never_downgrades_a_subsumption_query_even_with_unrelated_classes() {
        // THE REGRESSION-GUARD TEST: the exact same "unrelated classes" data
        // (from BOTH sources now) that produces Uncorroborated for Similarity
        // above MUST produce Trusted for Subsumption — this is the fix for
        // the real measured regression (committed corpus is-a class 4 -> 47
        // failures) the module doc's "why VerbNet only gates Similarity/
        // Equivalence" section documents. Neither source's class/association
        // sharing is valid evidence for an is-a claim, so neither may be
        // allowed to veto one.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = unrelated_association_conceptnet_fixture();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &subsumption_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_coverage_from_either_source_preserves_default_trust_for_a_lone_hit() {
        // No signal either way, from EITHER source, must default to
        // Trusted, not Uncorroborated — absence of data is not evidence
        // against the claim.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = corroborate_lone_hit(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &similarity_relation_kind(),
            cut,
            end,
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_cut_end_case_is_trusted_end_to_end_via_subsumption() {
        // Integration-level confirmation: routed through best_reaching_pair
        // (not corroborate_lone_hit directly), the real "is cut an end?"
        // shape — a Subsumption-kind lone hit — is Trusted regardless of
        // what either source says, because Subsumption is out of scope for
        // both. Uses the UNRELATED fixtures for both sources specifically,
        // so this test would fail loudly if the Similarity/Equivalence kind
        // gate in `corroborate_lone_hit` were ever accidentally removed or
        // bypassed.
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        let end = en.lookup("end")[0];
        let verbnet = unrelated_class_verbnet_fixture(cut, end);
        let conceptnet = unrelated_association_conceptnet_fixture();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = best_reaching_pair(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        );
        assert_eq!(outcome, ReachingPairOutcome::Trusted(cut, end));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn zero_reaching_pairs_returns_no_path() {
        let en = cut_end_lone_hit_fixture();
        let cut = en.lookup("cut")[0];
        // "cut" does not reach itself under Subsumption in this fixture
        // (no self-loop declared) — zero candidate pairs.
        let verbnet = no_coverage_verbnet();
        let conceptnet = no_coverage_conceptnet();
        let framenet = no_coverage_framenet();
        let sumo = no_coverage_sumo();
        let propbank = no_coverage_propbank();
        let outcome = best_reaching_pair(
            &en,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &[cut],
            &[],
            &subsumption_kind(),
        );
        assert_eq!(outcome, ReachingPairOutcome::NoPath);
    }
}
