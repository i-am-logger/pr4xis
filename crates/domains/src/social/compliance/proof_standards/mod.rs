//! Statute-derived proof-standard partitions — application-layer
//! ontologies that adjoin into the reference proof-standard tier at
//! `social::judicial::proof_standard`.
//!
//! Each sub-module captures a single federal proof-standard regime
//! shared across multiple statutes (whistleblower retaliation,
//! securities fraud, civil rights enforcement). Membership reflects
//! statutory cross-reference, not statutory home: e.g., SOX § 1514A,
//! FRSA § 20109, CFPA § 1057, NDAA § 4712 all incorporate the AIR21
//! § 42121(b)(2)(B) burden-shifting framework, so the single
//! `air21_framework` ontology serves them all.
//!
//! # Modular-ontology placement (Stuckenschmidt, Parent, Spaccapietra 2009)
//!
//! ```text
//!   [reference layer] social::judicial::proof_standard
//!     ProofStandard
//!       ├── Preponderance          tier 1
//!       ├── ClearAndConvincing     tier 2
//!       └── BeyondReasonableDoubt  tier 3
//!
//!   [application layer] social::compliance::proof_standards::{...}
//!     Air21ProofStandard
//!       └── ContributingFactor     tier 0   ← below the reference partition
//!     (future) FelaProofStandard …
//! ```
//!
//! # Why these don't live at the reference layer
//!
//! Per Guarino & Welty (2002, OntoClean, CACM 45(2):61–65), each tier
//! is a *statutory role* rather than a domain-general *type*. The
//! reference layer's three classical tiers (Preponderance / Clear-and-
//! Convincing / BeyondReasonableDoubt) name evidentiary thresholds in
//! the abstract; statute-derived tiers like ContributingFactor depend
//! on a particular legislative regime to exist. Mixing the two would
//! contaminate the reference partition with statute-specific concepts.
//!
//! # Holding pattern — M4.κ.4 auto-generation
//!
//! These sub-modules are interim hand-coded versions of what the
//! M4.κ.4 doctrine-discovery engine will eventually auto-generate from
//! the loaded statute text. The four-clause burden-shifting structure
//! at 49 U.S.C. § 42121(b)(2)(B)(i)–(iv) is mechanically extractable
//! by Formal Concept Analysis (Ganter & Wille 1999 §1.3); until the
//! end-to-end FCA-from-USLM pipeline lands, the ontology stays
//! hand-coded with the same literature grounding that
//! `social::judicial::proof_standard` carries.

pub mod air21_framework;
