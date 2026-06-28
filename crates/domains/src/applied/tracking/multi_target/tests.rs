use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::tracking::multi_target::engine::ManagedTrack;
use crate::applied::tracking::multi_target::ontology::*;
use crate::applied::tracking::multi_target::track_management::MofNLogic;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn track_lifecycle_category_laws() {
    assert_category_laws::<MultiTargetCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn multi_target_ontology_validates() {
    MultiTargetOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn deleted_is_absorbing() {
    assert!(DeletedIsAbsorbing.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn track_starts_tentative() {
    assert!(TrackStartsTentative.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn re_detection_possible() {
    assert!(ReDetectionPossible.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn m_of_n_confirms_track() {
    // 3-of-5: need 3 hits in 5 scans
    let mut logic = MofNLogic::new(3, 5);
    logic.record_hit();
    logic.record_hit();
    assert!(!logic.is_confirmed());
    logic.record_hit();
    assert!(!logic.is_confirmed()); // only 3 entries, need 5
    logic.record_miss();
    logic.record_miss();
    assert!(logic.is_confirmed()); // 3 hits in 5 scans
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn m_of_n_deletes_insufficient() {
    let mut logic = MofNLogic::new(3, 5);
    logic.record_hit();
    logic.record_miss();
    logic.record_miss();
    logic.record_miss();
    logic.record_miss();
    assert!(logic.should_delete()); // only 1 hit in 5
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn managed_track_lifecycle() {
    // 2-of-3 confirmation, 3 max coast
    let mut track = ManagedTrack::new_tentative(1, 2, 3, 3);
    assert_eq!(track.state, MultiTargetConcept::Tentative);

    track.on_detection();
    track.on_detection();
    // 3 hits (initial + 2), should confirm after window fills
    assert_eq!(track.state, MultiTargetConcept::Confirmed);

    // Miss → coasting
    track.on_miss();
    assert_eq!(track.state, MultiTargetConcept::Coasting);

    // Re-detection → confirmed
    track.on_detection();
    assert_eq!(track.state, MultiTargetConcept::Confirmed);

    // 3 consecutive misses → deleted
    track.on_miss();
    track.on_miss();
    track.on_miss();
    // First miss → coasting, then 2 more → may delete
    // Actually: miss→coast, miss→coast(2), miss→delete
    assert_eq!(track.state, MultiTargetConcept::Deleted);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn deleted_track_stays_deleted() {
    let mut track = ManagedTrack::new_tentative(1, 2, 3, 1);
    // Force deletion
    track.state = MultiTargetConcept::Deleted;

    // Try to revive — should stay deleted
    track.on_detection();
    assert_eq!(track.state, MultiTargetConcept::Deleted);
    track.on_miss();
    assert_eq!(track.state, MultiTargetConcept::Deleted);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn deleted_never_revives(
            hits in proptest::collection::vec(proptest::bool::ANY, 1..20),
        ) {
            let mut track = ManagedTrack::new_tentative(1, 2, 3, 1);
            track.state = MultiTargetConcept::Deleted;

            for &hit in &hits {
                if hit {
                    track.on_detection();
                } else {
                    track.on_miss();
                }
                prop_assert_eq!(track.state, MultiTargetConcept::Deleted,
                    "deleted track must stay deleted");
            }
        }

        #[test]
        fn track_always_starts_tentative(m in 1..5_usize, n in 2..8_usize, max_coast in 1..10_usize) {
            let m = m.min(n); // m <= n
            let track = ManagedTrack::new_tentative(0, m, n, max_coast);
            prop_assert_eq!(track.state, MultiTargetConcept::Tentative);
        }
    }

    pr4xis::register_praxis_value!(deleted_never_revives, Verifiable);
    pr4xis::register_praxis_value!(track_always_starts_tentative, Verifiable);
}
