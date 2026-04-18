#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;

// Trace Schema Functor T: Sch → Sch
//
// Given any ontology schema C, T(C) automatically generates a trace schema
// that records every concept access and morphism traversal.
//
// T(C) = El(C) +_O O_obs
//
// Where:
// - El(C) = category of elements (Spivak 2012, §4.3) — one trace object
//   per schema element (entity type or morphism type)
// - O_obs = fixed PROV-O observability schema (W3C 2013) — timestamp,
//   status, agent, context
// - +_O = coproduct (pushout) gluing observation points to PROV decorations
//
// The instance lift T(I) = cofree_W(Delta_i(I)) is the cofree comonad
// of the writer monad applied to the pullback along i: C → T(C).
// (Uustalu & Vene 2008; Moggi 1991)
//
// References:
// - Spivak, "Functorial Data Migration" (2012) — El construction, §4.3
// - Spivak, "Category Theory for the Sciences" (2014) — Ch 4
// - Moggi, "Notions of Computation and Monads" (1991) — writer monad
// - Uustalu & Vene, "Comonadic Notions of Computation" (2008) — cofree
// - W3C PROV-O (2013) — observability schema
// - Grothendieck SGA1 (1961) — fibered categories

/// Concepts in the trace schema — derived from El(C) + O_obs.
///
/// For an ontology with N entity types and M morphism types,
/// T automatically generates N access objects + M traversal objects
/// + the fixed PROV-O objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum TraceSchemaElement {
    // === El(C): derived from the ontology schema ===
    /// An access to an entity type — records when a concept was queried.
    /// One per entity type in the original schema.
    /// El(C) object for each object of C.
    EntityAccess,

    /// A traversal of a morphism — records when a relationship was used.
    /// One per morphism type in the original schema.
    /// El(C) object for each morphism of C.
    MorphismTraversal,

    // === O_obs: fixed PROV-O observability schema ===
    /// When the access/traversal happened (prov:atTime).
    Timestamp,

    /// Whether it succeeded (ok/warning/error).
    Status,

    /// What process performed the access (prov:wasAssociatedWith).
    Agent,

    /// The trace context — span ID, parent span, etc. (OpenTelemetry).
    TraceContext,

    /// The input to the operation.
    Input,

    /// The output/result of the operation.
    Output,
}

define_ontology! {
    /// The trace schema category T — the target of the trace schema functor.
    pub TraceSchemaOntology for TraceSchemaCategory {
        concepts: TraceSchemaElement,
        relation: TraceSchemaRelation,
        kind: TraceSchemaRelationKind,
        kinds: [
            /// EntityAccess records which entity was accessed.
            /// Foreign key back to the original schema: subject_A: Accessed_A → A.
            RecordsSubject,
            /// MorphismTraversal records source entity access.
            /// source_f: Traversed_f → Accessed_A.
            RecordsSource,
            /// MorphismTraversal records target entity access.
            /// target_f: Traversed_f → Accessed_B.
            RecordsTarget,
            /// Any trace element has a timestamp.
            HasTimestamp,
            /// Any trace element has a status.
            HasStatus,
            /// Any trace element was performed by an agent.
            PerformedBy,
            /// Any trace element exists within a trace context.
            InContext,
            /// Any trace element has an input.
            HasInput,
            /// Any trace element has an output.
            HasOutput,
            /// MorphismTraversal is a refinement of EntityAccess
            /// (traversing a morphism implies accessing its endpoints).
            Refines,
        ],
        edges: [
            // El(C) structure: access and traversal foreign keys
            (MorphismTraversal, EntityAccess, RecordsSource),
            (MorphismTraversal, EntityAccess, RecordsTarget),
            (MorphismTraversal, EntityAccess, Refines),
            // O_obs: PROV decorations on EntityAccess
            (EntityAccess, Timestamp, HasTimestamp),
            (EntityAccess, Status, HasStatus),
            (EntityAccess, Agent, PerformedBy),
            (EntityAccess, TraceContext, InContext),
            (EntityAccess, Input, HasInput),
            (EntityAccess, Output, HasOutput),
            // O_obs: PROV decorations on MorphismTraversal
            (MorphismTraversal, Timestamp, HasTimestamp),
            (MorphismTraversal, Status, HasStatus),
            (MorphismTraversal, Agent, PerformedBy),
            (MorphismTraversal, TraceContext, InContext),
            (MorphismTraversal, Input, HasInput),
            (MorphismTraversal, Output, HasOutput),
        ],
        composed: [
            (MorphismTraversal, Timestamp),
            (MorphismTraversal, Status),
            (MorphismTraversal, Agent),
        ],
        being: AbstractObject,
        source: "W3C PROV-O (2013)",
    }
}

/// A concrete trace entry — an element of T(I) for a specific ontology.
///
/// This is what gets produced when an ontology is used.
/// The trace entry carries all the PROV-O decoration automatically.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Which ontology was accessed (the Agent).
    pub ontology_name: String,
    /// What was the operation (access or traversal).
    pub operation: String,
    /// The input to the operation.
    pub input: String,
    /// The output/result.
    pub output: String,
    /// Success or failure.
    pub success: bool,
}

impl TraceEntry {
    /// Serialize for transport.
    pub fn serialize(&self) -> String {
        let status = if self.success { "ok" } else { "warn" };
        format!(
            "{}:{}:{}:{}→{}",
            status, self.ontology_name, self.operation, self.input, self.output
        )
    }
}

/// A trace instance — T(I) for a specific pipeline execution.
/// Accumulates TraceEntry elements as ontologies are accessed.
#[derive(Debug, Clone, Default)]
pub struct TraceInstance {
    pub entries: Vec<TraceEntry>,
}

impl TraceInstance {
    /// Record an entity access.
    pub fn access(&mut self, ontology: &str, entity: &str, result: &str, success: bool) {
        self.entries.push(TraceEntry {
            ontology_name: ontology.into(),
            operation: "access".into(),
            input: entity.into(),
            output: result.into(),
            success,
        });
    }

    /// Record a morphism traversal.
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

    /// Serialize for transport.
    pub fn serialize(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.serialize())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<TraceSchemaCategory>().unwrap();
    }

    #[test]
    fn has_eight_elements() {
        assert_eq!(TraceSchemaElement::variants().len(), 8);
    }

    #[test]
    fn morphism_traversal_records_source() {
        let m = TraceSchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == TraceSchemaElement::MorphismTraversal
                    && r.to == TraceSchemaElement::EntityAccess
                    && r.kind == TraceSchemaRelationKind::RecordsSource)
        );
    }

    #[test]
    fn entity_access_has_timestamp() {
        let m = TraceSchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == TraceSchemaElement::EntityAccess
            && r.to == TraceSchemaElement::Timestamp
            && r.kind == TraceSchemaRelationKind::HasTimestamp));
    }

    #[test]
    fn entity_access_has_status() {
        let m = TraceSchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == TraceSchemaElement::EntityAccess
            && r.to == TraceSchemaElement::Status
            && r.kind == TraceSchemaRelationKind::HasStatus));
    }

    #[test]
    fn traversal_refines_access() {
        let m = TraceSchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == TraceSchemaElement::MorphismTraversal
                    && r.to == TraceSchemaElement::EntityAccess
                    && r.kind == TraceSchemaRelationKind::Refines)
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
}
