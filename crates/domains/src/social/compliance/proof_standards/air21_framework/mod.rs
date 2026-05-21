//! The AIR21 burden-shifting proof-standard framework — *statutory
//! text only*. Case-law confirmations live at their respective per-
//! case homes (`social::compliance::case_law::murray_v_ubs_2024`,
//! `social::compliance::case_law::lawson_v_fmr_2014`, …) and compose
//! with this ontology through the proof-framework synthesis layer,
//! not by being cited here.
//!
//! 49 U.S.C. § 42121(b)(2)(B)(i)–(iv) codifies the four-clause
//! burden-shifting structure that a complainant must traverse to
//! recover for whistleblower retaliation. Under clauses (i)–(ii), the
//! complainant proves causation by showing the protected activity was
//! a **contributing factor** in the adverse action — a *lower*
//! evidentiary bar than the classical reference-layer tiers
//! (Preponderance, Clear-and-Convincing, Beyond Reasonable Doubt).
//! Clauses (iii)–(iv) give the respondent a rebuttal defense by
//! clear-and-convincing evidence that the same action would have been
//! taken absent the protected activity. This ontology models the
//! *complainant's* contributing-factor concept; the reference-layer
//! `ClearAndConvincing` covers the respondent's defense.
//!
//! # Multi-statute reach
//!
//! AIR21's framework is incorporated by express cross-reference into
//! every modern federal whistleblower-retaliation regime:
//!
//! - 18 U.S.C. § 1514A(b)(2)(C) — Sarbanes–Oxley § 806
//! - 49 U.S.C. § 20109(d)(2) — Federal Rail Safety Act
//! - 12 U.S.C. § 5567(c)(3) — Consumer Financial Protection Act
//! - 41 U.S.C. § 4712(b)(4) — NDAA federal contractor retaliation
//! - 6 U.S.C. § 1142(b)(2)(B) — National Transit Systems Security Act
//!
//! A single ontology with one `ContributingFactor` leaf serves all
//! such statutes; the per-statute cross-reference is captured at the
//! composition layer (`social::compliance::compositions::proof_framework`).
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
//!   [application layer] social::compliance::proof_standards::air21_framework
//!     Air21ProofStandard
//!       └── ContributingFactor     tier 0   ← below the reference partition
//! ```
//!
//! # Why this lives at the compliance proof_standards/, not the reference layer
//!
//! `ContributingFactor` is statute-specific — it surfaces in AIR21,
//! SOX, FRSA, CFPA, NDAA and other federal whistleblower regimes
//! that incorporate AIR21's procedural framework. It is *not* a
//! general-jurisprudence concept; it would fail Guarino & Welty's
//! OntoClean filter (type vs role) if added as a reference-layer
//! leaf. Per the bottom-up rule, this ontology stays as close to its
//! primary source (AIR21 § 42121(b)(2)(B)) as possible and cites only
//! statutory text. Case-law refinements (no retaliatory intent
//! required; temporal proximity sufficient) live at their respective
//! case homes.
//!
//! # Statutory sources only
//!
//! - **49 U.S.C. § 42121(b)(2)(B)(i)–(iv)** — the four-clause
//!   burden-shifting structure: complainant's prima facie
//!   contributing-factor showing (clauses (i)–(ii)) and respondent's
//!   clear-and-convincing same-action defense (clauses (iii)–(iv)).
//! - **18 U.S.C. § 1514A(b)(2)(C)** — SOX whistleblower civil-action
//!   procedure cross-reference into the AIR21 burden-shifting
//!   framework.
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
