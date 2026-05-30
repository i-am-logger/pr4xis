//! Evidence requirement — the requirement level (Required / Recommended /
//! Optional) attached to a piece of evidence in a legal pleading.
//!
//! Mirrors RFC 2119 (BCP 14) requirement-level keywords exactly: the
//! correspondence is `Required ≡ MUST/SHALL`, `Recommended ≡ SHOULD`,
//! `Optional ≡ MAY`. The IETF requirement-level vocabulary is the
//! cleanest closed taxonomy of legal/specification requirement
//! semantics; McCormick on Evidence §337 provides the evidentiary-law
//! grounding for the same partition.
//!
//! # Concept partition
//!
//! ```text
//! RequirementLevel
//!   ├── Required     — MUST / SHALL — failure to provide defeats the pleading
//!   ├── Recommended  — SHOULD       — failure weakens but does not defeat
//!   └── Optional     — MAY          — provided when available, no consequence if absent
//! ```
//!
//! `Required` and `Optional` are *opposites* on a strict-need axis;
//! `Recommended` sits between them as a graded middle.
//!
//! # Literature
//!
//! - **RFC 2119 (Bradner 1997)** "Key words for use in RFCs to Indicate
//!   Requirement Levels", IETF — the canonical requirement-level
//!   vocabulary.
//! - **BCP 14 (Leiba 2017)** "Ambiguity of Uppercase vs Lowercase in
//!   RFC 2119 Key Words" — IETF Best Current Practice extending
//!   RFC 2119.
//! - **McCormick on Evidence (Strong et al., 8th ed. 2022)** §337
//!   (Burden of Production: Evidence Required) — legal-evidence
//!   grounding for the same partition.
//! - **Federal Rules of Evidence, Rule 104** — preliminary questions of
//!   evidentiary admissibility.

pub mod ontology;

#[cfg(test)]
mod tests;
