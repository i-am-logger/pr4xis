//! Proof standard — the fraction-of-evidence tier required for a party
//! to carry its burden on a given issue.
//!
//! Different bodies of law impose different stringency tiers. Praxis
//! captures the four standards relevant to U.S. civil and SOX-1514A
//! whistleblower litigation:
//!
//! ```text
//! ProofStandard
//!   ├── ContributingFactor       — AIR21 § 42121(b)(2)(B) burden-shifting
//!   ├── Preponderance            — civil default; > 50% probability
//!   ├── ClearAndConvincing       — heightened civil; ~ 75% probability
//!   └── BeyondReasonableDoubt    — criminal; ~ 95% probability
//! ```
//!
//! The four leaves admit a natural **stringency order** captured by the
//! `StringencyOf` quality: `ContributingFactor < Preponderance <
//! ClearAndConvincing < BeyondReasonableDoubt`. The order is *total* on
//! the leaves; the root `ProofStandard` is abstract.
//!
//! # Why ContributingFactor is a separate concept
//!
//! SOX § 806 (18 U.S.C. § 1514A) incorporates the AIR21 burden-shifting
//! framework: the plaintiff need only show that protected activity was
//! a "contributing factor" in the adverse action. The defendant can
//! then rebut only by **clear and convincing** evidence that it would
//! have taken the same action absent the protected activity. The
//! *Lawson v. FMR LLC* (2014) decision (Supreme Court extending SOX
//! 1514A to private-contractor employees) cites this asymmetric
//! framework as a deliberate plaintiff-friendly tilt. ContributingFactor
//! is therefore lower than Preponderance for the plaintiff's prima
//! facie showing but interacts with a CC-standard defendant rebuttal.
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
//! - **Lawson v. FMR LLC**, 571 U.S. 429 (2014) — Supreme Court
//!   extension of SOX 1514A to private contractor employees;
//!   discusses the contributing-factor / clear-and-convincing
//!   asymmetry.
//! - **18 U.S.C. § 1514A(b)(2)(C)** — the SOX statute incorporating
//!   AIR21 § 42121(b)(2)(B)'s burden-shifting framework.
//! - **AIR21 — 49 U.S.C. § 42121(b)(2)(B)** — the original
//!   contributing-factor / clear-and-convincing-rebuttal statute.
//! - **Marx v. Schnuck Markets**, 869 F.3d 656 (8th Cir. 2014) and
//!   *Murray v. UBS Securities, LLC*, 601 U.S. 23 (2024) — appellate
//!   and Supreme Court clarification of the contributing-factor
//!   standard's pleading and proof requirements.

pub mod ontology;

#[cfg(test)]
mod tests;
