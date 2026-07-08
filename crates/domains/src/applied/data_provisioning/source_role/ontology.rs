//! Source role — the `prov:Role` a registered data source plays with respect
//! to the activity that consumes it.
//!
//! # The distinction this ontology draws
//!
//! The data-provisioning registry ([`super::super::registry`]) enumerates
//! *every* managed external source praxis hash-pins and can materialize —
//! WordNet, the SPAR OWL vocabularies, the U.S. Code titles, but *also* the
//! Adobe Glyph List, the W3C XSDs, the OOXML schema archive, the Base16
//! colour-scheme collection, the conformance test suites. These are not one
//! population. A glyph list is consumed by a *decoding* activity (it grounds
//! the PDF `/Differences` resolver); WordNet is consumed by the *reasoner*
//! as chat knowledge. Counting them all as "available knowledge" is the bug
//! this ontology fixes: it gives each registered source its **role** — the
//! kind of activity that consumes it — so the knowledge-boundary catalog can
//! count only the sources a user can actually reason over.
//!
//! # Literature — PROV-O roles
//!
//! Lebo, Sahoo & McGuinness (2013), *PROV-O: The PROV Ontology*, W3C
//! Recommendation 30 April 2013, define **`prov:Role`** as "the function of
//! an entity with respect to an activity in which it is used or generated"
//! (§3, `prov:hadRole`), and **`prov:used`** (§2.1.2) as the relation from an
//! `prov:Activity` to the entity it consumed. A source's role is therefore
//! *which activity uses it*:
//!
//! - [`SourceRoleConcept::ChatKnowledge`] — used by the **reasoning**
//!   activity (the source is loaded as an ontology the reasoner queries).
//! - [`SourceRoleConcept::DecoderInput`] — used by a **decoding /
//!   provisioning** activity (the source grounds or decodes *other* sources;
//!   `prov:used` by a decoder, never by the reasoner directly).
//! - [`SourceRoleConcept::NotYetLoadable`] — used by **no** activity yet
//!   (registered and hash-pinned, but its canonical encoding has no runtime
//!   decoder, so no activity can consume it — a role that is, for now, empty).
//!
//! The three roles partition the registry (every registered source has
//! exactly one), mirroring how [`super::super::ontology::canonical_encoding`]
//! assigns exactly one `ContentType` per kind. The partition is realized by
//! the [`super::functor::SourceKindToRole`] functor and machine-checked by
//! [`EveryRegisteredKindHasConcreteRole`](super::functor::EveryRegisteredKindHasConcreteRole).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SourceRole",
    source: "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation 30 April 2013",

    concepts: [
        // Root — the abstract function an entity has w.r.t. a consuming activity.
        SourceRole,

        // The three concrete roles (leaves) partitioning the registry.
        ChatKnowledge,
        DecoderInput,
        NotYetLoadable,
    ],

    labels: {
        SourceRole: ("en", "Source role",
            "Lebo, Sahoo & McGuinness (2013) PROV-O §3 `prov:Role`: the function of a registered source with respect to the activity that uses it. The abstract root partitioned by the three concrete roles."),
        ChatKnowledge: ("en", "Chat knowledge",
            "A source `prov:used` by the REASONING activity — loaded as a live ontology the reasoner queries (WordNet, the SPAR OWL vocabularies, U.S. Code titles, the legal lexicon). The only role a chat user can reach as loadable knowledge."),
        DecoderInput: ("en", "Decoder input",
            "A source `prov:used` by a DECODING / PROVISIONING activity — it grounds or decodes OTHER sources rather than being reasoned over (an XSD/DTD schema, the Adobe Glyph List, a conformance test suite, a colour-scheme collection, the closed-class lexical substrate of the language engine). First-class, hash-pinned, and verified — but internal provisioning substrate, not browsable knowledge."),
        NotYetLoadable: ("en", "Not yet loadable",
            "A registered, hash-pinned source `prov:used` by NO activity yet — its canonical encoding has no runtime decoder (the PDF-published federal statutes, procedural rules, and case law awaiting a PDF loader). A role that is presently empty; it re-activates automatically when a decoder for its encoding lands."),
    },

    is_a: [
        // The three roles partition the abstract SourceRole.
        (ChatKnowledge, SourceRole),
        (DecoderInput, SourceRole),
        (NotYetLoadable, SourceRole),
    ],

    opposes: [
        // The three roles are pairwise mutually exclusive — a source has
        // exactly one role (PROV-O: one function w.r.t. the consuming activity).
        (ChatKnowledge, DecoderInput),
        (DecoderInput, ChatKnowledge),
        (ChatKnowledge, NotYetLoadable),
        (NotYetLoadable, ChatKnowledge),
        (DecoderInput, NotYetLoadable),
        (NotYetLoadable, DecoderInput),
    ],
}

// ---------------------------------------------------------------------------
// Quality — is this role the chat-loadable one?
// ---------------------------------------------------------------------------

/// Quality: whether a role means "the reasoner can load and query this source
/// as knowledge". Only [`SourceRoleConcept::ChatKnowledge`] returns true;
/// `DecoderInput` and `NotYetLoadable` are false. The root abstract
/// [`SourceRoleConcept::SourceRole`] is undecided (`None`).
///
/// This is the concept-level grounding of
/// [`super::functor::is_chat_loadable`]: the entry-level predicate is the
/// composite `IsChatLoadable ∘ SourceKindToRole` applied to a registry entry.
#[derive(Debug, Clone)]
pub struct IsChatLoadable;

impl Quality for IsChatLoadable {
    type Individual = SourceRoleConcept;
    type Value = bool;

    fn get(&self, concept: &SourceRoleConcept) -> Option<bool> {
        use SourceRoleConcept as R;
        match concept {
            R::ChatKnowledge => Some(true),
            R::DecoderInput | R::NotYetLoadable => Some(false),
            R::SourceRole => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical string <-> concept (parser / wire boundary)
// ---------------------------------------------------------------------------

/// Canonical string for a role concept — the stable wire label the
/// self-model surface renders and the inverse of [`parse_role`].
pub fn role_name(c: SourceRoleConcept) -> &'static str {
    use SourceRoleConcept as R;
    match c {
        R::SourceRole => "SourceRole",
        R::ChatKnowledge => "ChatKnowledge",
        R::DecoderInput => "DecoderInput",
        R::NotYetLoadable => "NotYetLoadable",
    }
}

/// Parse a role name into its typed variant. `None` if `s` names no role
/// (fail-closed, no silent default).
pub fn parse_role(s: &str) -> Option<SourceRoleConcept> {
    use SourceRoleConcept as R;
    Some(match s {
        "SourceRole" => R::SourceRole,
        "ChatKnowledge" => R::ChatKnowledge,
        "DecoderInput" => R::DecoderInput,
        "NotYetLoadable" => R::NotYetLoadable,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for SourceRoleOntology {
    type Cat = SourceRoleCategory;
    type Qual = IsChatLoadable;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        // The registry-facing invariants live in `functor.rs` (they consult
        // the functor + registry); include it here so `validate()` runs the
        // complete axiom set for this ontology. It carries the real,
        // registry-grounded coverage — every registered source maps to exactly
        // one concrete role, no leaf maps to the abstract root. (The former
        // `RolePartitionIsTotal` was logically equivalent — both `Ok` iff no kind
        // maps to the abstract root — and carried a vacuous role-count-sum leg a
        // total exhaustive match always satisfies, so it was collapsed into this
        // one. The earlier `ChatKnowledgeIsTheSoleLoadableRole` was a tautology:
        // its `verify()` merely re-read `IsChatLoadable::get`'s own match arms with
        // no independent ground — loadability IS the role classification — so it
        // too was removed rather than shipped as `f == def`. The sole-loadable
        // fact is covered by the `only_chat_knowledge_is_loadable` and
        // `is_chat_loadable_selects_only_chat_knowledge_entries` tests.)
        axioms.push(Box::new(super::functor::EveryRegisteredKindHasConcreteRole));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SourceRoleCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SourceRoleOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_concepts() {
        // Root + three concrete roles.
        assert_eq!(SourceRoleConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn only_chat_knowledge_is_loadable() {
        use SourceRoleConcept as R;
        let q = IsChatLoadable;
        assert_eq!(q.get(&R::ChatKnowledge), Some(true));
        assert_eq!(q.get(&R::DecoderInput), Some(false));
        assert_eq!(q.get(&R::NotYetLoadable), Some(false));
        // The abstract root is undecided.
        assert_eq!(q.get(&R::SourceRole), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn role_name_round_trips_through_parse_role() {
        for c in SourceRoleConcept::variants() {
            assert_eq!(parse_role(role_name(c)), Some(c));
        }
        assert_eq!(parse_role("NotARole"), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_concrete_role_is_a_source_role() {
        // The three concrete roles each reach the abstract root via is_a —
        // the partition is well-formed.
        use pr4xis::category::{Arrow, Category};
        let subsumptions: Vec<_> = SourceRoleCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceRoleRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        use SourceRoleConcept as R;
        for role in [R::ChatKnowledge, R::DecoderInput, R::NotYetLoadable] {
            assert!(
                subsumptions.contains(&(role, R::SourceRole)),
                "{role:?} must is_a SourceRole"
            );
        }
    }
}
