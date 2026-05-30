//! Valence — the argumentative role a legal term plays in a claim or defence.
//!
//! Hart (1961) introduced the primary/secondary distinction; MacCormick
//! (1978) refined it for *argumentative role* — whether a rule operates
//! as a support for a claim, a defeating consideration, or a procedural
//! scope-setter. Wigmore (1937) §27 gives the rhetorical grounding for
//! the supporting / defeating partition in trial advocacy.
//!
//! # Concept partition
//!
//! ```text
//! Valence
//!   ├── Supportive  — pro-claimant (advances the moving party's case)
//!   ├── Defensive   — pro-respondent (defeats the moving party's case)
//!   └── Procedural  — scope, jurisdiction, definitions, non-merits gating
//! ```
//!
//! The triad is **complete and pairwise distinct**: every term in a
//! statutory ontology occupies exactly one valence role with respect to
//! a posited claim. Supportive and Defensive *oppose* each other (the
//! same statutory provision cannot both advance and defeat the same
//! claim; if a provision can play either role it is itself procedural
//! and admits both roles by *scope*, not by valence). Procedural is
//! the orthogonal axis — neither pro nor con on the merits.
//!
//! # Literature
//!
//! - **Hart (1961)** *The Concept of Law*, Oxford University Press,
//!   ch. V — primary rules of obligation vs. secondary rules of
//!   recognition; the primary distinction underlying argumentative
//!   role classification.
//! - **MacCormick (1978)** *Legal Reasoning and Legal Theory*, Oxford
//!   University Press, ch. 3 — "Universalisability and Justification"
//!   for the supporting/defeating refinement; ch. 5 §3 on procedural
//!   vs substantive distinction.
//! - **Wigmore (1937)** *A Students' Textbook of the Law of Evidence*,
//!   Foundation Press, §27 (Argumentative valence in trial advocacy)
//!   — empirical grounding for the supportive/defensive partition.

pub mod ontology;

#[cfg(test)]
mod tests;
