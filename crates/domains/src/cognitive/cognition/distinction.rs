//! Distinction — the most fundamental act of cognition.
//!
//! "Draw a distinction." — G. Spencer-Brown, Laws of Form (1969)
//!
//! Before categories, before logic, before knowledge — there is distinction.
//! The act of drawing a boundary that separates "this" from "not this."
//! Everything in praxis — Entity, Opposition, Boundary, Bit — is a distinction.
//!
//! References:
//! - Spencer-Brown, Laws of Form (1969)
//! - Kauffman, Laws of Form — An Exploration
//! - von Foerster, Observing Systems (1981) — distinction as basis of observation

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "Distinction",
    source: "Spencer-Brown (1969)",
    being: AbstractObject,

    concepts: [Void, Mark, Boundary, MarkedSpace, UnmarkedSpace, ReEntry],

    labels: {
        Void: ("en", "Void", "The unmarked state — the void before any distinction is made."),
        Mark: ("en", "Mark", "The act of indicating one side of a distinction."),
        Boundary: ("en", "Boundary", "The distinction itself, separating marked from unmarked."),
        MarkedSpace: ("en", "Marked space", "What is indicated (this side of the boundary)."),
        UnmarkedSpace: ("en", "Unmarked space", "What is not indicated (the other side)."),
        ReEntry: ("en", "Re-entry", "The distinction applied to itself (self-reference). Creates time and self-awareness (Spencer-Brown Ch. 11)."),
    },

    edges: [
        // Law of calling: Mark creates Boundary.
        (Mark, Boundary, Creates),
        // Law of crossing: Boundary separates spaces.
        (Boundary, MarkedSpace, Separates),
        (Boundary, UnmarkedSpace, Separates),
        // Mark indicates MarkedSpace.
        (Mark, MarkedSpace, Indicates),
        // Void → Mark (distinction emerges from void).
        (Void, Mark, Creates),
        // ReEntry: distinction applied to itself.
        (ReEntry, Mark, AppliesTo),
        (ReEntry, Boundary, AppliesTo),
    ],
}

/// Draw a distinction — the fundamental cognitive act.
/// Returns the two sides of the distinction.
pub fn draw_distinction<T: Clone + PartialEq + core::fmt::Debug>(marked: T, unmarked: T) -> (T, T) {
    assert_ne!(
        marked, unmarked,
        "a distinction requires two different things"
    );
    (marked, unmarked)
}
