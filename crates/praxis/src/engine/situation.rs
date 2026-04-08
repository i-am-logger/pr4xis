use std::fmt::Debug;

/// A situation is a snapshot of the world at a point in time.
///
/// Situations are immutable — every action produces a NEW situation.
/// The old situation is preserved for history/undo.
///
/// Implement this for your domain state (chess board, elevator positions,
/// case phase, etc.)
pub trait Situation: Clone + Debug + PartialEq {
    /// Human-readable description of this situation.
    fn describe(&self) -> String;

    /// Is this a terminal situation (no further actions possible)?
    fn is_terminal(&self) -> bool;
}
