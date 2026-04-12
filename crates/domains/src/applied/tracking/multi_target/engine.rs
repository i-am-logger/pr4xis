use crate::applied::tracking::multi_target::ontology::TrackState;
use crate::applied::tracking::multi_target::track_management::{CoastingLogic, MofNLogic};

/// A managed track with lifecycle state.
#[derive(Debug, Clone)]
pub struct ManagedTrack {
    pub id: usize,
    pub state: TrackState,
    pub confirmation: MofNLogic,
    pub coasting: CoastingLogic,
}

impl ManagedTrack {
    pub fn new_tentative(id: usize, m: usize, n: usize, max_coast: usize) -> Self {
        let mut confirmation = MofNLogic::new(m, n);
        confirmation.record_hit(); // first detection
        Self {
            id,
            state: TrackState::Tentative,
            confirmation,
            coasting: CoastingLogic::new(max_coast),
        }
    }

    /// Process a detection hit.
    pub fn on_detection(&mut self) {
        self.confirmation.record_hit();
        self.coasting.record_hit();
        match self.state {
            TrackState::Tentative => {
                if self.confirmation.is_confirmed() {
                    self.state = TrackState::Confirmed;
                }
            }
            TrackState::Coasting => {
                self.state = TrackState::Confirmed;
            }
            _ => {}
        }
    }

    /// Process a missed detection.
    pub fn on_miss(&mut self) {
        self.confirmation.record_miss();
        self.coasting.record_miss();
        match self.state {
            TrackState::Tentative => {
                if self.confirmation.should_delete() {
                    self.state = TrackState::Deleted;
                }
            }
            TrackState::Confirmed => {
                self.state = TrackState::Coasting;
            }
            TrackState::Coasting => {
                if self.coasting.should_delete() {
                    self.state = TrackState::Deleted;
                }
            }
            TrackState::Deleted => {} // absorbing
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state != TrackState::Deleted
    }
}
