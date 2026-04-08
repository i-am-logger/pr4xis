use super::argument::{Argument, CheckItem};
use super::entity::Entity;
use chrono::NaiveDate;

/// Types of motions — each carries its specific context.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionType {
    PreliminaryInjunction { irreparable_harm: String },
    Disqualification { conflict: String },
    MotionToDismiss { grounds: String },
    SummaryJudgment { undisputed_facts: Vec<String> },
    DiscoveryMotion { scope: String },
    ProtectiveOrder { protecting: String },
    MotionInLimine { exclude: String },
    MotionToCompel { compelling: String },
    MotionForSanctions { basis: String },
}

/// Motion status — rich state machine where each state carries HOW it got there.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum MotionStatus {
    Pending {
        filed: NaiveDate,
        movant: Entity,
    },
    Opposed {
        filed: NaiveDate,
        movant: Entity,
        opposition_date: NaiveDate,
        by: Entity,
    },
    UnderAdvisement {
        filed: NaiveDate,
        movant: Entity,
        since: NaiveDate,
    },
    Granted {
        ruling_date: NaiveDate,
        judge: Entity,
        order: String,
    },
    Denied {
        ruling_date: NaiveDate,
        judge: Entity,
        reason: String,
    },
    GrantedInPart {
        ruling_date: NaiveDate,
        judge: Entity,
        granted: String,
        denied: String,
    },
    Moot {
        reason: String,
        date: NaiveDate,
    },
    Withdrawn {
        date: NaiveDate,
        reason: Option<String>,
    },
}

impl MotionStatus {
    /// Is this a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            MotionStatus::Granted { .. }
                | MotionStatus::Denied { .. }
                | MotionStatus::GrantedInPart { .. }
                | MotionStatus::Moot { .. }
                | MotionStatus::Withdrawn { .. }
        )
    }

    /// Tag for matching (avoids pattern matching on inner data for transitions).
    pub fn tag(&self) -> StatusTag {
        match self {
            MotionStatus::Pending { .. } => StatusTag::Pending,
            MotionStatus::Opposed { .. } => StatusTag::Opposed,
            MotionStatus::UnderAdvisement { .. } => StatusTag::UnderAdvisement,
            MotionStatus::Granted { .. } => StatusTag::Granted,
            MotionStatus::Denied { .. } => StatusTag::Denied,
            MotionStatus::GrantedInPart { .. } => StatusTag::GrantedInPart,
            MotionStatus::Moot { .. } => StatusTag::Moot,
            MotionStatus::Withdrawn { .. } => StatusTag::Withdrawn,
        }
    }
}

/// Lightweight tag for status matching without inner data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusTag {
    Pending,
    Opposed,
    UnderAdvisement,
    Granted,
    Denied,
    GrantedInPart,
    Moot,
    Withdrawn,
}

impl StatusTag {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StatusTag::Granted
                | StatusTag::Denied
                | StatusTag::GrantedInPart
                | StatusTag::Moot
                | StatusTag::Withdrawn
        )
    }

    pub fn valid_transitions(&self) -> Vec<StatusTag> {
        match self {
            StatusTag::Pending => vec![StatusTag::Opposed, StatusTag::Withdrawn, StatusTag::Moot],
            StatusTag::Opposed => vec![
                StatusTag::UnderAdvisement,
                StatusTag::Withdrawn,
                StatusTag::Moot,
            ],
            StatusTag::UnderAdvisement => vec![
                StatusTag::Granted,
                StatusTag::Denied,
                StatusTag::GrantedInPart,
                StatusTag::Moot,
            ],
            _ => vec![],
        }
    }
}

/// Impact assessment — each level carries detail.
#[derive(Debug, Clone, PartialEq)]
pub enum Impact {
    Positive { detail: String },
    Minimal { detail: String },
    Warning { detail: String },
    Critical { detail: String },
}

/// Stakeholder outcome analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct Stakeholder {
    pub name: String,
    pub role: String,
    pub if_granted: Vec<Impact>,
    pub if_denied: Vec<Impact>,
}

/// Risk comparison: what happens if granted vs denied.
#[derive(Debug, Clone, PartialEq)]
pub struct RiskComparison {
    pub stakeholders: Vec<Stakeholder>,
    pub summary_if_granted: String,
    pub summary_if_denied: String,
}

/// Assessment of a decision.
#[derive(Debug, Clone, PartialEq)]
pub struct Assessment {
    pub summary: String,
    pub risk_comparison: Option<RiskComparison>,
    pub checklist_summary: Option<Vec<CheckItem>>,
}

/// A decision before the court.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub question: String,
    pub motion_type: MotionType,
    pub status: MotionStatus,
    pub arguments: Vec<Argument>,
    pub assessment: Assessment,
}

/// Actions that advance a motion.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionAction {
    Oppose {
        date: NaiveDate,
        by: Entity,
    },
    TakeUnderAdvisement {
        date: NaiveDate,
    },
    Grant {
        date: NaiveDate,
        judge: Entity,
        order: String,
    },
    Deny {
        date: NaiveDate,
        judge: Entity,
        reason: String,
    },
    GrantInPart {
        date: NaiveDate,
        judge: Entity,
        granted: String,
        denied: String,
    },
    DeclareMoot {
        date: NaiveDate,
        reason: String,
    },
    Withdraw {
        date: NaiveDate,
        reason: Option<String>,
    },
}

impl Decision {
    /// Apply a motion action. Returns Err if the transition is invalid.
    pub fn act(&self, action: MotionAction) -> Result<Decision, &'static str> {
        let current_tag = self.status.tag();
        let target_tag = match &action {
            MotionAction::Oppose { .. } => StatusTag::Opposed,
            MotionAction::TakeUnderAdvisement { .. } => StatusTag::UnderAdvisement,
            MotionAction::Grant { .. } => StatusTag::Granted,
            MotionAction::Deny { .. } => StatusTag::Denied,
            MotionAction::GrantInPart { .. } => StatusTag::GrantedInPart,
            MotionAction::DeclareMoot { .. } => StatusTag::Moot,
            MotionAction::Withdraw { .. } => StatusTag::Withdrawn,
        };

        if !current_tag.valid_transitions().contains(&target_tag) {
            return Err("invalid motion status transition");
        }

        let new_status = match action {
            MotionAction::Oppose { date, by } => {
                let (filed, movant) = extract_filed_movant(&self.status)?;
                MotionStatus::Opposed {
                    filed,
                    movant,
                    opposition_date: date,
                    by,
                }
            }
            MotionAction::TakeUnderAdvisement { date } => {
                let (filed, movant) = extract_filed_movant(&self.status)?;
                MotionStatus::UnderAdvisement {
                    filed,
                    movant,
                    since: date,
                }
            }
            MotionAction::Grant { date, judge, order } => MotionStatus::Granted {
                ruling_date: date,
                judge,
                order,
            },
            MotionAction::Deny {
                date,
                judge,
                reason,
            } => MotionStatus::Denied {
                ruling_date: date,
                judge,
                reason,
            },
            MotionAction::GrantInPart {
                date,
                judge,
                granted,
                denied,
            } => MotionStatus::GrantedInPart {
                ruling_date: date,
                judge,
                granted,
                denied,
            },
            MotionAction::DeclareMoot { date, reason } => MotionStatus::Moot { reason, date },
            MotionAction::Withdraw { date, reason } => MotionStatus::Withdrawn { date, reason },
        };

        let mut next = self.clone();
        next.status = new_status;
        Ok(next)
    }
}

fn extract_filed_movant(status: &MotionStatus) -> Result<(NaiveDate, Entity), &'static str> {
    match status {
        MotionStatus::Pending { filed, movant } => Ok((*filed, movant.clone())),
        MotionStatus::Opposed { filed, movant, .. } => Ok((*filed, movant.clone())),
        MotionStatus::UnderAdvisement { filed, movant, .. } => Ok((*filed, movant.clone())),
        _ => Err("cannot extract filed/movant from terminal status"),
    }
}
