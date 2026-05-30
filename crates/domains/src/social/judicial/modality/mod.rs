//! Obligation modality — the deontic mode in which a legal rule binds.
//!
//! Statutes use a small closed set of modal markers to fix the deontic
//! force of each provision: "shall" / "must" → mandatory; "may" →
//! discretionary; "shall not" / "must not" / "may not" → prohibitive.
//! Praxis types this distinction so legal reasoning can act on it
//! (obligations become provable claims; prohibitions become defensive
//! shields; discretion permits but does not compel).
//!
//! # Concept partition
//!
//! ```text
//! ObligationModality
//!   ├── Mandatory       — obligation; O p (von Wright 1951)
//!   ├── Prohibitive     — duty to refrain; F p ≡ O ¬p
//!   └── Discretionary   — permission; P p ≡ ¬O ¬p
//! ```
//!
//! The triad is exhaustive of the **deontic primitives in classical
//! deontic logic** (von Wright 1951) — every modal marker in a statute
//! reduces to one of these via the standard dualities:
//!
//! - `Mandatory(p)` and `Prohibitive(¬p)` are interderivable (`O p ↔ F ¬p`).
//! - `Discretionary(p)` is the contradictory of `Prohibitive(p)`
//!   (`P p ↔ ¬F p`); the same provision cannot both permit and forbid
//!   the same act under the same conditions.
//! - `Mandatory(p)` *entails* `Discretionary(p)` (one does what one must),
//!   but not conversely; they are not opposites.
//!
//! These are the *kinds*; the actual modal *word* observed in a passage
//! (e.g. "shall", "may not") is a separate typed field on the
//! `Obligation` container — a `Phrase` reference into the statute
//! `Context`.
//!
//! # Literature
//!
//! - **von Wright (1951)** "Deontic Logic", *Mind* 60(237):1–15 —
//!   founding paper of the modal-deontic tradition. The O / P / F
//!   operators correspond directly to Mandatory / Discretionary /
//!   Prohibitive here.
//! - **Halliday (1985)** *An Introduction to Functional Grammar*,
//!   Edward Arnold, ch. 10 (Modality) — the systemic-functional
//!   linguistic grounding for the modal markers ("shall", "may", "must")
//!   actually used in statutes.
//! - **Sergot, Sadri, Kowalski, Kriwaczek, Hammond, Cory (1986)**
//!   "The British Nationality Act as a logic program", *Communications
//!   of the ACM* 29(5):370–386 — the landmark formalization of a legal
//!   statute into Horn-clause deontic logic; demonstrates the
//!   Mandatory/Prohibitive/Discretionary partition operating on a real
//!   national statute.
//! - **Hohfeld (1913)** "Some Fundamental Legal Conceptions as Applied
//!   in Judicial Reasoning", *Yale Law Journal* 23(1):16–59 — the
//!   right/duty/privilege/no-right correlatives; Hohfeld's "privilege"
//!   maps to Discretionary, "duty" to Mandatory, "disability" to
//!   Prohibitive.

pub mod ontology;

#[cfg(test)]
mod tests;
