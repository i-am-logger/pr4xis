use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::tracking::multi_target::engine::ManagedTrack;
use crate::technology::tracking::multi_target::ontology::*;
use crate::technology::tracking::multi_target::track_management::MofNLogic;

#[test]
fn track_lifecycle_category_laws() {
    check_category_laws::<TrackLifecycleCategory>().unwrap();
}

#[test]
fn multi_target_ontology_validates() {
    MultiTargetOntology::validate().unwrap();
}

#[test]
fn deleted_is_absorbing() {
    assert!(DeletedIsAbsorbing.holds());
}

#[test]
fn track_starts_tentative() {
    assert!(TrackStartsTentative.holds());
}

#[test]
fn re_detection_possible() {
    assert!(ReDetectionPossible.holds());
}

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

#[test]
fn managed_track_lifecycle() {
    // 2-of-3 confirmation, 3 max coast
    let mut track = ManagedTrack::new_tentative(1, 2, 3, 3);
    assert_eq!(track.state, TrackState::Tentative);

    track.on_detection();
    track.on_detection();
    // 3 hits (initial + 2), should confirm after window fills
    assert_eq!(track.state, TrackState::Confirmed);

    // Miss → coasting
    track.on_miss();
    assert_eq!(track.state, TrackState::Coasting);

    // Re-detection → confirmed
    track.on_detection();
    assert_eq!(track.state, TrackState::Confirmed);

    // 3 consecutive misses → deleted
    track.on_miss();
    track.on_miss();
    track.on_miss();
    // First miss → coasting, then 2 more → may delete
    // Actually: miss→coast, miss→coast(2), miss→delete
    assert_eq!(track.state, TrackState::Deleted);
}

#[test]
fn deleted_track_stays_deleted() {
    let mut track = ManagedTrack::new_tentative(1, 2, 3, 1);
    // Force deletion
    track.state = TrackState::Deleted;

    // Try to revive — should stay deleted
    track.on_detection();
    assert_eq!(track.state, TrackState::Deleted);
    track.on_miss();
    assert_eq!(track.state, TrackState::Deleted);
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
            track.state = TrackState::Deleted;

            for &hit in &hits {
                if hit {
                    track.on_detection();
                } else {
                    track.on_miss();
                }
                prop_assert_eq!(track.state, TrackState::Deleted,
                    "deleted track must stay deleted");
            }
        }

        #[test]
        fn track_always_starts_tentative(m in 1..5_usize, n in 2..8_usize, max_coast in 1..10_usize) {
            let m = m.min(n); // m <= n
            let track = ManagedTrack::new_tentative(0, m, n, max_coast);
            prop_assert_eq!(track.state, TrackState::Tentative);
        }
    }
}
