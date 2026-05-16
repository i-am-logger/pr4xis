//! Proof-framework compositions — typed syntheses of multiple
//! loaded statutes (and, when the PDF loader lands, case-law
//! decisions) into unified doctrinal frameworks.
//!
//! A *composition* is a typed view over already-loaded `Statute` /
//! `Decision` instances plus the cross-source references that connect
//! them. It is **not** a new ontology with hand-coded concept enums —
//! every term it mentions resolves to a CURIE in some loaded source's
//! structural data, and the cross-source references are static data
//! (Rust statics holding `(curie, RelationType, curie)` triples)
//! rather than encoded enum variants. This keeps the composition
//! consistent with the bottom-up rule.
//!
//! # First composition: SOX retaliation proof framework
//!
//! [`proof_framework::sox_retaliation::framework`] composes
//! `sox_1514a@2002` and `air21_42121@2010` into the unified
//! whistleblower-retaliation framework that 18 U.S.C. § 1514A(b)(2)(C)
//! creates by cross-reference. Three typed cross-references connect
//! SOX's procedural and causation terms to AIR21's investigation and
//! burden-shifting framework. Future commits will add case-law
//! decisions (Murray, Lawson, Sylvester, …) once the PDF loader can
//! extract them.
//!
//! # Composition vs. statute
//!
//! - `Statute` (in `compliance::statutes::statute`) — a single
//!   primary source's terms + intra-statute relations. Validates
//!   every relation endpoint resolves *within* the statute.
//! - `ProofFramework` (here) — a typed wrapper around N statutes +
//!   M decisions + K cross-source references. Validates that every
//!   cross-reference endpoint resolves into one of the bundled
//!   sources. Carries authority-strength tags so conflict resolution
//!   can pick winners when multiple sources disagree.
//!
//! # Literature
//!
//! - **Hart, H.L.A. (1961)** *The Concept of Law*, Oxford — Ch. VI
//!   establishes the legal system as a *unified* normative order;
//!   compositions of primary rules sourced from distinct statutes
//!   require an explicit rule-of-recognition treatment.
//! - **Dickerson, Reed (1975)** *The Interpretation and Application
//!   of Statutes*, Little, Brown — §6.4 covers statutory cross-
//!   reference (a/k/a "incorporation by reference") doctrine.
//! - **Eskridge, William N., Frickey, Philip P. & Garrett, Elizabeth
//!   (latest ed.)** *Legislation and Statutory Interpretation*,
//!   Foundation Press — sources-of-law hierarchy; treatment of
//!   incorporated statutes.
//! - **Sartor, Giovanni (2005)** *Legal Reasoning: A Cognitive
//!   Approach to the Law*, Springer (Treatise of Legal Philosophy and
//!   General Jurisprudence vol. 5) — Ch. 21 formal modeling of
//!   normative authority including composition rules.

pub mod audit;
pub mod proof_framework;

pub use audit::{
    CompositionAuditReport, CrossRefAuditResult, CrossRefClassification,
    audit_composition_cross_refs,
};
pub use proof_framework::{
    CrossReference, CrossReferenceKind, ProofFramework, ProofFrameworkBuildError,
};
