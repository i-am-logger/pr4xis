//! Trace Schema Functor T: Sch → Sch — given any ontology schema C,
//! T(C) automatically generates a trace schema that records every
//! concept access and morphism traversal.
//!
//! `T(C) = El(C) +_O O_obs`
//!
//! Where `El(C)` is the category of elements (Spivak 2012 §4.3) — one
//! trace object per schema element — and `O_obs` is the fixed PROV-O
//! observability schema (W3C 2013). The instance lift `T(I) =
//! cofree_W(Δ_i(I))` is the cofree comonad of the writer monad applied
//! to the pullback along `i: C → T(C)` (Uustalu & Vene 2008;
//! Moggi 1991).
//!
//! # Literature
//!
//! - **Spivak (2012)** "Functorial Data Migration", *Information and
//!   Computation* 217:31-51 — El construction §4.3.
//! - **Spivak (2014)** *Category Theory for the Sciences*, MIT Press
//!   Ch. 4 — elements of a functor.
//! - **Moggi (1991)** "Notions of Computation and Monads",
//!   *Information and Computation* 93(1):55-92 — writer monad.
//! - **Uustalu & Vene (2008)** "Comonadic Notions of Computation",
//!   *Electronic Notes in Theoretical Computer Science* 203(5):263-284
//!   — cofree comonad construction.
//! - **W3C PROV-O (2013)** *PROV-O: The PROV Ontology* — observability
//!   schema (Activity / Agent / atTime).
//! - **Grothendieck (1961)** SGA 1 — fibered categories.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "TraceSchema",
    source: "Spivak (2012) Functorial Data Migration §4.3, Information and Computation 217:31-51; Spivak (2014) Category Theory for the Sciences Ch. 4, MIT Press; Moggi (1991) Notions of Computation and Monads, Information and Computation 93(1):55-92; Uustalu & Vene (2008) Comonadic Notions of Computation, ENTCS 203(5):263-284; W3C PROV-O (2013); Grothendieck (1961) SGA 1",

    concepts: [
        // El(C) - derived from the source ontology schema.
        EntityAccess,
        MorphismTraversal,
        // O_obs - fixed PROV-O observability schema.
        Timestamp,
        Status,
        Agent,
        TraceContext,
        Input,
        Output,
    ],

    labels: {
        EntityAccess: ("en", "Entity access",
            "Spivak (2012) §4.3 El(C) object: an access to an entity type - records when a concept was queried. One per object of C."),
        MorphismTraversal: ("en", "Morphism traversal",
            "Spivak (2012) §4.3 El(C) object: a traversal of a morphism - records when a relationship was used. One per morphism of C."),
        Timestamp: ("en", "Timestamp",
            "W3C PROV-O (2013) prov:atTime - when the access or traversal happened."),
        Status: ("en", "Status",
            "Outcome of the access/traversal - ok / warning / error."),
        Agent: ("en", "Agent",
            "W3C PROV-O (2013) prov:wasAssociatedWith - what process performed the access."),
        TraceContext: ("en", "Trace context",
            "OpenTelemetry SpanContext: span id, parent span, baggage."),
        Input: ("en", "Input",
            "The input to the operation."),
        Output: ("en", "Output",
            "The output / result of the operation."),
    },

    edges: [
        // El(C) structure: foreign keys back into the schema.
        (MorphismTraversal, EntityAccess, RecordsSource),
        (MorphismTraversal, EntityAccess, RecordsTarget),
        (MorphismTraversal, EntityAccess, Refines),
        // PROV-O decorations on EntityAccess.
        (EntityAccess, Timestamp, HasTimestamp),
        (EntityAccess, Status, HasStatus),
        (EntityAccess, Agent, PerformedBy),
        (EntityAccess, TraceContext, InContext),
        (EntityAccess, Input, HasInput),
        (EntityAccess, Output, HasOutput),
        // PROV-O decorations on MorphismTraversal.
        (MorphismTraversal, Timestamp, HasTimestamp),
        (MorphismTraversal, Status, HasStatus),
        (MorphismTraversal, Agent, PerformedBy),
        (MorphismTraversal, TraceContext, InContext),
        (MorphismTraversal, Input, HasInput),
        (MorphismTraversal, Output, HasOutput),
    ],
}

/// A concrete trace entry — an element of T(I) for a specific ontology.
/// Carries PROV-O decoration automatically.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub ontology_name: String,
    pub operation: String,
    pub input: String,
    pub output: String,
    pub success: bool,
}

impl TraceEntry {
    pub fn serialize(&self) -> String {
        let status = if self.success { "ok" } else { "warn" };
        format!(
            "{}:{}:{}:{}→{}",
            status, self.ontology_name, self.operation, self.input, self.output
        )
    }
}

/// A trace instance — T(I) for a specific pipeline execution.
#[derive(Debug, Clone, Default)]
pub struct TraceInstance {
    pub entries: Vec<TraceEntry>,
}

impl TraceInstance {
    pub fn access(&mut self, ontology: &str, entity: &str, result: &str, success: bool) {
        self.entries.push(TraceEntry {
            ontology_name: ontology.into(),
            operation: "access".into(),
            input: entity.into(),
            output: result.into(),
            success,
        });
    }

    pub fn traverse(
        &mut self,
        ontology: &str,
        morphism: &str,
        input: &str,
        output: &str,
        success: bool,
    ) {
        self.entries.push(TraceEntry {
            ontology_name: ontology.into(),
            operation: morphism.into(),
            input: input.into(),
            output: output.into(),
            success,
        });
    }

    pub fn serialize(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.serialize())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// Legacy alias — `TraceSchemaElement` was the hand-written enum's name.
pub type TraceSchemaElement = TraceSchemaConcept;

/// Quality: whether a concept is a PROV-O decoration vs an El(C)
/// foreign-key element. W3C PROV-O (2013) distinguishes provenance
/// metadata from the schema-element layer.
#[derive(Debug, Clone)]
pub struct IsProvDecoration;

impl Quality for IsProvDecoration {
    type Individual = TraceSchemaConcept;
    type Value = bool;

    fn get(&self, c: &TraceSchemaConcept) -> Option<bool> {
        use TraceSchemaConcept as T;
        Some(matches!(
            c,
            T::Timestamp | T::Status | T::Agent | T::TraceContext | T::Input | T::Output
        ))
    }
}

impl Ontology for TraceSchemaOntology {
    type Cat = TraceSchemaCategory;
    type Qual = IsProvDecoration;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TraceSchemaCategory>();
    }

    #[test]
    fn ontology_validates() {
        TraceSchemaOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn eight_elements() {
        assert_eq!(TraceSchemaConcept::variants().len(), 8);
    }

    #[test]
    fn morphism_traversal_records_source() {
        let m = TraceSchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == TraceSchemaConcept::MorphismTraversal
                    && r.target() == TraceSchemaConcept::EntityAccess
                    && r.kind() == TraceSchemaRelationKind::RecordsSource)
        );
    }

    #[test]
    fn entity_access_has_timestamp() {
        let m = TraceSchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == TraceSchemaConcept::EntityAccess
                    && r.target() == TraceSchemaConcept::Timestamp
                    && r.kind() == TraceSchemaRelationKind::HasTimestamp)
        );
    }

    #[test]
    fn traversal_refines_access() {
        let m = TraceSchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == TraceSchemaConcept::MorphismTraversal
                    && r.target() == TraceSchemaConcept::EntityAccess
                    && r.kind() == TraceSchemaRelationKind::Refines)
        );
    }

    #[test]
    fn trace_instance_accumulates() {
        let mut ti = TraceInstance::default();
        ti.access("WordNet", "dog", "found 8 senses", true);
        ti.traverse("WordNet Taxonomy", "is_a", "dog", "mammal → true", true);
        assert_eq!(ti.entries.len(), 2);
        assert_eq!(ti.entries[0].ontology_name, "WordNet");
        assert_eq!(ti.entries[1].operation, "is_a");
    }

    #[test]
    fn serialize_format() {
        let mut ti = TraceInstance::default();
        ti.access("WordNet", "dog", "8 senses", true);
        let s = ti.serialize();
        assert!(s.contains("ok:WordNet:access:dog→8 senses"));
    }

    fn arb_concept() -> impl Strategy<Value = TraceSchemaConcept> {
        proptest::sample::select(TraceSchemaConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in TraceSchemaCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TraceSchemaOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_prov_decoration_total(c in arb_concept()) {
            prop_assert!(IsProvDecoration.get(&c).is_some());
        }
    }
}
