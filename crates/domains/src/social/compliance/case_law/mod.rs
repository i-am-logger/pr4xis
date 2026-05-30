//! Per-case homes for U.S. judicial precedents — the case-law
//! counterpart to `social::compliance::statutes`.
//!
//! Each case lives in its own sub-module and (when its structural data
//! is pinned in `praxis.lock`) exposes a live [`Decision`] instance
//! via a `decision()` accessor. The pattern mirrors
//! `social::compliance::statutes::sox_1514a` — registration in
//! `praxis.toml`, structural extraction in `praxis.lock`, runtime
//! materialization through `Decision::from_structural`.
//!
//! # Why `Decision` and not `Statute`
//!
//! Statutes and cases are different *source-taxonomy* types
//! (`formal::meta::source_taxonomy` distinguishes `Statute`,
//! `Regulation`, `CaseLaw`, …). A case opinion has structural
//! features a statute lacks — *holdings* articulated, *dicta*
//! distinguishable, a *disposition* (affirmed / reversed / remanded),
//! a court, a date, a *binding scope* derived from the issuing
//! court. Sharing the `Statute` runtime would conflate these.
//!
//! Both types carry CURIE-typed terms and typed relations, so the
//! *internal* term/relation structure is largely parallel; the
//! divergence is in metadata (holding vs. statutory text, disposition
//! vs. effective date) and in the `AuthorityStrengthConcept` each
//! self-reports.
//!
//! # Authority strength
//!
//! Every `Decision` carries an
//! [`AuthorityStrengthConcept`](crate::social::judicial::authority_strength::ontology::AuthorityStrengthConcept)
//! — typically
//! `SupremeCourtPrecedent`, `ControllingCircuitPrecedent`,
//! `AdministrativeReviewBoardDecision`, or `DistrictCourtPrecedent`.
//! That tag participates in the binding-force ordering when the
//! proof-framework composition layer resolves conflicts between
//! authorities.
//!
//! # Literature
//!
//! - **Garner, Bryan A. et al. (2016)** *The Law of Judicial
//!   Precedent*, Thomson Reuters — comprehensive treatise on
//!   holdings, dicta, distinguishing, stare decisis, and the
//!   horizontal/vertical precedent hierarchy.
//! - **Schauer, Frederick (2009)** *Thinking Like a Lawyer*, Harvard
//!   — Ch. 3 "The Practice and Problems of Precedent."
//! - **The Bluebook (21st ed.)** Rule 10 — case citation formats and
//!   subsequent-history conventions.

pub mod decision;

pub use decision::{Decision, DecisionConstructError, Disposition};
