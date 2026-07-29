//! The comparison-relation lexicon — a surface→concept index for the closed
//! set of DERIVED relational-noun heads that license an overt
//! multi-participant PP complement ("the difference BETWEEN X and Y"),
//! distinct from `relation_lexicon` (a structural relation VERIFIED against
//! a materialized closure, e.g. "X part of Y") and from `predicate_lexicon`
//! (a rule-governance predicate, e.g. "eligible for").
//!
//! A "difference between X and Y" question is not a fact to check against a
//! loaded closure — no edge exists (or should exist) between two arbitrary
//! defined terms in a materialized closure recording that they "differ" —
//! it is a request to recite each named term's own gloss side by side. Kept
//! a SEPARATE trait method/index for exactly that reason: folding
//! "difference" into `relation_lexicon` would make it visible to
//! `relation_for_surface`'s two live closure-verification call sites
//! (`montague::apply`'s S-result predicative-complement branch;
//! `pr4xis_chat::answer_question`'s two-entity relational-claim check) and
//! silently misroute it into relation verification (predicate="difference",
//! no closure edge → false/abstain, never the two-gloss recital it should
//! produce).
//!
//! # Literature
//!
//! Barker, C. (2011) "Possessives and Relational Nouns," in Maienborn, von
//! Heusinger & Portner (eds.), *Semantics: An International Handbook of
//! Natural Language Meaning*, Berlin: de Gruyter Mouton, ch. 45 (cited here
//! from the freely-hosted 2008 preprint's own pagination, pp. 1108-1129,
//! §1.4 "Derived versus underived relational nominals," p.4): "non-derived
//! nouns appear to have a strict upper limit of two on the number of
//! overtly expressible participants... Derived nominals can have elaborate
//! argument structures inherited from their verbal source, e.g., *the
//! purchase of the property by the woman for a pittance*." "difference"
//! (deverbal from "differ (from)") is a DERIVED relational noun in exactly
//! this sense — contrast Barker's own UNDERIVED worked examples "brother",
//! "enemy", "sake", each capped at one overt complement (the "Secretary of
//! Commerce" shape `montague::apply`'s own regression tests pin). Verified
//! by direct fetch: semanticsarchive.net/Archive/WYxOTc5M/barker-possessives.pdf.
//!
//! # Why hand-authored Rust, not loaded `.prx` data
//!
//! Small and hand-authored, not loaded `.prx` data — the SAME precedent
//! `predicate_lexicon` sets, for the same reason: the returned
//! [`ConceptRef`]'s `ontology`/`name` are never read downstream of the
//! `is_some()` gate (`montague::apply`'s NP-result branch checks only
//! `Some`/`None`; [`crate::cognitive::linguistics::lambek::montague::Sem::comparison_leaves`]
//! carries the kind through only as an unread tuple element), so this index
//! does not need `relation_lexicon`'s content-addressed archive machinery
//! to "only... name something real" (`predicate_lexicon`'s own module doc).
//! It points at the Relations vocabulary's own already-declared, already-
//! cited `Association` concept (Smith et al. 2005 *Genome Biology* 6:R46 /
//! SKOS `related`, W3C 2009: "Uncommitted fallback when no stronger
//! relation applies... carries no other structural claim") — the honest
//! coarse label for "these named things stand in some noteworthy relation
//! the asker wants explained," without overclaiming the antonym-specific
//! `Opposition` kind (Saussure 1916; Cruse 1986 — lexical polarity, not what
//! a "difference between X and Y" question asks about) or forcing a new
//! closure-verified relation kind into the CLOSED twelve-member vocabulary
//! (`formal::relations::ontology`'s own module doc: "twelve canonical
//! binary relation types").

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

use pr4xis_runtime::ontology::{ConceptRef, relations_kind};

/// The loaded surface→concept index for derived-relational-noun heads that
/// license a multi-participant PP complement. Today: `"difference"` → the
/// Relations vocabulary's `Association` concept (see module doc for why
/// that target, and why this stays a separate index from
/// `relation_lexicon`).
pub fn comparison_relation_surface_index() -> BTreeMap<String, ConceptRef> {
    let mut index = BTreeMap::new();
    index.insert("difference".to_string(), relations_kind("Association"));
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn difference_is_indexed_as_a_comparison_relation() {
        let index = comparison_relation_surface_index();
        assert_eq!(
            index.get("difference"),
            Some(&relations_kind("Association")),
            "difference (Barker 2011 derived relational noun) indexes to the \
             Relations vocabulary's uncommitted-relatedness fallback"
        );
    }

    /// The "Secretary of Commerce" precedent's head noun must NOT be
    /// indexed: "secretary" is an ordinary PP-modified NP, not a
    /// Barker-derived relational noun — this is the direct regression pin
    /// for `montague::apply`'s own guard (see
    /// `a_derived_prepositional_np_np_does_not_trigger_the_apposition_guard`).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_unregistered_head_is_not_indexed() {
        let index = comparison_relation_surface_index();
        assert_eq!(index.get("secretary"), None);
    }
}
