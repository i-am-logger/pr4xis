//! Legal authority strength — the binding/persuasive hierarchy of
//! American jurisprudence and the vertical tier ordering within each.
//!
//! ```text
//! AuthorityStrength
//!   ├── BindingAuthority                          (must be followed when applicable)
//!   │     ├── ConstitutionalText                  tier 9
//!   │     ├── FederalStatute                      tier 8
//!   │     ├── SupremeCourtPrecedent               tier 7
//!   │     ├── FederalRegulation                   tier 6
//!   │     └── ControllingCircuitPrecedent         tier 5
//!   └── PersuasiveAuthority                       (may be followed)
//!         ├── AdministrativeReviewBoardDecision   tier 4
//!         ├── SisterCircuitPrecedent              tier 3
//!         ├── DistrictCourtPrecedent              tier 2
//!         └── SecondarySource                     tier 1
//! ```
//!
//! Nine leaves, partitioned by `BindingForceCategory` into Binding
//! (tier ≥ 5) and Persuasive (tier < 5). The `BindingForceOf` quality
//! assigns each leaf a strict integer tier — total on leaves, `None`
//! on the abstract root and the two branches.
//!
//! # Why this lives in `judicial::` rather than `compliance::`
//!
//! Authority strength is a *meta* feature of legal sources — the same
//! framework applies to constitutional law, criminal procedure,
//! whistleblower retaliation, contract disputes, every domain. It is
//! reference-layer (Stuckenschmidt et al. 2009) and shared across all
//! statute / case-law / regulation modules. Per-source modules
//! (`social::compliance::statutes::sox_1514a`,
//! `social::compliance::case_law::murray_v_ubs_2024`, …) tag their
//! contents with an `AuthorityStrengthConcept` to participate in this
//! ordering.
//!
//! # Hierarchy vs. jurisdiction
//!
//! `BindingForceOf` captures the *vertical* tier — how high in the
//! abstract hierarchy. Whether a tier-5 ControllingCircuitPrecedent
//! actually binds a given district court depends on *jurisdiction* —
//! the Tenth Circuit binds the District of Colorado, but is only
//! persuasive in the Eastern District of Texas. The
//! `JurisdictionScopeOf` quality captures this horizontal dimension
//! by attaching a typed `Identifier` (CURIE into a jurisdiction
//! ontology) to each authority instance. The two dimensions are
//! orthogonal.
//!
//! # Literature
//!
//! - **Hart, H.L.A. (1961)** *The Concept of Law*, Oxford — Ch. VI,
//!   the *rule of recognition* establishes the ultimate criteria of
//!   legal validity and the primary/secondary rule distinction.
//! - **Schauer, Frederick (2009)** *Thinking Like a Lawyer: A New
//!   Introduction to Legal Reasoning*, Harvard — Ch. 3 "The Practice
//!   and Problems of Precedent" and Ch. 5 "Authority and Authorities"
//!   articulate the binding/persuasive distinction and the vertical
//!   hierarchy.
//! - **Garner, Bryan A. et al. (2016)** *The Law of Judicial
//!   Precedent*, Thomson Reuters — comprehensive treatise (910 pp.)
//!   on stare decisis, vertical and horizontal precedent, binding vs.
//!   persuasive authority.
//! - **Sartor, Giovanni (2005)** *Legal Reasoning: A Cognitive
//!   Approach to the Law*, Springer (Treatise of Legal Philosophy and
//!   General Jurisprudence vol. 5) — Ch. 21 formal modeling of
//!   normative authority, defeasibility, and conflict resolution.
//! - **Eskridge, Frickey, Garrett, Brudney (latest ed.)** *Legislation
//!   and Statutory Interpretation*, Foundation Press — Ch. 1 sources
//!   and hierarchy of law.
//! - **Marbury v. Madison**, 5 U.S. (1 Cranch) 137 (1803) — judicial
//!   review and constitutional supremacy.
//! - **U.S. Const. Art. VI, cl. 2** — Supremacy Clause: the
//!   Constitution, federal statutes pursuant to it, and federal
//!   treaties are "the supreme Law of the Land."
//! - **Erie R.R. v. Tompkins**, 304 U.S. 64 (1938) — vertical
//!   federal-state hierarchy in diversity jurisdiction.
//! - **Chevron U.S.A. v. NRDC**, 467 U.S. 837 (1984) — agency
//!   regulation receives deference when statute is ambiguous;
//!   establishes FederalRegulation's binding tier conditioned on
//!   delegation.
//! - **Skidmore v. Swift & Co.**, 323 U.S. 134 (1944) — agency
//!   interpretations without Chevron protection receive *persuasive*
//!   weight, calibrated to expertise; grounds the
//!   AdministrativeReviewBoardDecision tier.
//! - **Guarino & Welty (2002)** "Evaluating Ontological Decisions
//!   with OntoClean", *CACM* 45(2):61–65 — type vs. role distinction;
//!   rationale for keeping jurisdictional scope (a *role* attached to
//!   each instance) separate from binding force (a *type*-level
//!   property).
//! - **Stuckenschmidt, Parent, Spaccapietra (eds.) (2009)** *Modular
//!   Ontologies*, Springer LNCS 5445 — three-tier
//!   foundational/reference/application architecture; this ontology
//!   sits at the reference layer.

pub mod ontology;

#[cfg(test)]
mod tests;
