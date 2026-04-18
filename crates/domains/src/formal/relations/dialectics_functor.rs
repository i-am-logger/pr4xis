//! Cross-functor: Relations → Dialectics (issue #152).
//!
//! The Relations umbrella enumerates ten canonical binary relation
//! types — Subsumption, Parthood, Causation, Opposition, Similarity,
//! Precedence, Equivalence, Specialisation, Dependence, Association
//! — plus seven algebraic structural properties. Dialectics is the
//! richer philosophical framework that extends one of those relations
//! — Opposition — with Aristotelian + Hegelian + Priestian vocabulary.
//!
//! This functor proves the formal correspondence: every Relations
//! concept has an image in Dialectics. The mapping is deliberately
//! shallow — it shows the *semantic connection* between the two
//! ontologies without claiming every nuance of dialectics is
//! derivable from bare relation structure.
//!
//! # The mapping
//!
//! | Relations concept | Dialectics concept | Reason |
//! |---|---|---|
//! | `Opposition` | `DialecticalMoment` | The core bridge: binary opposition IS a dialectical moment (Aristotle Square + Hegel triad meet here) |
//! | `Equivalence` | `Synthesis` | In Hegel, Synthesis reconciles Thesis and Antithesis, producing a higher identity — which the Equivalence relation formalises |
//! | `Subsumption` | `DialecticalArgument` | Aristotelian syllogism reasons by subsumption; dialectical argument is Aristotelian reasoning about endoxa |
//! | `Specialisation` | `DeterminateNegation` | To specialise a genus is to determinately negate its generality (Hegel Logic §§80–82) |
//! | `Parthood` | `DeterminateNegation` | Taking a part is a determinate negation of the whole's unity |
//! | `Causation` | `Sublation` | Causal transitions are sublation-like: cause is preserved-in-effect, negated-as-bare-cause, elevated-into-result |
//! | `Precedence` | `Sublation` | Temporal precedence is the formal shadow of sublation's generative direction |
//! | `Dependence` | `Contradiction` | Ontological dependence creates internal tension between dependent and depended-on (Marx's internal contradiction) |
//! | `Similarity` | `NonIdentity` | Similarity holds between non-identical things — Adorno's non-identity residue that Synthesis fails to absorb |
//! | `Association` | `Endoxa` | Uncommitted "related-to" is the endoxa of relation-space: commonly-accepted connections without specific claims |
//! | `RelationType` | `DialecticalArgument` | The abstract parent maps to Aristotle's abstract reasoning-form |
//! | `StructuralProperty` | `DialecticalArgument` | Algebraic properties are abstract reasoning-forms about relations |
//! | `Symmetric` | `Contrary` | Symmetric relations mirror like Aristotelian contraries |
//! | `Antisymmetric` | `Contradictory` | Antisymmetric relations exclude like contradictories |
//! | `Transitive` | `Sublation` | Chain-closure is the transitive move Hegel's Sublation formalises |
//! | `Reflexive` | `Thesis` | Reflexivity = identity posited; Hegel's Thesis |
//! | `Irreflexive` | `DeterminateNegation` | Irreflexivity = identity-with-self denied |
//! | `Functional` | `DialecticalArgument` | Functional relations are single-valued reasoning rules |
//! | `Involutive` | `Sublation` | Involution = Sublation-as-round-trip (Aufhebung applied twice returns the origin) |
//!
//! The `Opposition ↦ DialecticalMoment` line is the load-bearing
//! claim. Everything else is supporting structure.
//!
//! Source: Relations ontology (this PR) + Dialectics ontology
//! (formal/logic/dialectics, already in workspace).

use pr4xis::category::{Category, Functor};

use super::ontology::{
    RelationsCategory, RelationsConcept, RelationsRelation, RelationsRelationKind,
};
use crate::formal::logic::dialectics::ontology::{
    DialecticsCategory, DialecticsConcept, DialecticsRelation, DialecticsRelationKind,
};

fn map_concept(c: &RelationsConcept) -> DialecticsConcept {
    use DialecticsConcept as D;
    use RelationsConcept as R;
    match c {
        // Relation types
        R::Opposition => D::DialecticalMoment,
        R::Equivalence => D::Synthesis,
        R::Subsumption => D::DialecticalArgument,
        R::Specialisation | R::Parthood => D::DeterminateNegation,
        R::Causation | R::Precedence => D::Sublation,
        R::Dependence => D::Contradiction,
        R::Similarity => D::NonIdentity,
        R::Association => D::Endoxa,
        R::RelationType | R::StructuralProperty => D::DialecticalArgument,

        // Structural properties
        R::Symmetric => D::Contrary,
        R::Antisymmetric => D::Contradictory,
        R::Transitive | R::Involutive => D::Sublation,
        R::Reflexive => D::Thesis,
        R::Irreflexive => D::DeterminateNegation,
        R::Functional => D::DialecticalArgument,
    }
}

/// Cross-functor: Relations → Dialectics.
///
/// The core ontological claim: the ten canonical binary relation types
/// (plus the seven structural properties of the Relations umbrella)
/// all find their richer philosophical home in Dialectics's
/// vocabulary. Opposition is the load-bearing bridge; the rest of the
/// mapping supplies context.
pub struct RelationsToDialectics;

impl Functor for RelationsToDialectics {
    type Source = RelationsCategory;
    type Target = DialecticsCategory;

    fn map_object(obj: &RelationsConcept) -> DialecticsConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &RelationsRelation) -> DialecticsRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            // Identity preserves: F(id_A) == id_{F(A)}.
            RelationsRelationKind::Identity => DialecticsCategory::identity(&from),
            // Every other kind maps to Composed in Dialectics — matching how
            // the target's Category::compose produces Composed morphisms for
            // non-Identity cases (so F(g∘f) == F(g)∘F(f) holds).
            _ => DialecticsRelation {
                from,
                to,
                kind: DialecticsRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(
    RelationsToDialectics,
    "Relations (Smith et al. 2005 OBO-RO; SKOS 2009; Tarski 1941) → Dialectics (Aristotle; Hegel; Priest). Opposition ↦ DialecticalMoment is the load-bearing bridge."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn relations_to_dialectics_laws_pass() {
        check_functor_laws::<RelationsToDialectics>().unwrap();
    }

    /// The headline claim: Opposition maps to DialecticalMoment.
    #[test]
    fn opposition_maps_to_dialectical_moment() {
        assert_eq!(
            RelationsToDialectics::map_object(&RelationsConcept::Opposition),
            DialecticsConcept::DialecticalMoment
        );
    }

    /// Equivalence maps to Synthesis — Hegel's higher unity.
    #[test]
    fn equivalence_maps_to_synthesis() {
        assert_eq!(
            RelationsToDialectics::map_object(&RelationsConcept::Equivalence),
            DialecticsConcept::Synthesis
        );
    }

    /// Structural property Symmetric maps to Contrary — both capture the
    /// Aristotelian symmetric-opposition structure.
    #[test]
    fn symmetric_maps_to_contrary() {
        assert_eq!(
            RelationsToDialectics::map_object(&RelationsConcept::Symmetric),
            DialecticsConcept::Contrary
        );
    }
}
