#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::note::Note;
use super::scale::{Scale, ScaleKind};
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Musical state: a current note in a scale context.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicState {
    pub note: Note,
    pub scale: Option<Scale>,
}

impl Situation for MusicState {
    fn describe(&self) -> String {
        match &self.scale {
            Some(scale) => format!(
                "{} (in {:?} scale from {})",
                self.note.name(),
                scale.kind,
                scale.root.name()
            ),
            None => format!("{} (no scale context)", self.note.name()),
        }
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MusicAction {
    Transpose { semitones: i16 },
    SetScale { kind: ScaleKind },
    ClearScale,
    MoveTo { note: Note },
}

impl Action for MusicAction {
    type Sit = MusicState;

    fn describe(&self) -> String {
        match self {
            MusicAction::Transpose { semitones } => format!("transpose {} semitones", semitones),
            MusicAction::SetScale { kind } => format!("set scale {:?}", kind),
            MusicAction::ClearScale => "clear scale".into(),
            MusicAction::MoveTo { note } => format!("move to {}", note.name()),
        }
    }
}

/// If a scale is set, transpositions must land on scale tones.
pub struct ScaleEnforcement;

impl Precondition<MusicAction> for ScaleEnforcement {
    fn check(&self, state: &MusicState, action: &MusicAction) -> PreconditionResult {
        let target_note = match action {
            MusicAction::Transpose { semitones } => match state.note.transpose(*semitones) {
                Some(n) => n,
                None => {
                    return PreconditionResult::satisfied(
                        "scale_enforcement",
                        "out of range — deferred to range_check",
                    );
                }
            },
            MusicAction::MoveTo { note } => *note,
            _ => return PreconditionResult::satisfied("scale_enforcement", "no note change"),
        };

        match &state.scale {
            Some(scale) if !scale.contains(target_note) => PreconditionResult::violated(
                "scale_enforcement",
                &format!(
                    "{} is not in the {:?} scale",
                    target_note.name(),
                    scale.kind
                ),
                &state.describe(),
                &action.describe(),
            ),
            Some(scale) => PreconditionResult::satisfied(
                "scale_enforcement",
                &format!("{} is in the {:?} scale", target_note.name(), scale.kind),
            ),
            None => PreconditionResult::satisfied("scale_enforcement", "no scale context"),
        }
    }

    fn describe(&self) -> &str {
        "notes must be in the current scale (if set)"
    }
}

/// MIDI range: notes must be 0-127.
pub struct RangeCheck;

impl Precondition<MusicAction> for RangeCheck {
    fn check(&self, state: &MusicState, action: &MusicAction) -> PreconditionResult {
        match action {
            MusicAction::Transpose { semitones } => match state.note.transpose(*semitones) {
                Some(_) => PreconditionResult::satisfied("range_check", "in MIDI range"),
                None => PreconditionResult::violated(
                    "range_check",
                    &format!(
                        "note {} + {} semitones out of MIDI range 0-127",
                        state.note.0, semitones
                    ),
                    &state.describe(),
                    &action.describe(),
                ),
            },
            MusicAction::MoveTo { note } => {
                if note.0 <= 127 {
                    PreconditionResult::satisfied("range_check", "in MIDI range")
                } else {
                    PreconditionResult::violated(
                        "range_check",
                        &format!("note {} out of MIDI range 0-127", note.0),
                        &state.describe(),
                        &action.describe(),
                    )
                }
            }
            _ => PreconditionResult::satisfied("range_check", "no range concern"),
        }
    }

    fn describe(&self) -> &str {
        "notes must be within MIDI range 0-127"
    }
}

fn apply_music(state: &MusicState, action: &MusicAction) -> Result<MusicState, String> {
    let mut next = state.clone();
    match action {
        MusicAction::Transpose { semitones } => {
            if let Some(n) = state.note.transpose(*semitones) {
                next.note = n;
            }
        }
        MusicAction::SetScale { kind } => {
            next.scale = Some(Scale::new(state.note, *kind));
        }
        MusicAction::ClearScale => next.scale = None,
        MusicAction::MoveTo { note } => next.note = *note,
    }
    Ok(next)
}

pub type MusicEngine = Engine<MusicAction>;

pub fn new_music(root: Note) -> MusicEngine {
    Engine::new(
        MusicState {
            note: root,
            scale: None,
        },
        vec![Box::new(RangeCheck), Box::new(ScaleEnforcement)],
        apply_music,
    )
}
