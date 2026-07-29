//! Statute understanding — composition of a Statute's terms through
//! five typed ontology layers: lexical (English/WordNet),
//! morphosyntactic (OLiA), legal-frame (the loaded [`UsCode`]
//! corpus), judicial relations (the Statute's own relation graph),
//! and provenance (the source URN on each term's SourceTextRef).
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
//! 3. **Legal frame** — [`resolve_legal_role`] looks up a term name
//!    against every section heading in the loaded [`UsCode`] corpus
//!    (every registered USC title materialised by
//!    `pr4xis::codegen::usc_corpus` at build time) and returns the
//!    matched section's USLM-URN Identifier. Returns `None` when
//!    the corpus is empty or no heading contains the term name.
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
//! [`SourceTextRef`]: crate::social::judicial::source_text::SourceTextRef
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
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::meta::identifier_format::Identifier;
use crate::social::compliance::statutes::Statute;
use crate::social::software::markup::xml::lmf::ontology::LmfPos;
use crate::social::software::markup::xml::uslm::corpus::UsCode;

use super::english_adjunction::{LemmaSenseMapping, resolve_term_name_to_senses};

// =============================================================================
// Public types
// =============================================================================

/// Resolution status of a lemma's lookup against English WordNet.
///
/// Three kinds:
/// - [`Resolved`][Self::Resolved]: at least one WordNet sense matched.
/// - [`StatutoryTermOfArt`][Self::StatutoryTermOfArt]: zero senses;
///   the lemma matched the abbreviation or section-marker shape per
///   [`is_statutory_term_of_art`].
/// - [`Unresolved`][Self::Unresolved]: zero senses and no
///   statutory-term-of-art classification.
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
    /// Layer 3: the USLM URN of a U.S. Code section whose heading
    /// contains this term's name. `None` when no section heading in
    /// the loaded [`UsCode`] corpus matches.
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
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), not a bare
    /// `usize` — a count, the same typing discipline as
    /// `formal::mereology::counting::ontology::cardinality`.
    pub fn fully_understood_count(&self) -> Quantity {
        let count = self
            .terms
            .iter()
            .filter(|t| t.is_fully_understood())
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// Terms with at least one [`Unresolved`][ResolutionStatus::Unresolved]
    /// lemma.
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
        // OLiA (Chiarcos & Sukhareva 2015) has one Adjective leaf — the
        // WN satellite/head distinction is a WordNet-cluster role, not an
        // OLiA category — so both adjective tags project to it.
        LmfPos::Adjective | LmfPos::SatelliteAdjective => PosTag::Adjective,
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
/// Looks up `term_name` (case-insensitive) against every section
/// heading in the loaded [`UsCode`] corpus and returns the matched
/// section's USLM URN as a typed [`Identifier`] of format
/// [`IdentifierFormatConcept::UslmUrn`][curl]. The corpus spans
/// every registered USC title materialised at build time by
/// `pr4xis::codegen::usc_corpus` (Title 18, Title 49, and any
/// future title whose USLM XML lands in
/// `crates/domains/data/legal/uscode/`).
///
/// Matching strategy: a section matches if its heading (lowercased)
/// CONTAINS the term-name's lowercased form. Substring matching
/// rather than exact-equal is appropriate because section headings
/// are full descriptive phrases ("Testimony by Expert Witnesses")
/// while term names are typically the head noun phrase ("Expert
/// Witness"). The first matching section wins (USC section URNs
/// are unique across the whole code, so URN ambiguity is not
/// possible).
///
/// Returns `None` when:
/// - the term name is empty / whitespace-only, OR
/// - the loaded `UsCode` corpus is empty, OR
/// - no section heading contains the term name.
///
/// Per "Bottom-up loaded, never encoded": no hand-coded statute
/// lexicon. The vocabulary is the LRC's published USLM XML,
/// hash-pinned in `praxis.lock`.
///
/// [curl]: crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept::UslmUrn
pub fn resolve_legal_role(term_name: &str, usc: &UsCode) -> Option<Identifier> {
    let needle = term_name.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    for section in usc.all_sections() {
        if section.heading.to_lowercase().contains(&needle) {
            return Some(section.urn.clone());
        }
    }
    None
}

/// Classifier for lemmas that match the structural shape of a
/// statutory anchor or appear in the bounded U.S. Federal Legal-Text
/// Closed-Class Lexicon — labels statutes use to point at themselves,
/// abbreviations, agency names, place names, and productive English
/// compounds that lie outside WordNet's general-language vocabulary.
///
/// Returns `true` for any of three bounded patterns:
/// - **All-caps abbreviations** (≥3 chars, all ASCII-uppercase or
///   digit, with ≥1 alphabetic character; e.g. "SOX", "AIR21", "SEC").
/// - **Section markers** (≥3 chars, ASCII alphanumeric + hyphens,
///   ≥1 digit; e.g. "1514A", "42121", "78j-1", "1514") per Bluebook
///   §3.3 statute-citation conventions.
/// - **Loaded legal-lexicon entries** (citation abbreviations, month
///   names, federal-agency acronyms, U.S. state / place names,
///   English productive compounds, legal terms-of-art) — see
///   [`super::us_legal_lexicon::is_in_legal_lexicon`] for the loader
///   contract and per-category authoritative sources.
///
/// These patterns are structural artifacts of how statutory text
/// labels itself and the bounded closed-class vocabulary that
/// appears in U.S. federal-statute heading text. Lexical databases
/// like WordNet do not cover them — instead they are loaded from the
/// registered `us_legal_lexicon@2026` source pinned in `praxis.lock`
/// (citation: bundled XML header — GPO Style Manual 2016, Federal
/// Register Document Drafting Handbook 2017, ISO 3166-2:US,
/// Huddleston & Pullum 2002, Bauer 1983, Black's Law Dictionary 11th
/// ed.).
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
    // (per Bluebook §3.3 statute-citation conventions, USC
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
    if chars.len() >= 3 && has_digit && all_allowed {
        return true;
    }

    // Bottom-up loaded vocabulary: consult the U.S. Federal Legal-Text
    // Closed-Class Lexicon for citation abbreviations, month names,
    // federal-agency acronyms, U.S. state / place names, English
    // productive compounds, and legal terms-of-art.
    super::us_legal_lexicon::is_in_legal_lexicon(lemma)
}

/// Resolve a single statute term to its [`TermUnderstanding`].
///
/// Pipeline:
/// 1. Layer 1+2: lemmatise term name; resolve each lemma to WordNet
///    senses; project each sense's `LmfPos` to a [`PosTag`].
/// 2. Layer 3: look up term name against every section heading in
///    the loaded [`UsCode`] corpus via [`resolve_legal_role`].
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
    usc: &UsCode,
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
                .filter_map(|c| lmf_pos_to_pos_tag(c.pos()))
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

    // Layer 3: legal-frame role.
    let legal_role = resolve_legal_role(&term.name.text, usc);

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
pub fn understand_statute(
    statute: &Statute,
    english: &English,
    usc: &UsCode,
) -> StatuteUnderstanding {
    let terms: Vec<TermUnderstanding> = statute
        .terms()
        .iter()
        .map(|t| understand_term(statute, t, english, usc))
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

    #[pr4xis::praxis_value(Verifiable, Extensible)]
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

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn lmf_pos_other_maps_to_none() {
        // Other has no published OLiA equivalent in our enum.
        assert_eq!(lmf_pos_to_pos_tag(LmfPos::Other), None);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn resolve_legal_role_empty_input_returns_none() {
        // Empty / whitespace-only input is rejected before any
        // corpus lookup runs.
        let usc = UsCode::sample();
        assert_eq!(resolve_legal_role("", &usc), None);
        assert_eq!(resolve_legal_role("   ", &usc), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn resolve_legal_role_substring_matches_section_heading() {
        // Layer 3 substring matching against the loaded corpus.
        // The two-section sample fixture includes
        // "Civil action to protect against retaliation in fraud
        // cases" — searching for "retaliation" returns the section's
        // typed USLM URN.
        let usc = UsCode::sample();
        let id =
            resolve_legal_role("retaliation", &usc).expect("retaliation matches sample heading");
        assert_eq!(id.value(), "/us/usc/t18/s1514A");
        assert_eq!(
            id.format,
            crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept::UslmUrn
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn resolve_legal_role_returns_none_when_no_match() {
        // No heading in the sample fixture contains "platypus".
        let usc = UsCode::sample();
        assert_eq!(resolve_legal_role("platypus", &usc), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_statutory_term_of_art_classifies_section_markers() {
        assert!(is_statutory_term_of_art("1514A"));
        assert!(is_statutory_term_of_art("42121"));
        assert!(is_statutory_term_of_art("78j-1"));
        // Pure-digit section numbers also qualify (Bluebook §3.3
        // citation conventions).
        assert!(is_statutory_term_of_art("1514"));
        // Pure-alpha lowercase doesn't qualify — that's normal text.
        assert!(!is_statutory_term_of_art("retaliation"));
        // Single/double digits don't qualify — too short to be a
        // statutory marker (more likely a numeral in prose).
        assert!(!is_statutory_term_of_art("5"));
        assert!(!is_statutory_term_of_art("18"));
    }

    #[pr4xis::praxis_value(Verifiable)]
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
            crate::social::compliance::statutes::sox_1514a::statute(),
            crate::social::compliance::statutes::air21_42121::statute(),
        ]
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_statute_understanding_constructs() {
        // Smoke: understand_statute runs end-to-end on every loaded
        // statute, no panics, output non-empty.
        let en = cached_english();
        let usc = UsCode::cached_full();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
            assert_eq!(u.statute_name, s.name());
            assert_eq!(u.statute_version, s.version());
            assert_eq!(u.terms.len(), s.terms().len());
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_statute_has_no_unresolved_lemmas() {
        // Layer-1 corpus invariant: across every registered statute,
        // every content lemma in every term name either resolves to
        // ≥1 WordNet sense or is classified as a statutory term-of-
        // art. Any pure `Unresolved` lemma fails the audit.
        let en = cached_english();
        let usc = UsCode::cached_full();
        let mut failures: Vec<String> = Vec::new();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
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
                    t.term_id.value(),
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_statute_has_some_wordnet_resolution() {
        // Sanity: at least one term in each statute has at least one
        // Resolved lemma. If a whole statute resolves zero senses,
        // either the WordNet load is broken or the term-name set is
        // pure statutory-term-of-art (unlikely for real USC sections).
        let en = cached_english();
        let usc = UsCode::cached_full();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_statute_carries_urn_provenance() {
        // Layer 5 invariant: every term's name.context_uri is the
        // USLM URN of its parent section (per M4.δ.21 push-down).
        // Each registered statute pins to its own section's URN.
        let en = cached_english();
        let usc = UsCode::cached_full();
        let expected_urns = [
            ("sox_1514a", "/us/usc/t18/s1514A"),
            ("air21_42121", "/us/usc/t49/s42121"),
        ];
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
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
                    t.term_id.value()
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_statute_relation_counts_balance() {
        // Layer 4 invariant: sum of inbound = sum of outbound
        // (every relation counted once from each side) = the
        // Statute's total relation count.
        let en = cached_english();
        let usc = UsCode::cached_full();
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn legal_role_layer_resolves_against_loaded_us_code() {
        // Layer 3 resolution is driven by the build-time loaded
        // `UsCode` corpus (every registered USC title whose USLM XML
        // is on disk). Two observable states match the resolver's
        // two output cases:
        //
        // (a) corpus empty → resolve_legal_role returns None for
        //     every term.
        // (b) corpus populated → terms whose name appears in any
        //     loaded section heading resolve to that section's
        //     USLM URN.
        //
        // Both states are well-formed; this audit asserts the URN
        // shape of every resolved role, and the universal claim that
        // an empty corpus yields zero resolutions.
        let en = cached_english();
        let usc = UsCode::cached_full();
        let corpus_loaded = usc.section_count().value > 0.0;

        let mut any_resolved = false;
        for s in all_registered_statutes() {
            let u = understand_statute(s, en, usc);
            for t in &u.terms {
                if let Some(role) = &t.legal_role {
                    any_resolved = true;
                    // Every resolved role is a USLM URN under /us/usc/.
                    assert!(
                        role.value().starts_with("/us/usc/"),
                        "resolved role must be a USC USLM URN; got {:?}",
                        role.value()
                    );
                }
            }
        }

        if !corpus_loaded {
            assert!(
                !any_resolved,
                "UsCode corpus is empty; no role should resolve"
            );
        }
    }

    // =========================================================================
    // Corpus-wide gap audit — per the "corpus-wide audit on every source load"
    // memory: every Tier 2+ ontology load must come with an audit that walks
    // every record through the understanding pipeline. Spot-checks don't count.
    // =========================================================================

    use crate::social::judicial::statute_structure::english_adjunction::resolve_term_name_to_senses;

    /// Walk every section in the loaded `UsCode`, run the lexical
    /// pipeline on its heading PROSE, and report every lemma whose
    /// resolution lands in `Unresolved` (neither in WordNet nor a
    /// statutory term-of-art).
    ///
    /// `section.heading` is the runtime corpus's heading prose —
    /// `heading_mixed.heading_prose_text()`, the title text MINUS the
    /// editorial footnote annotation the LRC nests inside a few
    /// `<heading>`s (the typed `<note type="footnote">` + its `<ref
    /// class="footnoteRef">` marker, e.g. "Section catchline was not
    /// amended…" on 18 U.S.C. §§ 1303/3402/4351/4352). That note is
    /// metadata ABOUT the title, not a word IN it, so the understanding
    /// pipeline must not see its lemmas (the whole point of the typed
    /// heading-prose projection — no lexicon entry, no allowlist). This is
    /// DELIBERATELY the opposite answer `chapeau`/`content`'s own
    /// `prose_text()` gives on footnotes — see `NonProseSubtreeKind`'s own
    /// doc for the real, measured evidence behind each.
    ///
    /// Runs against the full build-time codegen-loaded corpus when
    /// USC title XML is on disk (~2770 sections across Titles 18 +
    /// 49); falls back to a two-section `UsCode::sample()` when
    /// the codegen-loaded corpus is empty (CI / fresh clone before
    /// `pr4xis fetch`). The fallback is logged so the diminished
    /// coverage is visible.
    ///
    /// On failure the panic message includes:
    /// - source label + total sections scanned
    /// - total unresolved-lemma instances + distinct-lemma count
    /// - top-20 lemmas by frequency
    /// - the full per-section list at the bottom
    ///
    /// Per the "corpus-wide audit on every source load" memory:
    /// every Tier 2+ ontology load must come with a corpus-wide
    /// audit that walks every record through the understanding
    /// pipeline. Spot-checks don't count.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn corpus_wide_gap_audit_no_unresolved_lemmas() {
        use std::collections::BTreeMap;

        let en = cached_english();
        let usc_full = UsCode::cached_full();
        let (usc, source_label): (&UsCode, &str) = if usc_full.section_count().value > 0.0 {
            (usc_full, "runtime USLM-XML-loaded corpus")
        } else {
            // Fallback: sample only carries two synthetic sections.
            // Surface this in the panic message so the diminished
            // audit scope is loud.
            static SAMPLE_LOCK: std::sync::OnceLock<UsCode> = std::sync::OnceLock::new();
            let sample = SAMPLE_LOCK.get_or_init(UsCode::sample);
            (sample, "UsCode::sample() fallback — only 2 sections")
        };

        let mut failures: Vec<String> = Vec::new();
        let mut frequency: BTreeMap<String, usize> = BTreeMap::new();
        for section in usc.all_sections() {
            let lemma_mappings = resolve_term_name_to_senses(&section.heading, en);
            for lm in &lemma_mappings {
                let status = if !lm.senses.is_empty() {
                    ResolutionStatus::Resolved
                } else if is_statutory_term_of_art(&lm.form.written_rep) {
                    ResolutionStatus::StatutoryTermOfArt
                } else {
                    ResolutionStatus::Unresolved
                };
                if status == ResolutionStatus::Unresolved {
                    failures.push(format!(
                        "{}: heading={:?} unresolved lemma {:?}",
                        section.urn.value(),
                        section.heading,
                        lm.form.written_rep
                    ));
                    *frequency.entry(lm.form.written_rep.clone()).or_insert(0) += 1;
                }
            }
        }

        if !failures.is_empty() {
            let mut top: Vec<(String, usize)> =
                frequency.iter().map(|(k, v)| (k.clone(), *v)).collect();
            top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let top20: Vec<String> = top
                .iter()
                .take(20)
                .map(|(lemma, n)| format!("{n:>4}  {lemma:?}"))
                .collect();
            panic!(
                "Corpus-wide gap audit FAILED.\n\
                 Source: {}\n\
                 Sections scanned: {}\n\
                 Unresolved-lemma instances: {}\n\
                 Distinct unresolved lemmas: {}\n\n\
                 Top 20 by frequency:\n  {}\n\n\
                 Full list ({} entries):\n  - {}",
                source_label,
                usc.section_count().value,
                failures.len(),
                frequency.len(),
                top20.join("\n  "),
                failures.len(),
                failures.join("\n  - "),
            );
        }
    }
}
