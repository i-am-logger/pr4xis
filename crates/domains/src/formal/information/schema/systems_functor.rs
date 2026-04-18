use pr4xis::category::Functor;

use crate::formal::information::schema::ontology::*;
use crate::formal::systems::ontology::*;

// SystemsToSchema functor — proves every ontology IS a system.
//
// This is the key meta-level insight: ontology structure (Schema) maps
// directly to systems thinking concepts. An ontology has components (entities),
// interactions (morphisms), constraints (axioms), feedback (validation),
// emergence (composed behavior), and homeostasis (consistency).
//
// The mapping:
//   Component   → EntityType (a system component IS an entity type)
//   Interaction → MorphismType (interactions ARE typed relationships)
//   State       → Instance (system state IS an instance)
//   Transition  → Transform (transitions ARE natural transformations)
//   Constraint  → Axiom (system constraints ARE axioms)
//   Feedback    → PathEquation (feedback loops ARE compositional constraints)
//   Homeostasis → Schema (maintaining structure IS the schema itself)
//   Emergence   → Algebra (emergent properties ARE the evaluated algebra)
//   Boundary    → Presentation (system boundary IS the syntactic form)
//   Controller  → SchemaMapping (the controller maps between schemas)
//
// References:
// - von Bertalanffy, "General System Theory" (1968) — systems as wholes
// - Spivak, "Functorial Data Migration" (2012) — schema as category
// - Meadows, "Thinking in Systems" (2008) — feedback, emergence, boundaries

pub struct SystemsToSchema;

impl Functor for SystemsToSchema {
    type Source = SystemsCategory;
    type Target = SchemaCategory;

    fn map_object(obj: &SystemConcept) -> SchemaConcept {
        match obj {
            SystemConcept::Component => SchemaConcept::EntityType,
            SystemConcept::Interaction => SchemaConcept::MorphismType,
            SystemConcept::State => SchemaConcept::Instance,
            SystemConcept::Transition => SchemaConcept::Transform,
            SystemConcept::Constraint => SchemaConcept::Axiom,
            SystemConcept::Feedback => SchemaConcept::PathEquation,
            SystemConcept::Homeostasis => SchemaConcept::Schema,
            SystemConcept::Emergence => SchemaConcept::Algebra,
            SystemConcept::Boundary => SchemaConcept::Presentation,
            SystemConcept::Controller => SchemaConcept::SchemaMapping,
        }
    }

    fn map_morphism(m: &SystemRelation) -> SchemaRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            SystemRelationKind::Identity => SchemaRelationKind::Identity,
            SystemRelationKind::ComposesInto => SchemaRelationKind::Participates,
            SystemRelationKind::Changes => SchemaRelationKind::Transforms,
            SystemRelationKind::Governs => SchemaRelationKind::ContainsAxiom,
            SystemRelationKind::FeedsBack => SchemaRelationKind::ContainsEquation,
            SystemRelationKind::Stabilizes => SchemaRelationKind::ContainsEquation,
            SystemRelationKind::ArisesFrom => SchemaRelationKind::Evaluates,
            SystemRelationKind::Regulates => SchemaRelationKind::Maps,
            SystemRelationKind::Separates => SchemaRelationKind::Presents,
            SystemRelationKind::Composed => SchemaRelationKind::Composed,
        };
        SchemaRelation { from, to, kind }
    }
}
pr4xis::register_functor!(SystemsToSchema);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Functor;
    use pr4xis::category::entity::Entity;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<SystemsToSchema>().unwrap();
    }

    #[test]
    fn all_system_concepts_map() {
        for concept in SystemConcept::variants() {
            let _ = SystemsToSchema::map_object(&concept);
        }
    }

    #[test]
    fn mapping_is_total() {
        assert_eq!(SystemConcept::variants().len(), 10);
        let mapped: std::collections::HashSet<SchemaConcept> = SystemConcept::variants()
            .into_iter()
            .map(|c| SystemsToSchema::map_object(&c))
            .collect();
        assert!(
            mapped.len() >= 8,
            "functor should map to at least 8 distinct schema concepts, got {}",
            mapped.len()
        );
    }

    #[test]
    fn component_maps_to_entity_type() {
        assert_eq!(
            SystemsToSchema::map_object(&SystemConcept::Component),
            SchemaConcept::EntityType
        );
    }

    #[test]
    fn interaction_maps_to_morphism_type() {
        assert_eq!(
            SystemsToSchema::map_object(&SystemConcept::Interaction),
            SchemaConcept::MorphismType
        );
    }

    #[test]
    fn state_maps_to_instance() {
        assert_eq!(
            SystemsToSchema::map_object(&SystemConcept::State),
            SchemaConcept::Instance
        );
    }

    #[test]
    fn constraint_maps_to_axiom() {
        assert_eq!(
            SystemsToSchema::map_object(&SystemConcept::Constraint),
            SchemaConcept::Axiom
        );
    }

    #[test]
    fn emergence_maps_to_algebra() {
        assert_eq!(
            SystemsToSchema::map_object(&SystemConcept::Emergence),
            SchemaConcept::Algebra
        );
    }
}
