//! The verbal-relation lexicon — a surface→concept index for phrasal
//! relational predicates that are VERB-HEADED (carry their own tense:
//! "counts"/"count") rather than copula-complement-headed like
//! [`relation_lexicon`](super::relation_lexicon)'s "part of" or
//! [`predicate_lexicon`](super::predicate_lexicon)'s "eligible for".
//!
//! "Is X part of Y" and "X is eligible for Y" both route the copula "is"
//! through a PREDICATIVE-COMPLEMENT reading of the relational phrase
//! (`svo::relational_predicate`, `(S[adj]\NP)`-shaped): the relation surface
//! is the complement, "is" stays syncategorematic. A light verb like
//! "counts as" is different — there is no copula at all; "counts"/"count"
//! IS the finite verb of the clause ("Does EVV training count for
//! continuing education?", "What counts as a manual entry?"). Collapsing it
//! to a `relational_predicate` reading would type it for the wrong syntactic
//! slot; it needs the ordinary transitive-verb shape
//! (`svo::transitive_verb`/`svo::bare_transitive_verb`) instead, so
//! do-support/modal subject-aux inversion and ordinary declarative
//! composition both reach it the same way any other transitive verb is
//! reached. `montague::apply`'s ordinary transitive-verb application then
//! produces `Sem::Question{ predicate: "counts as", arguments: [X, Y] }` /
//! `Sem::Prop{...}` for free — no dedicated composition rule, matching
//! `relation_lexicon`'s own "no montague.rs edits needed" precedent, just a
//! different LEXICAL shape upstream of it.
//!
//! Consumed the SAME way
//! [`predicate_lexicon::predicate_surface_index`](super::predicate_lexicon::predicate_surface_index)
//! already is — merged into `composed.rs`'s `relation_surface_index`, the
//! ONE index [`LexicalReasoner::relation_for_surface`](super::english::LexicalReasoner::relation_for_surface)
//! reads — so once `chat/src/lib.rs`'s multi-word collapse closure offers
//! `[transitive_verb(), bare_transitive_verb()]` for a surface registered
//! here (see `verbal_relation_for_surface`), `answer_question`/
//! `answer_statement`'s existing `relation_for_surface(predicate)` lookup
//! resolves the SAME [`ConceptRef`] this module names, with no separate
//! wiring.
//!
//! Small and hand-authored in Rust, not loaded `.prx` data — the SAME
//! precedent [`predicate_lexicon`](super::predicate_lexicon) and
//! [`scope_predicate_lexicon`](super::scope_predicate_lexicon)
//! already set: a small, closed, non-transitively-reasoned-over surface set
//! that only needs to name something real, not a growing symbolic
//! vocabulary a closure reasons transitively over.
//!
//! # Authority (every surface cited)
//!
//! - `"count as"` / `"counts as"` → the [`MemberOf`](crate::formal::relations::ontology::RelationsConcept::MemberOf)
//!   relation kind (reused, not a new kind — Build-phase check per this
//!   task's design confirmed MemberOf's own structural axioms, irreflexive/
//!   non-symmetric/non-transitive-at-this-kind-alone, genuinely hold for
//!   "counts as"): Searle, J. (1995), *The Construction of Social
//!   Reality*, Free Press — the canonical "X counts as Y in context C"
//!   formula for institutional classification; Jones, A. J. I. & Sergot,
//!   M. (1996), "A Formal Characterisation of Institutionalised Power",
//!   *Logic Journal of the IGPL* 4(3):427-443 — the standard AI/formal-
//!   ontology treatment of counts-as as a conditional operator. An
//!   individual counting as a member of an institutional classification
//!   ("EVV training" counting as "continuing education") is exactly the
//!   individual-to-classification relation `MemberOf` already grounds
//!   (verb entries in a VerbNet class), not a category error to reuse it
//!   for.
//!
//! `"count for"` / `"counts for"` is DELIBERATELY ABSENT: Cambridge
//! Dictionary and Dictionary.com both attest "count for" as a genuinely
//! DIFFERENT sense from "count as" — worth/contribution ("count for
//! nothing"), not classification/equivalence — but an independent
//! literature search (this task, 2026-07-20) found no formal-ontology
//! citation grounding a "counts toward/credits toward a requirement"
//! relation the way Searle/Jones & Sergot ground counts-as. Per this
//! codebase's own citation discipline ("if you cannot verify one, say so
//! rather than fabricate it"), it is left out rather than force-mapped
//! either onto `MemberOf` (wrong sense) or a fabricated new kind — an
//! honest, tracked residual, not a silent gap.
//!
//! - `"take the place of"` → the [`Supersession`](crate::formal::relations::ontology::RelationsConcept::Supersession)
//!   relation kind (a NEW kind — Opposition doesn't fit: replacement is
//!   DIRECTIONAL, "EVV takes the place of claims submissions" does not mean
//!   the reverse, whereas Opposition is symmetric; no existing kind fits).
//!   Grounded in DCMI Metadata Terms' `dcterms:replaces`/`isReplacedBy` (see
//!   [`RelationsConcept::Supersession`](crate::formal::relations::ontology::RelationsConcept::Supersession)'s
//!   own label for the full citation) for the RELATION kind, and
//!   Merriam-Webster's dictionary entry "take
//!   the place of (someone or something)" = "to replace" for the IDIOM's
//!   surface meaning — the same two-tier citation (standard vocabulary for
//!   the relation-kind authority, dictionary for the idiom's meaning)
//!   `relation_lexicon`'s own "part of" entry already uses (BFO for the
//!   kind, ordinary English usage for the surface). Four words — within
//!   `max_surface_words()` (derived from every registered surface's own
//!   word count, this index included).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

use pr4xis_runtime::ontology::ConceptRef;

use crate::formal::relations::ontology::{member_of_relation_kind, supersession_relation_kind};

/// The loaded surface→concept index for verb-headed relational predicates.
/// Today: `"count as"` and `"counts as"` (base and third-singular finite
/// forms — this closed set carries no morphological generalization of its
/// own, the SAME "both surface forms are registered explicitly" precedent
/// [`scope_predicate_lexicon::scope_predicate_surfaces`](super::scope_predicate_lexicon::scope_predicate_surfaces)
/// already sets for "fall under"/"falls under") → the [`member_of_relation_kind`];
/// and `"take the place of"` → the [`supersession_relation_kind`] (a single
/// closed idiom — Merriam-Webster attests no comparably common variant
/// worth registering alongside it, unlike "count as"'s tense pair).
pub fn verbal_relation_surface_index() -> BTreeMap<String, ConceptRef> {
    let mut index = BTreeMap::new();
    let member_of = member_of_relation_kind();
    for surface in ["count as", "counts as"] {
        index.insert(surface.to_string(), member_of.clone());
    }
    index.insert(
        "take the place of".to_string(),
        supersession_relation_kind(),
    );
    index
}

/// Every registered verb-headed relational surface — the closed set
/// `chat/src/lib.rs`'s multi-word collapse closure probes to decide whether
/// a span deserves the verb-shaped reading (`[transitive_verb(),
/// bare_transitive_verb()]`) rather than the copula-complement
/// `relational_predicate` reading `relation_lexicon`/`scope_predicate_lexicon`
/// surfaces get. Derived from [`verbal_relation_surface_index`] (never a
/// second, independently-maintained literal set).
pub fn verbal_relation_surfaces() -> alloc::collections::BTreeSet<String> {
    verbal_relation_surface_index().into_keys().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn count_as_and_counts_as_name_the_member_of_kind() {
        let index = verbal_relation_surface_index();
        let expected = member_of_relation_kind();
        for surface in ["count as", "counts as"] {
            let target = index
                .get(surface)
                .unwrap_or_else(|| panic!("{surface:?} must be indexed"));
            assert_eq!(target, &expected);
        }
        assert_eq!(index.len(), 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn take_the_place_of_names_the_supersession_kind() {
        let index = verbal_relation_surface_index();
        let target = index
            .get("take the place of")
            .expect("\"take the place of\" must be indexed");
        assert_eq!(target, &supersession_relation_kind());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn take_the_place_of_is_four_words() {
        assert_eq!("take the place of".split_whitespace().count(), 4);
    }

    /// "count for"/"counts for" are deliberately NOT indexed — see this
    /// module's own doc for the citation search that came up empty.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn count_for_is_not_indexed() {
        let index = verbal_relation_surface_index();
        for surface in ["count for", "counts for", "count", "counts"] {
            assert_eq!(index.get(surface), None, "{surface:?} must not be indexed");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn verbal_relation_surfaces_matches_the_index_keys() {
        let index = verbal_relation_surface_index();
        let surfaces = verbal_relation_surfaces();
        assert_eq!(surfaces.len(), index.len());
        for surface in index.keys() {
            assert!(surfaces.contains(surface));
        }
    }
}
