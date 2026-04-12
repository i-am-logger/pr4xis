use pr4xis::category::Functor;

use super::ontology::*;
use super::trace_schema::*;

// Schema → TraceSchema functor T.
//
// Proof that every ontology schema IS automatically traceable (Spivak 2012 + PROV-O 2013).
//
// The El(C) construction (Spivak 2012, §4.3) produces one trace element
// per schema element. Entity types become access points; morphism types
// become traversals. The PROV-O decorations (timestamp, status, agent)
// attach automatically to every trace element.
//
// The mapping:
//   Schema          → EntityAccess (the schema is an entity you query)
//   EntityType      → EntityAccess (querying an entity type)
//   MorphismType    → MorphismTraversal (following a relationship)
//   PathEquation    → Input (constraints on traversal)
//   Axiom           → Status (axiom determines validity)
//   Instance        → Output (instance is computed result)
//   Population      → Output (population of an entity type)
//   SchemaMapping   → MorphismTraversal (mapping traverses between schemas)
//   Transform       → MorphismTraversal (transform traverses between instances)
//   Presentation    → Input (syntactic form — what goes in)
//   Algebra         → Output (semantic form — what comes out)
//
// References:
// - Spivak, "Functorial Data Migration" (2012) — El construction
// - Spivak, "Category Theory for the Sciences" (2014) — Ch 4
// - W3C PROV-O (2013) — observability schema
// - Moggi, "Notions of Computation and Monads" (1991) — writer monad

pub struct SchemaToTraceSchema;

impl Functor for SchemaToTraceSchema {
    type Source = SchemaCategory;
    type Target = TraceSchemaCategory;

    fn map_object(obj: &SchemaConcept) -> TraceSchemaElement {
        match obj {
            SchemaConcept::Schema => TraceSchemaElement::EntityAccess,
            SchemaConcept::EntityType => TraceSchemaElement::EntityAccess,
            SchemaConcept::MorphismType => TraceSchemaElement::MorphismTraversal,
            SchemaConcept::PathEquation => TraceSchemaElement::Input,
            SchemaConcept::Axiom => TraceSchemaElement::Status,
            SchemaConcept::Instance => TraceSchemaElement::Output,
            SchemaConcept::Population => TraceSchemaElement::Output,
            SchemaConcept::SchemaMapping => TraceSchemaElement::MorphismTraversal,
            SchemaConcept::Transform => TraceSchemaElement::MorphismTraversal,
            SchemaConcept::Presentation => TraceSchemaElement::Input,
            SchemaConcept::Algebra => TraceSchemaElement::Output,
        }
    }

    fn map_morphism(m: &SchemaRelation) -> TraceSchemaRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            SchemaRelationKind::Identity => TraceSchemaRelationKind::Identity,
            SchemaRelationKind::ContainsEntity => TraceSchemaRelationKind::RecordsSubject,
            SchemaRelationKind::ContainsMorphism => TraceSchemaRelationKind::RecordsSource,
            SchemaRelationKind::ContainsEquation => TraceSchemaRelationKind::HasInput,
            SchemaRelationKind::ContainsAxiom => TraceSchemaRelationKind::HasStatus,
            SchemaRelationKind::Participates => TraceSchemaRelationKind::RecordsSource,
            SchemaRelationKind::InstantiatedFrom => TraceSchemaRelationKind::HasOutput,
            SchemaRelationKind::Assigns => TraceSchemaRelationKind::HasOutput,
            SchemaRelationKind::Maps => TraceSchemaRelationKind::RecordsTarget,
            SchemaRelationKind::Transforms => TraceSchemaRelationKind::Refines,
            SchemaRelationKind::Evaluates => TraceSchemaRelationKind::HasOutput,
            SchemaRelationKind::Presents => TraceSchemaRelationKind::HasInput,
            SchemaRelationKind::Composed => TraceSchemaRelationKind::Composed,
        };
        TraceSchemaRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<SchemaToTraceSchema>().unwrap();
    }

    #[test]
    fn entity_type_maps_to_entity_access() {
        assert_eq!(
            SchemaToTraceSchema::map_object(&SchemaConcept::EntityType),
            TraceSchemaElement::EntityAccess
        );
    }

    #[test]
    fn morphism_type_maps_to_traversal() {
        assert_eq!(
            SchemaToTraceSchema::map_object(&SchemaConcept::MorphismType),
            TraceSchemaElement::MorphismTraversal
        );
    }

    #[test]
    fn axiom_maps_to_status() {
        assert_eq!(
            SchemaToTraceSchema::map_object(&SchemaConcept::Axiom),
            TraceSchemaElement::Status
        );
    }

    #[test]
    fn instance_maps_to_output() {
        assert_eq!(
            SchemaToTraceSchema::map_object(&SchemaConcept::Instance),
            TraceSchemaElement::Output
        );
    }

    #[test]
    fn presentation_maps_to_input() {
        assert_eq!(
            SchemaToTraceSchema::map_object(&SchemaConcept::Presentation),
            TraceSchemaElement::Input
        );
    }
}
