//! The predicate lexicon — a surface→concept index for phrasal predicates
//! that are NOT structural relations (contrast `relation_lexicon`, whose
//! entries cross through `relations_kind` into Tarski (1941)/OBO-RO's
//! binary relation-type vocabulary: Parthood, Subsumption, etc.).
//! "Eligible for" is a conditional-rule-GOVERNANCE predicate (Minsky 1974
//! frame semantics — is asker X governed by rule Y), not a structural
//! relation between two loaded concepts; forcing it into `RelationsConcept`
//! would be a category error this codebase's own "literature or remove"
//! discipline forbids (no OBO-RO/SKOS precedent treats eligibility as a
//! formal relation TYPE the way part-of or causation are).
//!
//! Consumed the SAME way `relation_lexicon::relation_surface_index`
//! already is: `montague::apply`'s non-copula predicate discriminator only
//! checks `LexicalReasoner::relation_for_surface(word).is_some()` — the
//! returned [`ConceptRef`]'s ontology/name are never actually read
//! downstream of that check, so this index does not need `relations_kind`'s
//! crossing at all; it only needs to name something real.
//!
//! Small and hand-authored in Rust, not loaded `.prx` data, following the
//! SAME precedent `conditional_rule::registry`'s
//! `asset_transfer_penalty_rule` already set: Sergot, Sadri, Kowalski,
//! Kriwaczek, Hammond & Cory (1986) "The British Nationality Act as a Logic
//! Program" grounds this whole capability in a HUMAN reduction of real
//! statutes, not a growing symbolic vocabulary reasoned over by a closure
//! (unlike `RelationsConcept`, nothing folds transitively over this map).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::ontology::ConceptRef;

/// The loaded surface→concept index for rule-governance predicates. Today:
/// `"eligible for"` → `ConditionalRuleFrame`'s `Governance` concept
/// (`crate::social::judicial::conditional_rule::ontology`) — the Frame
/// (Minsky 1974) a `ConditionalRule` question asks about. `"eligible"` ALONE is
/// deliberately absent: without an object it composes fine as a bare
/// predicative adjective (`svo::predicate_adjective`, "the individual is
/// eligible") and needs no relation-surface collapse; only the two-word
/// adjective+PP-complement span needs one, exactly as `relation_lexicon`
/// lexicalizes `"part of"` but not the copula-default `"is a"`.
pub fn predicate_surface_index() -> BTreeMap<String, ConceptRef> {
    let mut index = BTreeMap::new();
    index.insert(
        "eligible for".to_string(),
        ConceptRef::new(
            OntologyName::new_static("ConditionalRuleFrame"),
            "Governance",
        ),
    );
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eligible_for_names_the_governance_frame() {
        let index = predicate_surface_index();
        let target = index.get("eligible for").expect("eligible for is indexed");
        assert_eq!(target.ontology.as_str(), "ConditionalRuleFrame");
        assert_eq!(target.name, "Governance");
    }

    /// The bare adjective (no object) must NOT be indexed — it already
    /// composes as a predicative adjective without a relation-surface
    /// collapse (see the module doc); indexing it would make the
    /// multi-word recognizer wrongly collapse "eligible" alone the way
    /// `relation_lexicon` deliberately omits the copula-default "is a".
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn bare_eligible_is_not_indexed() {
        let index = predicate_surface_index();
        assert_eq!(index.get("eligible"), None);
    }
}
