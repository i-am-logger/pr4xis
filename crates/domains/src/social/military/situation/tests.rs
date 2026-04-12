use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::social::military::situation::engine::*;
use crate::social::military::situation::ontology::*;

#[test]
fn situation_category_laws() {
    check_category_laws::<SituationCategory>().unwrap();
}

#[test]
fn situation_ontology_validates() {
    SituationOntology::validate().unwrap();
}

#[test]
fn entity_identification_first_holds() {
    assert!(EntityIdentificationFirst.holds());
}

#[test]
fn intent_requires_relationship_holds() {
    assert!(IntentRequiresRelationship.holds());
}

#[test]
fn situation_assessment_construction() {
    let mut sa = SituationAssessment::new();
    sa.add_entity(TrackedEntity {
        id: 1,
        classification: "aircraft",
        position: [0.0, 0.0],
        velocity: [100.0, 0.0],
        confidence: 0.9,
    });
    sa.add_entity(TrackedEntity {
        id: 2,
        classification: "aircraft",
        position: [50.0, 0.0],
        velocity: [100.0, 0.0],
        confidence: 0.85,
    });
    assert_eq!(sa.num_entities(), 2);
}

#[test]
fn formation_detection() {
    let a = TrackedEntity {
        id: 1,
        classification: "aircraft",
        position: [0.0, 0.0],
        velocity: [100.0, 0.0],
        confidence: 0.9,
    };
    let b = TrackedEntity {
        id: 2,
        classification: "aircraft",
        position: [50.0, 0.0],
        velocity: [100.0, 0.0],
        confidence: 0.9,
    };
    let rel = classify_relationship(&a, &b);
    assert_eq!(rel.relation_type, RelationType::Formation);
}

#[test]
fn converging_entities() {
    let a = TrackedEntity {
        id: 1,
        classification: "vessel",
        position: [0.0, 0.0],
        velocity: [5.0, 0.0],
        confidence: 0.9,
    };
    let b = TrackedEntity {
        id: 2,
        classification: "vessel",
        position: [1000.0, 0.0],
        velocity: [-5.0, 0.0],
        confidence: 0.9,
    };
    let rel = classify_relationship(&a, &b);
    assert_eq!(rel.relation_type, RelationType::Converging);
}

#[test]
fn assess_relationships_populates() {
    let mut sa = SituationAssessment::new();
    for i in 0..3 {
        sa.add_entity(TrackedEntity {
            id: i,
            classification: "unknown",
            position: [i as f64 * 100.0, 0.0],
            velocity: [0.0, 0.0],
            confidence: 0.5,
        });
    }
    sa.assess_relationships();
    // 3 entities -> 3 pairs
    assert_eq!(sa.num_relationships(), 3);
    assert_eq!(sa.current_level, SituationElement::Relationship);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn classification_confidence_bounded(
            x1 in -1000.0..1000.0_f64,
            y1 in -1000.0..1000.0_f64,
            vx1 in -100.0..100.0_f64,
            vy1 in -100.0..100.0_f64,
            x2 in -1000.0..1000.0_f64,
            y2 in -1000.0..1000.0_f64,
            vx2 in -100.0..100.0_f64,
            vy2 in -100.0..100.0_f64
        ) {
            let a = TrackedEntity {
                id: 0, classification: "a",
                position: [x1, y1], velocity: [vx1, vy1], confidence: 0.9,
            };
            let b = TrackedEntity {
                id: 1, classification: "b",
                position: [x2, y2], velocity: [vx2, vy2], confidence: 0.9,
            };
            let rel = classify_relationship(&a, &b);
            prop_assert!(rel.confidence >= 0.0 && rel.confidence <= 1.0,
                "confidence should be in [0,1], got {}", rel.confidence);
        }

        #[test]
        fn relationship_count_is_n_choose_2(n in 2..8_usize) {
            let mut sa = SituationAssessment::new();
            for i in 0..n {
                sa.add_entity(TrackedEntity {
                    id: i, classification: "x",
                    position: [i as f64 * 100.0, 0.0],
                    velocity: [0.0, 0.0], confidence: 0.5,
                });
            }
            sa.assess_relationships();
            let expected = n * (n - 1) / 2;
            prop_assert_eq!(sa.num_relationships(), expected);
        }
    }
}
