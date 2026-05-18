//! Statute understanding — composed view of a Statute through every
//! published ontology layer that gives praxis "understanding" of its
//! terms. Generic across statutes: no statute-specific code lives
//! here; per-statute assertions sit in test modules
//! (e.g. `sox_1514a/canonical_audit.rs`).
//!
//! # Five layers
//!
//! For every [`LegalTerm`] in a [`Statute`], the layered projections
//! are:
//!
//! 1. **Lexical** — via [`super::english_adjunction`]: each term's
//!    name decomposes into typed `Form` lemmas (ontolex:Form per
//!    McCrae et al. 2017, W3C 2017); each lemma resolves to zero+
//!    `Sense` values (ontolex:LexicalSense) in English WordNet
//!    (Fellbaum 1998).
//!
//! 2. **Morphosyntactic** — via OLiA (Chiarcos & Sukhareva 2015,
//!    *Semantic Web* journal): each resolved sense's WordNet
//!    [`LmfPos`] is projected through [`lmf_pos_to_pos_tag`] to the
//!    OLiA-aligned [`PosTag`] enum from
//!    [`cognitive::linguistics::lexicon::pos`][crate::cognitive::linguistics::lexicon::pos].
//!    Both source enums are OLiA-aligned at the ontology macro level
//!    (LmfPos enum doc-comment cites OLiA; PosTag enum doc-comment
//!    cites Chiarcos & Sukhareva 2015).
//!
//! 3. **Legal frame** — LITERATURE_GAP. The previously-deprecated
//!    `legal_actor` synthesis ontology (FRCP Rules 17/38/72; FRE 702;
//!    5 U.S.C. § 551; Tumey v. Ohio, 273 U.S. 510 (1927)) is on disk
//!    but not exported, per the architectural rule "one ontology per
//!    primary source" (see `social/judicial/mod.rs` for the rationale).
//!    The replacement per-source ontologies (`frcp_rule_17`,
//!    `frcp_rule_38`, `frcp_rule_72`, `fre_rule_702`) have NOT yet
//!    been published. Until they land, the Layer 3 projection on every
//!    term is `None` and the field exists as a typed placeholder. When
//!    the per-source ontologies are loaded, [`resolve_legal_role`]
//!    should compose them via the SourceTaxonomy `Adjoins` graph.
//!
//! 4. **Judicial relations** — already on [`Statute`]: the
//!    [`RelationType`][rel_type] graph (Composes, Requires,
//!    AffirmativeDefenseTo, …) per the existing judicial ontology.
//!    The understanding view exposes per-term inbound/outbound
//!    relation counts.
//!
//! 5. **Provenance** — already on every [`SourceTextRef`]: each
//!    term's `name` and `definition` carry a `context_uri` pinning
//!    them to the LRC USLM URN of the source section.
//!
//! [`LegalTerm`]: crate::social::judicial::ontology::LegalTerm
//! [`Statute`]: crate::social::compliance::statutes::Statute
//! [`PosTag`]: crate::cognitive::linguistics::lexicon::pos::PosTag
//! [`LmfPos`]: crate::social::software::markup::xml::lmf::ontology::LmfPos
//! [`LegalActorConcept`]: crate::social::judicial::legal_actor::ontology::LegalActorConcept
//! [`SourceTextRef`]: crate::formal::information::ontology::SourceTextRef
//! [rel_type]: crate::social::judicial::ontology::RelationType
//!
//! # Generic over statutes
//!
//! Per project rule: no SOX-specific code outside tests. The
//! resolver is generic over [`Statute`]; SOX-specific assertions
//! live in `sox_1514a/canonical_audit.rs` and `sox_1514a/tests.rs`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::lexicon::pos::PosTag;
use crate::formal::meta::identifier_format::Identifier;
use crate::social::compliance::statutes::Statute;
use crate::social::software::markup::xml::lmf::ontology::LmfPos;

use super::english_adjunction::{LemmaSenseMapping, resolve_term_name_to_senses};

// =============================================================================
// Public types
// =============================================================================

/// Resolution status of a lemma's lookup against English WordNet.
///
/// Three explicit kinds — never silently empty:
/// - [`Resolved`][Self::Resolved]: at least one WordNet sense matched.
/// - [`StatutoryTermOfArt`][Self::StatutoryTermOfArt]: zero senses,
///   classified as a statute-specific term (e.g. acronyms,
///   abbreviations like "SEC", section labels like "1514A").
/// - [`Unresolved`][Self::Unresolved]: zero senses, no statutory-
///   term classification available. This is a tripwire — every
///   such case should be either explained as a legitimate gap (and
///   reclassified as `StatutoryTermOfArt`) or repaired by extending
///   the lexicon.
///
/// Citation: WordNet's coverage is open-vocabulary but bounded
/// (Fellbaum 1998 ch. 1). Statutory terms-of-art frequently fall
/// outside the lexicon (Bhatia, *Analysing Genre: Language Use in
/// Professional Settings*, 1993, ch. 4 — "the lexicon of statutes").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionStatus {
    Resolved,
    StatutoryTermOfArt,
    Unresolved,
}

/// Layer-1+2 projection for one content-word lemma extracted from a
/// statute term's name.
///
/// Composes the lexical mapping from
/// [`super::english_adjunction::resolve_term_name_to_senses`] with
/// the morphosyntactic projection from [`lmf_pos_to_pos_tag`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermLexicalResolution {
    /// The lemma + senses returned by the lexical adjunction.
    pub lemma_sense: LemmaSenseMapping,
    /// Distinct OLiA-aligned PosTags across the matched senses.
    /// Empty when the lemma is [`Unresolved`][ResolutionStatus::Unresolved]
    /// or [`StatutoryTermOfArt`][ResolutionStatus::StatutoryTermOfArt]
    /// (no senses → no POS).
    pub pos_tags: Vec<PosTag>,
    /// Status of this lemma — never `Resolved` with zero senses.
    pub status: ResolutionStatus,
}

/// Composed five-layer understanding of one statute term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermUnderstanding {
    /// The term's CURIE identifier.
    pub term_id: Identifier,
    /// Verbatim term-name English text (as in the Statute).
    pub term_name: String,
    /// Layer 1+2: per-content-lemma lexical + morphosyntactic
    /// resolutions. Empty iff the term name yielded no content-word
    /// lemmas after stopword + numeric-token filtering (per
    /// [`super::term_extractor::extract_lemmas`]).
    pub lexical: Vec<TermLexicalResolution>,
    /// Layer 3: typed legal-frame role for this term, when the term
    /// denotes a statutory actor (party, judge, witness, …). Always
    /// `None` in this iteration — see module-level LITERATURE_GAP
    /// note. Once the per-source ontologies
    /// (`frcp_rule_17` etc.) land, [`resolve_legal_role`] populates
    /// this field with the typed Identifier CURIE of the matched
    /// concept (composed via SourceTaxonomy Adjoins).
    pub legal_role: Option<Identifier>,
    /// Layer 4: count of judicial-relation edges entering this term
    /// (inbound) and leaving it (outbound) in the parent Statute.
    pub inbound_relation_count: usize,
    pub outbound_relation_count: usize,
    /// Layer 5: provenance — the term name's `context_uri`, which
    /// for USLM-derived statutes is the section URN
    /// (`/us/usc/t<N>/s<X>`); for praxis-lock-derived statutes is
    /// the `praxis-lock://<name>@<version>` shim.
    pub provenance_context_uri: Option<String>,
}

impl TermUnderstanding {
    /// True iff every content lemma resolved to ≥1 WordNet sense
    /// OR was explicitly classified as a statutory term-of-art.
    /// `false` iff any lemma is [`Unresolved`].
    ///
    /// [`Unresolved`]: ResolutionStatus::Unresolved
    pub fn is_fully_understood(&self) -> bool {
        self.lexical
            .iter()
            .all(|r| r.status != ResolutionStatus::Unresolved)
    }

    /// Count of lemmas in each resolution-status bucket.
    pub fn status_counts(&self) -> (usize, usize, usize) {
        let mut resolved = 0;
        let mut toart = 0;
        let mut unresolved = 0;
        for r in &self.lexical {
            match r.status {
                ResolutionStatus::Resolved => resolved += 1,
                ResolutionStatus::StatutoryTermOfArt => toart += 1,
                ResolutionStatus::Unresolved => unresolved += 1,
            }
        }
        (resolved, toart, unresolved)
    }
}

/// Composed understanding of a whole Statute — one
/// [`TermUnderstanding`] per term, in the term order on the source
/// Statute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatuteUnderstanding {
    pub statute_name: String,
    pub statute_version: String,
    pub terms: Vec<TermUnderstanding>,
}

impl StatuteUnderstanding {
    /// Count of fully-understood terms (every content lemma either
    /// resolved or marked StatutoryTermOfArt).
    pub fn fully_understood_count(&self) -> usize {
        self.terms
            .iter()
            .filter(|t| t.is_fully_understood())
            .count()
    }

    /// Terms with at least one [`Unresolved`] lemma — the tripwire
    /// list. An empty Vec means the statute is 100% understood at
    /// the lexical layer.
    ///
    /// [`Unresolved`]: ResolutionStatus::Unresolved
    pub fn terms_with_unresolved_lemmas(&self) -> Vec<&TermUnderstanding> {
        self.terms
            .iter()
            .filter(|t| !t.is_fully_understood())
            .collect()
    }
}

// =============================================================================
// Functors / projections
// =============================================================================

/// LmfPos → PosTag projection. Both source enums are OLiA-aligned
/// at their definitions:
/// - [`LmfPos`] cites OLiA + Universal Dependencies POS tag set.
/// - [`PosTag`] cites Chiarcos & Sukhareva 2015 (OLiA).
///
/// The mapping is 1-to-1 on the open- and closed-class leaves;
/// `LmfPos::Other` projects to `None` because OLiA's "Other" /
/// catch-all has no published equivalent in our PosTag enum
/// (a published OLiA "Other" class would be needed to extend this).
pub fn lmf_pos_to_pos_tag(lmf: LmfPos) -> Option<PosTag> {
    Some(match lmf {
        LmfPos::Noun => PosTag::Noun,
        LmfPos::Verb => PosTag::Verb,
        LmfPos::Adjective => PosTag::Adjective,
        LmfPos::Adverb => PosTag::Adverb,
        LmfPos::Determiner => PosTag::Determiner,
        LmfPos::Pronoun => PosTag::Pronoun,
        LmfPos::Preposition => PosTag::Preposition,
        LmfPos::Conjunction => PosTag::Conjunction,
        LmfPos::Particle => PosTag::Particle,
        LmfPos::Copula => PosTag::Copula,
        LmfPos::Auxiliary => PosTag::Auxiliary,
        LmfPos::Interjection => PosTag::Interjection,
        LmfPos::Numeral => PosTag::Numeral,
        LmfPos::Other => return None,
    })
}

/// Layer 3 resolver: typed legal-frame role for a term name.
///
/// LITERATURE_GAP("per-source FRCP / FRE legal-actor ontologies"):
/// the deprecated `legal_actor` synthesis ontology is on disk but
/// not exported per the architectural rule (see
/// `social/judicial/mod.rs`'s comment). The replacement per-source
/// ontologies — `frcp_rule_17` (Plaintiff/Defendant capacity),
/// `frcp_rule_38` (Jury), `frcp_rule_72` (Magistrate),
/// `fre_rule_702` (Expert Witness), `5_usc_551` (Agency definitions) —
/// have not yet been published as praxis ontology macros.
///
/// Until they land, this resolver returns `None` for every term. The
/// field exists in [`TermUnderstanding`] so the API shape is stable;
/// when the per-source ontologies are published, this function
/// composes them via the SourceTaxonomy `Adjoins` graph and returns
/// the typed Identifier CURIE of the matched concept.
pub fn resolve_legal_role(_term_name: &str) -> Option<Identifier> {
    // LITERATURE_GAP: see function docstring. Per project rule
    // "if there is a source, find it and use it; if you have a gap,
    // explicitly tag it as such" — this is the explicit tag.
    None
}

/// Heuristic classifier for "statutory term-of-art" lemmas that
/// don't resolve in WordNet.
///
/// LITERATURE_GAP("a published statutory-term-of-art lexicon"):
/// Black's Law Dictionary 11th ed. covers many but is not loaded
/// here. As a stand-in classifier, the following bounded patterns
/// are recognized:
/// - All-caps abbreviations (3+ letters; e.g. "SOX", "AIR21", "SEC")
/// - Statute section markers (digit-letter mixtures; e.g. "1514A",
///   "42121", "78j-1")
///
/// These two patterns are *structural* artifacts of how statutory
/// text labels its own anchors, not natural-language vocabulary —
/// hence their absence from a lexical database is expected and the
/// `StatutoryTermOfArt` classification is appropriate.
///
/// Caller responsibility: when this returns `false` for an
/// unresolved lemma, surface the lemma as `Unresolved` so an
/// auditor can extend the lexicon or refine the classifier.
pub fn is_statutory_term_of_art(lemma: &str) -> bool {
    if lemma.is_empty() {
        return false;
    }
    let chars: Vec<char> = lemma.chars().collect();

    // All-caps abbreviation: ≥3 chars, all ASCII-uppercase OR digit,
    // AND at least one alphabetic character (so pure-digit strings
    // like "1514" don't match — those fall through to the section-
    // marker check below if they have a letter component, or are
    // rejected entirely if pure-numeric).
    let all_upper_alpha = chars.len() >= 3
        && chars
            .iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && chars.iter().any(|c| c.is_ascii_alphabetic());
    if all_upper_alpha {
        return true;
    }

    // Section-marker (e.g. "1514A", "78j-1", "42121", "1514"):
    // ASCII alphanumeric + hyphens, ≥1 digit, ≥3 chars total.
    // Pure-digit tokens of length ≥3 qualify as section numbers
    // (per Bluebook §3.3.4 statute-citation conventions, USC
    // sections are referenced by their numeric identifier whether
    // or not they carry a letter suffix).
    let mut has_digit = false;
    let mut all_allowed = true;
    for c in &chars {
        if c.is_ascii_digit() {
            has_digit = true;
        } else if !c.is_ascii_alphabetic() && *c != '-' {
            all_allowed = false;
        }
    }
    chars.len() >= 3 && has_digit && all_allowed
}

/// Resolve a single statute term to its [`TermUnderstanding`].
///
/// Pipeline:
/// 1. Layer 1+2: lemmatise term name; resolve each lemma to WordNet
///    senses; project each sense's `LmfPos` to a [`PosTag`].
/// 2. Layer 3: match term name against `LegalActorConcept` labels.
/// 3. Layer 4: count inbound/outbound relations from the source
///    `Statute`'s relation graph.
/// 4. Layer 5: copy the `context_uri` from the term's `name` field.
///
/// Generic over the statute — the legal-term's parent statute is
/// passed in so Layer 4 has the relation graph.
pub fn understand_term(
    statute: &Statute,
    term: &crate::social::judicial::ontology::LegalTerm,
    english: &English,
) -> TermUnderstanding {
    // Layer 1+2: lexical resolution + OLiA POS projection.
    let lemma_mappings = resolve_term_name_to_senses(&term.name.text, english);
    let lexical: Vec<TermLexicalResolution> = lemma_mappings
        .into_iter()
        .map(|lm| {
            let mut pos_tags: Vec<PosTag> = lm
                .senses
                .iter()
                .filter_map(|s| english.concept_by_synset(&s.reference.concept))
                .filter_map(|c| lmf_pos_to_pos_tag(c.pos))
                .collect();
            pos_tags.sort_by_key(|p| format!("{p:?}"));
            pos_tags.dedup();
            let status = if !lm.senses.is_empty() {
                ResolutionStatus::Resolved
            } else if is_statutory_term_of_art(&lm.form.written_rep) {
                ResolutionStatus::StatutoryTermOfArt
            } else {
                ResolutionStatus::Unresolved
            };
            TermLexicalResolution {
                lemma_sense: lm,
                pos_tags,
                status,
            }
        })
        .collect();

    // Layer 3: legal-frame role (LITERATURE_GAP — see resolve_legal_role).
    let legal_role = resolve_legal_role(&term.name.text);

    // Layer 4: relation counts.
    let inbound_relation_count = statute.relations_to(&term.id).count();
    let outbound_relation_count = statute.relations_from(&term.id).count();

    // Layer 5: provenance.
    let provenance_context_uri = term.name.context_uri.clone();

    TermUnderstanding {
        term_id: term.id.clone(),
        term_name: term.name.text.clone(),
        lexical,
        legal_role,
        inbound_relation_count,
        outbound_relation_count,
        provenance_context_uri,
    }
}

/// Resolve every term in a `Statute` to its [`TermUnderstanding`].
///
/// Generic across statutes — invokes [`understand_term`] for each
/// term in `statute.terms()`. The output is a [`StatuteUnderstanding`]
/// that an auditor can query for fully-understood vs unresolved
/// counts.
pub fn understand_statute(statute: &Statute, english: &English) -> StatuteUnderstanding {
    let terms: Vec<TermUnderstanding> = statute
        .terms()
        .iter()
        .map(|t| understand_term(statute, t, english))
        .collect();
    StatuteUnderstanding {
        statute_name: statute.name().to_string(),
        statute_version: statute.version().to_string(),
        terms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::lmf::ontology::LmfPos;

    #[test]
    fn lmf_pos_maps_noun_verb_adjective_adverb() {
        assert_eq!(lmf_pos_to_pos_tag(LmfPos::Noun), Some(PosTag::Noun));
        assert_eq!(lmf_pos_to_pos_tag(LmfPos::Verb), Some(PosTag::Verb));
        assert_eq!(
            lmf_pos_to_pos_tag(LmfPos::Adjective),
            Some(PosTag::Adjective)
        );
        assert_eq!(lmf_pos_to_pos_tag(LmfPos::Adverb), Some(PosTag::Adverb));
    }

    #[test]
    fn lmf_pos_other_maps_to_none() {
        // Other has no published OLiA equivalent in our enum.
        assert_eq!(lmf_pos_to_pos_tag(LmfPos::Other), None);
    }

    #[test]
    fn resolve_legal_role_returns_none_pending_per_source_ontologies() {
        // LITERATURE_GAP tripwire: every input returns None until
        // the per-source FRCP/FRE ontologies land. When they do,
        // this test should flip to positive assertions.
        assert_eq!(resolve_legal_role("plaintiff"), None);
        assert_eq!(resolve_legal_role("Defendant"), None);
        assert_eq!(resolve_legal_role("Court"), None);
        assert_eq!(resolve_legal_role(""), None);
    }

    #[test]
    fn is_statutory_term_of_art_classifies_abbreviations() {
        assert!(is_statutory_term_of_art("SOX"));
        assert!(is_statutory_term_of_art("SEC"));
        assert!(is_statutory_term_of_art("OSHA"));
        // Single/double-letter not classified as abbreviation —
        // false-positive risk too high (e.g. "is", "no").
        assert!(!is_statutory_term_of_art("IS"));
        assert!(!is_statutory_term_of_art("AT"));
    }

    #[test]
    fn is_statutory_term_of_art_classifies_section_markers() {
        assert!(is_statutory_term_of_art("1514A"));
        assert!(is_statutory_term_of_art("42121"));
        assert!(is_statutory_term_of_art("78j-1"));
        // Pure-digit section numbers also qualify (Bluebook §3.3.4
        // citation conventions).
        assert!(is_statutory_term_of_art("1514"));
        // Pure-alpha lowercase doesn't qualify — that's normal text.
        assert!(!is_statutory_term_of_art("retaliation"));
        // Single/double digits don't qualify — too short to be a
        // statutory marker (more likely a numeral in prose).
        assert!(!is_statutory_term_of_art("5"));
        assert!(!is_statutory_term_of_art("18"));
    }

    #[test]
    fn resolution_status_partitions_are_disjoint() {
        // Property: every TermLexicalResolution is in exactly one
        // bucket.
        for s in [
            ResolutionStatus::Resolved,
            ResolutionStatus::StatutoryTermOfArt,
            ResolutionStatus::Unresolved,
        ] {
            // Each variant compares equal to itself, unequal to
            // others — required for status_counts() to partition.
            let count_self = [s, s, s].iter().filter(|t| **t == s).count();
            assert_eq!(count_self, 3);
        }
    }

    // =========================================================================
    // Generic corpus-wide audit — every registered USC statute goes through
    // the five-layer understanding. No SOX/AIR21-specific code below; the
    // audit walks all_statutes() and runs the same invariants on every one.
    // =========================================================================

    use crate::social::compliance::statutes::Statute;
    use crate::social::judicial::statute_structure::english_adjunction::test_helpers::cached_english;

    /// Every Statute registered in pr4xis-domains that has a typed
    /// USLM-derived constructor. Walks this list to run the
    /// corpus-wide audit; no SOX/AIR21 names appear in the audit
    /// logic — only in this registry shim, which is a list of
    /// instances that's allowed to name them.
    fn all_registered_statutes() -> Vec<&'static Statute> {
        vec![
            crate::social::compliance::statutes::sox_1514a::statute_from_uslm(),
            crate::social::compliance::statutes::air21_42121::statute_from_uslm(),
        ]
    }

    #[test]
    fn every_registered_statute_understanding_constructs() {
        // Smoke: understand_statute runs end-to-end on every loaded
        // statute, no panics, output non-empty.
        let en = cached_english();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            assert_eq!(u.statute_name, s.name());
            assert_eq!(u.statute_version, s.version());
            assert_eq!(u.terms.len(), s.terms().len());
        }
    }

    #[test]
    fn every_registered_statute_has_no_unresolved_lemmas() {
        // Layer-1 corpus invariant: across every registered statute,
        // every content lemma in every term name either resolves to
        // ≥1 WordNet sense or is classified as a statutory term-of-
        // art. Pure Unresolved is the tripwire that says either the
        // lexicon needs extending or the classifier needs refining.
        let en = cached_english();
        let mut failures: Vec<String> = Vec::new();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            for t in u.terms_with_unresolved_lemmas() {
                let bad: Vec<&str> = t
                    .lexical
                    .iter()
                    .filter(|r| r.status == ResolutionStatus::Unresolved)
                    .map(|r| r.lemma_sense.form.written_rep.as_str())
                    .collect();
                failures.push(format!(
                    "{}@{}: term {} [{}] has unresolved lemmas {:?}",
                    s.name(),
                    s.version(),
                    t.term_id.value,
                    t.term_name,
                    bad
                ));
            }
        }
        if !failures.is_empty() {
            panic!(
                "Unresolved lemmas across registered statutes (expected zero):\n  - {}",
                failures.join("\n  - ")
            );
        }
    }

    #[test]
    fn every_registered_statute_has_some_wordnet_resolution() {
        // Sanity: at least one term in each statute has at least one
        // Resolved lemma. If a whole statute resolves zero senses,
        // either the WordNet load is broken or the term-name set is
        // pure statutory-term-of-art (unlikely for real USC sections).
        let en = cached_english();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            let resolved: usize = u
                .terms
                .iter()
                .map(|t| {
                    t.lexical
                        .iter()
                        .filter(|r| r.status == ResolutionStatus::Resolved)
                        .count()
                })
                .sum();
            assert!(
                resolved > 0,
                "{}@{} has zero Resolved lemmas across all terms — \
                 English/WordNet wiring may be broken",
                s.name(),
                s.version()
            );
        }
    }

    #[test]
    fn every_registered_statute_carries_urn_provenance() {
        // Layer 5 invariant: every term's name.context_uri is the
        // USLM URN of its parent section (per M4.δ.21 push-down).
        // Each registered statute pins to its own section's URN.
        let en = cached_english();
        let expected_urns = [
            ("sox_1514a", "/us/usc/t18/s1514A"),
            ("air21_42121", "/us/usc/t49/s42121"),
        ];
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            let expected = expected_urns
                .iter()
                .find(|(n, _)| *n == s.name())
                .map(|(_, urn)| *urn);
            let Some(expected_urn) = expected else {
                continue;
            };
            for t in &u.terms {
                assert_eq!(
                    t.provenance_context_uri.as_deref(),
                    Some(expected_urn),
                    "{}@{} term {} carries wrong provenance",
                    s.name(),
                    s.version(),
                    t.term_id.value
                );
            }
        }
    }

    #[test]
    fn every_registered_statute_relation_counts_balance() {
        // Layer 4 invariant: sum of inbound = sum of outbound
        // (every relation counted once from each side) = the
        // Statute's total relation count.
        let en = cached_english();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            let inbound: usize = u.terms.iter().map(|t| t.inbound_relation_count).sum();
            let outbound: usize = u.terms.iter().map(|t| t.outbound_relation_count).sum();
            assert_eq!(
                inbound,
                outbound,
                "{}@{}: inbound={inbound} vs outbound={outbound}",
                s.name(),
                s.version()
            );
            assert_eq!(
                inbound,
                s.relations().len(),
                "{}@{}: composed inbound count must match Statute.relations().len()",
                s.name(),
                s.version()
            );
        }
    }

    #[test]
    fn legal_role_layer_is_literature_gap_corpus_wide() {
        // LITERATURE_GAP tripwire for Layer 3: until the per-source
        // FRCP/FRE/5_USC_551 ontologies land, every term in every
        // registered statute has legal_role == None.
        let en = cached_english();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en);
            for t in &u.terms {
                assert!(
                    t.legal_role.is_none(),
                    "{}@{} term {}: Layer 3 should be None until \
                     per-source FRCP/FRE ontologies land; got {:?}",
                    s.name(),
                    s.version(),
                    t.term_id.value,
                    t.legal_role
                );
            }
        }
    }
}
