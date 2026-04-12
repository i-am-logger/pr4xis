use super::decision::{Decision, MotionAction};
use super::entity::Entity;
use super::finding::Ruling;
use chrono::NaiveDate;

/// Case phase — rich state machine where each phase carries context.
#[derive(Debug, Clone, PartialEq)]
pub enum CasePhase {
    PreFiling,
    Filed {
        court: Entity,
        date: NaiveDate,
    },
    Discovery {
        began: NaiveDate,
    },
    Motions {
        active_count: usize,
    },
    PreTrial {
        trial_date: NaiveDate,
    },
    Trial {
        began: NaiveDate,
    },
    PostTrial {
        verdict: String,
        date: NaiveDate,
    },
    Appeal {
        court: Entity,
        filed: NaiveDate,
    },
    Closed {
        reason: CloseReason,
        date: NaiveDate,
    },
}

/// Why a case was closed.
#[derive(Debug, Clone, PartialEq)]
pub enum CloseReason {
    Settlement {
        terms: String,
    },
    Dismissal {
        reason: String,
        with_prejudice: bool,
    },
    Verdict {
        outcome: String,
    },
    Voluntary,
}

impl CasePhase {
    pub fn tag(&self) -> PhaseTag {
        match self {
            CasePhase::PreFiling => PhaseTag::PreFiling,
            CasePhase::Filed { .. } => PhaseTag::Filed,
            CasePhase::Discovery { .. } => PhaseTag::Discovery,
            CasePhase::Motions { .. } => PhaseTag::Motions,
            CasePhase::PreTrial { .. } => PhaseTag::PreTrial,
            CasePhase::Trial { .. } => PhaseTag::Trial,
            CasePhase::PostTrial { .. } => PhaseTag::PostTrial,
            CasePhase::Appeal { .. } => PhaseTag::Appeal,
            CasePhase::Closed { .. } => PhaseTag::Closed,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, CasePhase::Closed { .. })
    }
}

/// Lightweight phase tag for transition checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseTag {
    PreFiling,
    Filed,
    Discovery,
    Motions,
    PreTrial,
    Trial,
    PostTrial,
    Appeal,
    Closed,
}

impl PhaseTag {
    pub fn is_terminal(&self) -> bool {
        *self == PhaseTag::Closed
    }

    pub fn valid_transitions(&self) -> Vec<PhaseTag> {
        match self {
            PhaseTag::PreFiling => vec![PhaseTag::Filed],
            PhaseTag::Filed => vec![PhaseTag::Discovery, PhaseTag::Motions, PhaseTag::Closed],
            PhaseTag::Discovery => vec![PhaseTag::Motions, PhaseTag::PreTrial, PhaseTag::Closed],
            PhaseTag::Motions => vec![PhaseTag::Discovery, PhaseTag::PreTrial, PhaseTag::Closed],
            PhaseTag::PreTrial => vec![PhaseTag::Trial, PhaseTag::Closed],
            PhaseTag::Trial => vec![PhaseTag::PostTrial],
            PhaseTag::PostTrial => vec![PhaseTag::Appeal, PhaseTag::Closed],
            PhaseTag::Appeal => vec![PhaseTag::Motions, PhaseTag::Trial, PhaseTag::Closed],
            PhaseTag::Closed => vec![],
        }
    }
}

/// Actions that advance the case lifecycle.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum CaseAction {
    File {
        court: Entity,
        date: NaiveDate,
    },
    BeginDiscovery {
        date: NaiveDate,
    },
    FileMotion {
        motion: Decision,
        date: NaiveDate,
    },
    RuleOnMotion {
        motion_index: usize,
        action: MotionAction,
        date: NaiveDate,
    },
    SetForTrial {
        date: NaiveDate,
    },
    BeginTrial {
        date: NaiveDate,
    },
    Verdict {
        outcome: String,
        date: NaiveDate,
    },
    Appeal {
        court: Entity,
        date: NaiveDate,
    },
    Settle {
        terms: String,
        date: NaiveDate,
    },
    Dismiss {
        reason: String,
        with_prejudice: bool,
        date: NaiveDate,
    },
}

/// Result of a case action — NOT a bool.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    Ok { description: String },
    InvalidTransition { from: PhaseTag, action: String },
    MotionNotFound { index: usize },
    MotionError { message: String },
}

impl ActionResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, ActionResult::Ok { .. })
    }
}

/// An event in the case history.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseEvent {
    pub date: NaiveDate,
    pub phase_tag: PhaseTag,
    pub description: String,
}

/// A case: the full lifecycle container.
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub caption: String,
    pub phase: CasePhase,
    pub motions: Vec<Decision>,
    pub rulings: Vec<Ruling>,
    pub events: Vec<CaseEvent>,
}

impl Case {
    pub fn new(caption: &str) -> Self {
        Self {
            caption: caption.to_string(),
            phase: CasePhase::PreFiling,
            motions: Vec::new(),
            rulings: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Apply an action to the case.
    pub fn act(&mut self, action: CaseAction) -> ActionResult {
        match action {
            CaseAction::File { court, date } => {
                if self.phase.tag() != PhaseTag::PreFiling {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "file".into(),
                    };
                }
                self.phase = CasePhase::Filed { court, date };
                self.event(date, "Case filed")
            }

            CaseAction::BeginDiscovery { date } => {
                if !self.can_transition_to(PhaseTag::Discovery) {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "begin discovery".into(),
                    };
                }
                self.phase = CasePhase::Discovery { began: date };
                self.event(date, "Discovery begun")
            }

            CaseAction::FileMotion { motion, date } => {
                let tag = self.phase.tag();
                if tag != PhaseTag::Motions && tag != PhaseTag::Discovery && tag != PhaseTag::Filed
                {
                    return ActionResult::InvalidTransition {
                        from: tag,
                        action: "file motion".into(),
                    };
                }
                self.motions.push(motion);
                let count = self
                    .motions
                    .iter()
                    .filter(|m| !m.status.is_terminal())
                    .count();
                if tag != PhaseTag::Motions {
                    self.phase = CasePhase::Motions {
                        active_count: count,
                    };
                }
                self.event(date, "Motion filed")
            }

            CaseAction::RuleOnMotion {
                motion_index,
                action: motion_action,
                date,
            } => {
                if motion_index >= self.motions.len() {
                    return ActionResult::MotionNotFound {
                        index: motion_index,
                    };
                }
                match self.motions[motion_index].act(motion_action) {
                    Ok(advanced) => {
                        self.motions[motion_index] = advanced;
                        self.event(date, "Motion ruled")
                    }
                    Err(msg) => ActionResult::MotionError {
                        message: msg.to_string(),
                    },
                }
            }

            CaseAction::SetForTrial { date } => {
                if !self.can_transition_to(PhaseTag::PreTrial) {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "set for trial".into(),
                    };
                }
                self.phase = CasePhase::PreTrial { trial_date: date };
                self.event(date, &format!("Trial set for {}", date))
            }

            CaseAction::BeginTrial { date } => {
                if self.phase.tag() != PhaseTag::PreTrial {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "begin trial".into(),
                    };
                }
                self.phase = CasePhase::Trial { began: date };
                self.event(date, "Trial begun")
            }

            CaseAction::Verdict { outcome, date } => {
                if self.phase.tag() != PhaseTag::Trial {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "verdict".into(),
                    };
                }
                self.phase = CasePhase::PostTrial {
                    verdict: outcome.clone(),
                    date,
                };
                self.event(date, &format!("Verdict: {}", outcome))
            }

            CaseAction::Appeal { court, date } => {
                if self.phase.tag() != PhaseTag::PostTrial {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "appeal".into(),
                    };
                }
                self.phase = CasePhase::Appeal { court, filed: date };
                self.event(date, "Appeal filed")
            }

            CaseAction::Settle { terms, date } => {
                if self.phase.is_terminal() {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "settle".into(),
                    };
                }
                self.phase = CasePhase::Closed {
                    reason: CloseReason::Settlement {
                        terms: terms.clone(),
                    },
                    date,
                };
                self.event(date, &format!("Settled: {}", terms))
            }

            CaseAction::Dismiss {
                reason,
                with_prejudice,
                date,
            } => {
                if self.phase.is_terminal() {
                    return ActionResult::InvalidTransition {
                        from: self.phase.tag(),
                        action: "dismiss".into(),
                    };
                }
                self.phase = CasePhase::Closed {
                    reason: CloseReason::Dismissal {
                        reason: reason.clone(),
                        with_prejudice,
                    },
                    date,
                };
                self.event(date, &format!("Dismissed: {}", reason))
            }
        }
    }

    fn can_transition_to(&self, target: PhaseTag) -> bool {
        self.phase.tag().valid_transitions().contains(&target)
    }

    fn event(&mut self, date: NaiveDate, description: &str) -> ActionResult {
        self.events.push(CaseEvent {
            date,
            phase_tag: self.phase.tag(),
            description: description.to_string(),
        });
        ActionResult::Ok {
            description: description.to_string(),
        }
    }
}
