use crate::applied::tracking::multi_target::ontology::MultiTargetConcept;
use crate::applied::tracking::multi_target::track_management::{CoastingLogic, MofNLogic};

/// A managed track with lifecycle state.
#[derive(Debug, Clone)]
pub struct ManagedTrack {
    pub id: usize,
    pub state: MultiTargetConcept,
    pub confirmation: MofNLogic,
    pub coasting: CoastingLogic,
}

impl ManagedTrack {
    pub fn new_tentative(id: usize, m: usize, n: usize, max_coast: usize) -> Self {
        let mut confirmation = MofNLogic::new(m, n);
        confirmation.record_hit(); // first detection
        Self {
            id,
            state: MultiTargetConcept::Tentative,
            confirmation,
            coasting: CoastingLogic::new(max_coast),
        }
    }

    /// Process a detection hit.
    pub fn on_detection(&mut self) {
        self.confirmation.record_hit();
        self.coasting.record_hit();
        match self.state {
            MultiTargetConcept::Tentative if self.confirmation.is_confirmed() => {
                self.state = MultiTargetConcept::Confirmed;
            }
            MultiTargetConcept::Coasting => {
                self.state = MultiTargetConcept::Confirmed;
            }
            _ => {}
        }
    }

    /// Process a missed detection.
    pub fn on_miss(&mut self) {
        self.confirmation.record_miss();
        self.coasting.record_miss();
        match self.state {
            MultiTargetConcept::Tentative => {
                if self.confirmation.should_delete() {
                    self.state = MultiTargetConcept::Deleted;
                }
            }
            MultiTargetConcept::Confirmed => {
                self.state = MultiTargetConcept::Coasting;
            }
            MultiTargetConcept::Coasting => {
                if self.coasting.should_delete() {
                    self.state = MultiTargetConcept::Deleted;
                }
            }
            MultiTargetConcept::Deleted => {} // absorbing
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state != MultiTargetConcept::Deleted
    }
}
