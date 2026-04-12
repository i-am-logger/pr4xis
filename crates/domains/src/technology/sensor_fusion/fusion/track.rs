use pr4xis::category::Entity;
use pr4xis::ontology::Quality;

/// Track status in the fusion engine context.
///
/// A track represents a persistent estimate of a target's state
/// maintained across multiple sensor updates. The status tracks
/// the lifecycle of this estimate.
///
/// Source: Bar-Shalom et al. (2001), Chapter 7 — "Track Maintenance."
///         Blackman & Popoli (1999), Chapter 4 — "Track Management."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FusionTrackStatus {
    /// Track is actively being updated by sensor measurements.
    /// The filter is in the predict-update cycle.
    Active,
    /// Track has not received measurements for some time.
    /// The filter is predicting forward without updates.
    /// Covariance is growing (uncertainty increasing).
    Coasting,
    /// Track has been declared lost — too long without measurements,
    /// or covariance has grown beyond the acceptable threshold.
    /// Should be deleted or archived.
    Lost,
}

impl Entity for FusionTrackStatus {
    fn variants() -> Vec<Self> {
        vec![Self::Active, Self::Coasting, Self::Lost]
    }
}

/// Quality: whether the track status allows state updates.
#[derive(Debug, Clone)]
pub struct AcceptsUpdates;

impl Quality for AcceptsUpdates {
    type Individual = FusionTrackStatus;
    type Value = bool;

    fn get(&self, status: &FusionTrackStatus) -> Option<bool> {
        Some(match status {
            FusionTrackStatus::Active => true,
            FusionTrackStatus::Coasting => true, // can still receive updates if re-acquired
            FusionTrackStatus::Lost => false,    // track is terminated
        })
    }
}

/// Quality: description of each track status.
#[derive(Debug, Clone)]
pub struct TrackStatusDescription;

impl Quality for TrackStatusDescription {
    type Individual = FusionTrackStatus;
    type Value = &'static str;

    fn get(&self, status: &FusionTrackStatus) -> Option<&'static str> {
        Some(match status {
            FusionTrackStatus::Active => "receiving measurements, filter is predict/update cycling",
            FusionTrackStatus::Coasting => {
                "no recent measurements, predicting forward with growing uncertainty"
            }
            FusionTrackStatus::Lost => "declared lost, covariance exceeded threshold or timed out",
        })
    }
}

/// Track lifecycle transitions.
///
/// Valid transitions:
/// - Active -> Coasting (measurement dropout)
/// - Coasting -> Active (re-acquisition)
/// - Coasting -> Lost (timeout exceeded)
/// - Active -> Lost (catastrophic failure / divergence)
///
/// Invalid transitions:
/// - Lost -> Active (must create a new track)
/// - Lost -> Coasting (lost is terminal)
pub fn is_valid_transition(from: FusionTrackStatus, to: FusionTrackStatus) -> bool {
    use FusionTrackStatus::*;
    matches!(
        (from, to),
        (Active, Active)
            | (Active, Coasting)
            | (Active, Lost)
            | (Coasting, Active)
            | (Coasting, Coasting)
            | (Coasting, Lost)
            | (Lost, Lost)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_statuses_have_descriptions() {
        let desc = TrackStatusDescription;
        for status in FusionTrackStatus::variants() {
            assert!(desc.get(&status).is_some());
        }
    }

    #[test]
    fn lost_does_not_accept_updates() {
        let q = AcceptsUpdates;
        assert_eq!(q.get(&FusionTrackStatus::Lost), Some(false));
    }

    #[test]
    fn active_accepts_updates() {
        let q = AcceptsUpdates;
        assert_eq!(q.get(&FusionTrackStatus::Active), Some(true));
    }

    #[test]
    fn valid_transitions() {
        use FusionTrackStatus::*;
        assert!(is_valid_transition(Active, Coasting));
        assert!(is_valid_transition(Coasting, Active));
        assert!(is_valid_transition(Coasting, Lost));
        assert!(is_valid_transition(Active, Lost));
    }

    #[test]
    fn invalid_transitions() {
        use FusionTrackStatus::*;
        assert!(!is_valid_transition(Lost, Active));
        assert!(!is_valid_transition(Lost, Coasting));
    }

    #[test]
    fn three_status_variants() {
        assert_eq!(FusionTrackStatus::variants().len(), 3);
    }
}
