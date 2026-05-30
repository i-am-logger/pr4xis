//! Proof standard — the fraction-of-evidence tier required for a party
//! to carry its burden on a given issue.
//!
//! Praxis captures the three classical proof standards of U.S. civil
//! and criminal evidence law:
//!
//! ```text
//! ProofStandard
//!   ├── Preponderance            — civil default; > 50% probability
//!   ├── ClearAndConvincing       — heightened civil; ~ 75% probability
//!   └── BeyondReasonableDoubt    — criminal; ~ 95% probability
//! ```
//!
//! The three leaves admit a strict **stringency order** captured by the
//! `StringencyOf` quality: `Preponderance < ClearAndConvincing <
//! BeyondReasonableDoubt`. The order is *total* on the leaves; the
//! root `ProofStandard` is abstract.
//!
//! # Reference layer only
//!
//! This ontology is the **reference (domain-general)** layer
//! (Stuckenschmidt, Parent, Spaccapietra 2009). Statute-specific tiers
//! — e.g., the contributing-factor / clear-and-convincing-rebuttal
//! burden-shifting framework of AIR21 § 42121(b)(2)(B) (applied to
//! SOX 1514A whistleblower actions) — live in their own
//! **statute-specific (application)** ontologies, codegen'd from
//! `praxis.lock`. Statute-specific concepts adjoin into the reference
//! layer through the `SourceTaxonomy` `Adjoins` graph
//! (`Statute ⊣ LegalLexicon`), not by being added as leaves here.
//! That keeps this ontology's literature free of statute-specific
//! cites and its concept partition free of statute-specific tiers.
//!
//! # Literature
//!
//! - **McCauliff (1982)** "Burdens of Proof: Degrees of Belief, Quanta
//!   of Evidence, or Constitutional Guarantees?" *University of
//!   Pittsburgh Law Review* 35:1293–1335 — empirical study of
//!   judges' probabilistic interpretations of the three classical
//!   tiers.
//! - **Brilmayer (1990)** "Second-Order Evidence and Bayesian Logic"
//!   *Boston University Law Review* 66:673–701 — Bayesian
//!   formalization of the stringency tiers.
//! - **In re Winship**, 397 U.S. 358 (1970) — establishes
//!   BeyondReasonableDoubt as the constitutional standard for every
//!   element of a criminal offense.
//! - **McCormick on Evidence (Strong et al., 8th ed. 2022)** §339-343
//!   — the leading U.S. evidence treatise's treatment of burdens of
//!   proof.
//! - **Federal Rules of Evidence (2024)** — the modern federal
//!   codification of evidence standards.
//! - **Guarino & Welty (2002)** "Evaluating Ontological Decisions with
//!   OntoClean", *Communications of the ACM* 45(2):61–65 — type vs
//!   role distinction; rationale for keeping the reference layer
//!   free of application-specific concepts.
//! - **Stuckenschmidt, Parent, Spaccapietra (eds.) (2009)** *Modular
//!   Ontologies: Concepts, Theories and Techniques for Knowledge
//!   Modularization*, Springer LNCS 5445 — three-tier
//!   foundational/reference/application architecture.

pub mod ontology;

#[cfg(test)]
mod tests;
