use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::social::compliance::classification::{Confidence, EntityType};
use crate::social::military::situation::combat_identity::CombatIdentityConcept;
use crate::social::military::situation::engine::*;
use crate::social::military::situation::kinematic_relation::KinematicRelationConcept;
use crate::social::military::situation::ontology::*;

/// Build a planar tracked entity in a given reference frame (test helper).
fn entity(
    id: usize,
    entity_type: EntityType,
    identity: CombatIdentityConcept,
    frame: ReferenceFrame,
    position: [f64; 2],
    velocity: [f64; 2],
    confidence: Confidence,
) -> TrackedEntity {
    TrackedEntity {
        id,
        entity_type,
        identity,
        frame,
        position: Vector::new(vec![position[0], position[1]]),
        velocity: Vector::new(vec![velocity[0], velocity[1]]),
        confidence,
    }
}

/// A friendly aircraft tracked in the NED frame (the common test case).
fn ned_aircraft(id: usize, position: [f64; 2], velocity: [f64; 2]) -> TrackedEntity {
    entity(
        id,
        EntityType::Aircraft,
        CombatIdentityConcept::Friend,
        ReferenceFrame::NED,
        position,
        velocity,
        Confidence::High,
    )
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn situation_category_laws() {
    assert_category_laws::<SituationCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn situation_ontology_validates() {
    SituationOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn entity_identification_first_holds() {
    assert!(EntityIdentificationFirst.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn intent_requires_relationship_holds() {
    assert!(IntentRequiresRelationship.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn situation_assessment_construction() {
    let mut sa = SituationAssessment::new();
    sa.add_entity(ned_aircraft(1, [0.0, 0.0], [100.0, 0.0]));
    sa.add_entity(ned_aircraft(2, [50.0, 0.0], [100.0, 0.0]));
    assert_eq!(sa.num_entities(), 2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn formation_detection() {
    let a = ned_aircraft(1, [0.0, 0.0], [100.0, 0.0]);
    let b = ned_aircraft(2, [50.0, 0.0], [100.0, 0.0]);
    let rel = classify_relationship(&a, &b).expect("both in the NED frame → defined relationship");
    assert_eq!(rel.relation_type, KinematicRelationConcept::Formation);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn converging_entities() {
    let a = entity(
        1,
        EntityType::Watercraft,
        CombatIdentityConcept::Unknown,
        ReferenceFrame::NED,
        [0.0, 0.0],
        [5.0, 0.0],
        Confidence::High,
    );
    let b = entity(
        2,
        EntityType::Watercraft,
        CombatIdentityConcept::Unknown,
        ReferenceFrame::NED,
        [1000.0, 0.0],
        [-5.0, 0.0],
        Confidence::High,
    );
    let rel = classify_relationship(&a, &b).expect("both in the NED frame → defined relationship");
    assert_eq!(rel.relation_type, KinematicRelationConcept::Converging);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn different_frames_have_no_relationship() {
    // Relative kinematics are undefined across reference frames — the situation
    // engine refuses to relate an NED-framed track to an ECEF-framed one until a
    // frame transform aligns them (no fabricated relationship).
    let a = ned_aircraft(1, [0.0, 0.0], [100.0, 0.0]);
    let b = entity(
        2,
        EntityType::Aircraft,
        CombatIdentityConcept::Friend,
        ReferenceFrame::ECEF,
        [50.0, 0.0],
        [100.0, 0.0],
        Confidence::High,
    );
    assert!(classify_relationship(&a, &b).is_none());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn assess_relationships_populates() {
    let mut sa = SituationAssessment::new();
    for i in 0..3 {
        sa.add_entity(entity(
            i,
            EntityType::Unknown,
            CombatIdentityConcept::Unknown,
            ReferenceFrame::NED,
            [i as f64 * 100.0, 0.0],
            [0.0, 0.0],
            Confidence::Moderate,
        ));
    }
    sa.assess_relationships();
    // 3 entities in a common frame -> 3 pairs
    assert_eq!(sa.num_relationships(), 3);
    assert_eq!(sa.current_level, SituationConcept::Relationship);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn relationship_confidence_is_weakest_link(
            x1 in -1000.0..1000.0_f64,
            y1 in -1000.0..1000.0_f64,
            vx1 in -100.0..100.0_f64,
            vy1 in -100.0..100.0_f64,
            x2 in -1000.0..1000.0_f64,
            y2 in -1000.0..1000.0_f64,
            vx2 in -100.0..100.0_f64,
            vy2 in -100.0..100.0_f64
        ) {
            // Two same-frame tracks of differing confidence: the relationship
            // confidence is the weaker (min t-norm), regardless of geometry.
            let a = entity(0, EntityType::Unknown, CombatIdentityConcept::Unknown,
                ReferenceFrame::NED, [x1, y1], [vx1, vy1], Confidence::High);
            let b = entity(1, EntityType::Unknown, CombatIdentityConcept::Unknown,
                ReferenceFrame::NED, [x2, y2], [vx2, vy2], Confidence::Moderate);
            let rel = classify_relationship(&a, &b)
                .expect("both in the NED frame → defined relationship");
            prop_assert_eq!(rel.confidence, Confidence::Moderate);
        }

        #[test]
        fn relationship_count_is_n_choose_2(n in 2..8_usize) {
            let mut sa = SituationAssessment::new();
            for i in 0..n {
                sa.add_entity(entity(i, EntityType::Unknown, CombatIdentityConcept::Unknown,
                    ReferenceFrame::NED, [i as f64 * 100.0, 0.0], [0.0, 0.0], Confidence::Moderate));
            }
            sa.assess_relationships();
            // All entities share the NED frame, so every pair is defined.
            let expected = n * (n - 1) / 2;
            prop_assert_eq!(sa.num_relationships(), expected);
        }
    }

    pr4xis::register_praxis_value!(relationship_confidence_is_weakest_link, Verifiable);
    pr4xis::register_praxis_value!(relationship_count_is_n_choose_2, Verifiable);
}
