//! The presupposition-trigger lexicon — the closed set of English temporal
//! subordinating conjunctions whose complement clause is PRESUPPOSED
//! (backgrounded as already true, not asserted) when the clause is fronted:
//! "Since a seal is a mammal, …" commits the speaker to "a seal is a mammal"
//! regardless of how the main clause resolves.
//!
//! Small and hand-authored in Rust, not loaded `.prx` data — the SAME
//! precedent [`predicate_lexicon`](super::predicate_lexicon) already set: a
//! small, closed, non-transitively-reasoned-over surface set that only needs
//! to name something real.
//!
//! Authority: Stanford Encyclopedia of Philosophy, "Presupposition" (Beaver &
//! Geurts, Winter 2014 ed.), §1, identifies "temporal clauses headed by
//! 'before', 'after', 'since', etc." as a presupposition-trigger class,
//! citing Beaver & Condoravdi (2003) "A Uniform Analysis of Before and
//! After", *Proceedings of SALT 13*, and Heinämäki (1974) "Semantics of
//! English Temporal Connectives", PhD dissertation, University of Texas at
//! Austin.
//!
//! Karttunen (1973) "Presuppositions of Compound Sentences", *Linguistic
//! Inquiry* 4(2):169-193, establishes the hole/plug/filter machinery this
//! trigger class feeds into, but its own worked examples are the logical
//! connectives "and"/"or"/"if...then" and factive/implicative attitude verbs
//! (say, tell, believe) — never "since"/"before"/"after" by name. Cited here
//! only for the general apparatus, not as authority for this specific
//! surface set (a mis-citation caught and corrected during this session).
//!
//! Only "since" is currently registered: it is the one member of this class
//! already tagged `subcat="SubordinatingConjunction"` in the closed-class
//! lexicon (`data/function-words/english.xml`). "before"/"after" are
//! currently lexicalized there only as prepositions (`adp`) — their clausal
//! (temporal-connective) sense needs a second `<Sense>` entry before they can
//! be added here; a tracked follow-up, not silently dropped.

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

/// The verified temporal-clause presupposition-trigger surfaces.
pub fn presupposition_trigger_surfaces() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    set.insert("since".to_string());
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn since_is_a_registered_temporal_presupposition_trigger() {
        assert!(presupposition_trigger_surfaces().contains("since"));
    }

    /// "because" is deliberately absent: the verified citation (SEP,
    /// Beaver & Condoravdi 2003, Heinämäki 1974) grounds "before"/"after"/
    /// "since" as TEMPORAL-clause triggers; it does not establish causal
    /// "because"-clauses as presupposition triggers, and Karttunen (1973)
    /// itself never analyzes "because" either. Literature-or-remove.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn because_is_not_included_absent_a_verified_citation() {
        assert!(!presupposition_trigger_surfaces().contains("because"));
    }
}
