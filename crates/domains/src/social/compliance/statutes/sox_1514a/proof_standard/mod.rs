//! SOX 1514A's contributing-factor causation standard — *statutory
//! text only*. Case-law confirmations live at their respective per-
//! case homes (`social::compliance::case_law::murray_v_ubs_2024`,
//! `social::compliance::case_law::lawson_v_fmr_2014`, …) and compose
//! with this ontology through the proof-framework synthesis layer,
//! not by being cited here.
//!
//! 18 U.S.C. § 1514A(b)(2)(C) cross-references the burden-shifting
//! framework of AIR21, codified at 49 U.S.C. § 42121(b)(2)(B). Under
//! that statute, a complainant proves causation by showing the
//! protected activity was a **contributing factor** in the adverse
//! action — a *lower* evidentiary bar than the classical reference-
//! layer tiers (Preponderance, Clear-and-Convincing, Beyond
//! Reasonable Doubt). The respondent may rebut under
//! § 42121(b)(2)(B)(iii) by demonstrating, by clear and convincing
//! evidence, that the same action would have been taken absent the
//! protected activity. This ontology models the *complainant's*
//! contributing-factor concept; the reference-layer
//! `ClearAndConvincing` covers the respondent's defense.
//!
//! # Modular-ontology placement (Stuckenschmidt et al. 2009)
//!
//! ```text
//!   [reference layer] social::judicial::proof_standard
//!     ProofStandard
//!       ├── Preponderance          tier 1
//!       ├── ClearAndConvincing     tier 2
//!       └── BeyondReasonableDoubt  tier 3
//!
//!   [application layer] social::compliance::statutes::sox_1514a::proof_standard
//!     SoxProofStandard
//!       └── ContributingFactor     tier 0   ← below the reference partition
//! ```
//!
//! # Why this lives at SOX's home, not in the reference layer
//!
//! `ContributingFactor` is statute-specific — it surfaces in AIR21,
//! SOX, FRSA, CFPA, NDAA and other federal whistleblower regimes
//! that incorporate AIR21's procedural framework. It is *not* a
//! general-jurisprudence concept; it would fail Guarino & Welty's
//! OntoClean filter (type vs role) if added as a reference-layer
//! leaf. Per the user's bottom-up rule, this ontology stays as close
//! to its primary source (AIR21 § 42121(b)(2)(B)) as possible and
//! cites only statutory text. Case-law refinements (no retaliatory
//! intent required; temporal proximity sufficient) live at their
//! respective case homes.
//!
//! # Statutory sources only
//!
//! - **18 U.S.C. § 1514A(b)(2)(C)** — SOX whistleblower civil-action
//!   procedure cross-reference into the AIR21 burden-shifting
//!   framework.
//! - **49 U.S.C. § 42121(b)(2)(B)(i)–(iv)** — the four-clause
//!   burden-shifting structure: complainant's prima facie
//!   contributing-factor showing (clauses (i)–(ii)) and respondent's
//!   clear-and-convincing same-action defense (clauses (iii)–(iv)).
//!
//! # Architectural sources
//!
//! - **Stuckenschmidt, Parent, Spaccapietra (eds.) (2009)** *Modular
//!   Ontologies*, Springer LNCS 5445 — three-tier
//!   foundational/reference/application architecture; rationale for
//!   placing statute-specific tiers at the application layer.
//! - **Guarino & Welty (2002)** "Evaluating Ontological Decisions
//!   with OntoClean", *CACM* 45(2):61–65 — type vs. role distinction;
//!   rationale for not contaminating the reference layer with
//!   statute-specific concepts.

pub mod ontology;

#[cfg(test)]
mod tests;
