//! The scope-predicate lexicon — the closed set of English phrasal predicates
//! that ask a set-membership/scope-coverage question ("is X subject to Y",
//! "does Y cover X") without naming a new relation TYPE: each one asserts the
//! SAME thing "X is a Y" does (X falls within the extension Y denotes), just
//! through a different surface. Contrast [`relation_lexicon`](super::relation_lexicon),
//! whose entries cross into a genuinely distinct relation kind (Parthood is
//! not Subsumption); nothing here does that — these surfaces are pure
//! TOKENIZATION triggers for the existing Subsumption default.
//!
//! Deliberately NOT folded into `relation_lexicon::relation_surface_index`
//! (nor `predicate_lexicon::predicate_surface_index`, which the composed
//! reasoner merges into that SAME index — see `composed.rs`'s
//! `relation_surface_index` construction): both of those feed
//! [`LexicalReasoner::surface_for_relation`](super::english::LexicalReasoner::surface_for_relation),
//! the INVERSE lookup that phrases a relational affirmation ("X is part of
//! Y"). Registering a Subsumption-pointing entry there once broke it network-
//! wide — `surface_for_relation(&subsumption_kind())` is a documented `None`
//! invariant (the copula default has no lexicalized connective), and every
//! affirmative is-a/directional-No answer in the system reads it. This
//! lexicon is consulted ONLY by the chat pipeline's multi-word collapse
//! closure (`collapse_multiword_surfaces`'s callback in `crates/chat/src/
//! lib.rs`), which needs nothing more than "does this span deserve a
//! `relational_predicate` chart type" — never a `ConceptRef`, so it cannot
//! pollute the reverse lookup even by construction.
//!
//! `answer_question`/`answer_statement`'s existing fallback,
//! `relation_for_surface(predicate).unwrap_or_else(subsumption_kind)`,
//! already resolves an un-lexicalized predicate to Subsumption — exactly the
//! semantics every surface here needs, since none of them names a relation
//! `relation_lexicon` doesn't already carry.
//!
//! Small and hand-authored in Rust, not loaded `.prx` data — the SAME
//! precedent [`predicate_lexicon`](super::predicate_lexicon) and
//! [`presupposition_trigger_lexicon`](super::presupposition_trigger_lexicon)
//! already set: a small, closed, non-transitively-reasoned-over surface set
//! that only needs to name something real.
//!
//! Authority: five of the six surfaces are adjective/past-participle heads
//! taking a governed preposition complement — the SAME complementation shape
//! `predicate_lexicon` already lexicalizes for "eligible for" — and the
//! sixth ("fall"/"falls under") is a prepositional verb; both categories are
//! standard, well-documented English predicate-complementation patterns
//! (Quirk, Greenbaum, Leech & Svartvik (1985) *A Comprehensive Grammar of the
//! English Language*, Longman — adjective complementation by a prepositional
//! phrase, and prepositional verbs, are both treated at length there).
//! No new relation semantics is claimed for any of them — see the module doc
//! above.

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

/// The registered scope/coverage-query surfaces, every one a two-word
/// adjective/participle/verb + governed-preposition span (Quirk et al. 1985):
/// - `"subject to"`, `"applicable to"` — adjective + `to`
/// - `"required for"` — past participle + `for`
/// - `"covered under"`, `"included in"` — past participle + `under`/`in`
/// - `"fall under"`, `"falls under"` — prepositional verb (base and 3sg forms;
///   English inflection is not lexicalized elsewhere in this closed set, so
///   both surface forms are registered explicitly)
pub fn scope_predicate_surfaces() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for surface in [
        "subject to",
        "required for",
        "applicable to",
        "covered under",
        "fall under",
        "falls under",
        "included in",
    ] {
        set.insert(surface.to_string());
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_lexicon_carries_the_seven_registered_scope_surfaces() {
        let set = scope_predicate_surfaces();
        for surface in [
            "subject to",
            "required for",
            "applicable to",
            "covered under",
            "fall under",
            "falls under",
            "included in",
        ] {
            assert!(set.contains(surface), "{surface:?} must be registered");
        }
        assert_eq!(set.len(), 7);
    }

    /// Every surface is exactly two whitespace-separated words — the width
    /// `relation_lexicon`'s "part of" already forces `max_surface_words()` to
    /// admit, so no reasoner needs to widen its scan window for this set.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn every_surface_is_two_words() {
        for surface in scope_predicate_surfaces() {
            assert_eq!(
                surface.split_whitespace().count(),
                2,
                "{surface:?} is not two words"
            );
        }
    }

    /// The bare adjective/verb (no governed preposition) is deliberately
    /// absent — same discipline `predicate_lexicon` documents for bare
    /// "eligible": without its complement it composes as an ordinary
    /// predicate and needs no collapse.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn bare_heads_without_their_preposition_are_not_indexed() {
        let set = scope_predicate_surfaces();
        for bare in [
            "subject",
            "required",
            "applicable",
            "covered",
            "fall",
            "included",
        ] {
            assert!(!set.contains(bare), "{bare:?} must not be indexed alone");
        }
    }
}
