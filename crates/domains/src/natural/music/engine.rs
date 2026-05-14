#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::note::Note;
use super::scale::{Scale, ScaleKind};
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

/// Musical state: a current note in a scale context.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicState {
    pub note: Note,
    pub scale: Option<Scale>,
}

impl Situation for MusicState {}

#[derive(Debug, Clone, PartialEq)]
pub enum MusicAction {
    Transpose { semitones: i16 },
    SetScale { kind: ScaleKind },
    ClearScale,
    MoveTo { note: Note },
}

impl Action for MusicAction {
    type Sit = MusicState;
}

/// If a scale is set, transpositions must land on scale tones.
pub struct ScaleEnforcement;

impl Precondition<MusicAction> for ScaleEnforcement {
    fn check(&self, state: &MusicState, action: &MusicAction) -> Verdict {
        let meta = axiom_meta(
            "scale_enforcement",
            "notes must be in the current scale (if set)",
            "Lerdahl & Jackendoff (1983) A Generative Theory of Tonal Music §3; Schoenberg (1954) Structural Functions of Harmony",
        );
        let target_note = match action {
            MusicAction::Transpose { semitones } => match state.note.transpose(*semitones) {
                Some(n) => n,
                None => return Ok(Box::new(SimpleProof::new(meta))),
            },
            MusicAction::MoveTo { note } => *note,
            _ => return Ok(Box::new(SimpleProof::new(meta))),
        };

        match &state.scale {
            Some(scale) if !scale.contains(target_note) => {
                Err(Box::new(SimpleCounterexample::new(meta)))
            }
            _ => Ok(Box::new(SimpleProof::new(meta))),
        }
    }
}

/// MIDI range: notes must be 0-127.
pub struct RangeCheck;

impl Precondition<MusicAction> for RangeCheck {
    fn check(&self, state: &MusicState, action: &MusicAction) -> Verdict {
        let meta = axiom_meta(
            "range_check",
            "notes must be within MIDI range 0-127",
            "MIDI 1.0 Detailed Specification (1996) §A.4 Note Number Range",
        );
        match action {
            MusicAction::Transpose { semitones } => match state.note.transpose(*semitones) {
                Some(_) => Ok(Box::new(SimpleProof::new(meta))),
                None => Err(Box::new(SimpleCounterexample::new(meta))),
            },
            MusicAction::MoveTo { note } => {
                if note.0 <= 127 {
                    Ok(Box::new(SimpleProof::new(meta)))
                } else {
                    Err(Box::new(SimpleCounterexample::new(meta)))
                }
            }
            _ => Ok(Box::new(SimpleProof::new(meta))),
        }
    }
}

fn apply_music(
    state: &MusicState,
    action: &MusicAction,
) -> Result<MusicState, Box<dyn Counterexample>> {
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
