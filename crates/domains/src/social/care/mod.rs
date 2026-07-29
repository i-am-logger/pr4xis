//! Care — family caregiving and HCBS workforce/compliance domain lexicons.
//! DOLCE: SocialObject (care services, programs, and the institutional
//! vocabulary regulating them).
//!
//! The two registered WN-LMF definitional lexicons of the Caregiver AI
//! Challenge, each a [`SourceTaxonomyConcept`](crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept)
//! `DomainLexicon` leaf with its own kind (so chat residency selects
//! definitional lexicons BY KIND, never by name, and never confuses them
//! with the closed-class recognizer `LegalLexicon`):
//!
//! - [`caregiving_lexicon`] — Track 1, `caregiving_lexicon@2026`
//!   (`CaregivingLexicon`): family-caregiving terms of art. Statutory
//!   glosses verified against the loaded Title 42 corpus; Medicaid HCBS
//!   waiver, Medicare home health / hospice, OAA, NIA dementia-care, and
//!   ACL guardianship vocabulary.
//! - [`hcbs_compliance_lexicon`] — Track 2, `hcbs_compliance_lexicon@2026`
//!   (`HcbsComplianceLexicon`): EVV / HCBS workforce-compliance terms of
//!   art. 42 USC 1396b(l) EVV definitions, the 2024 Ensuring Access to
//!   Medicaid Services final rule (89 FR 40542), settings rule, billing,
//!   and program-integrity vocabulary.
//!
//! Both ride the ONE generic WN-LMF lexicon bridge
//! ([`lexicon_runtime_ontology`](crate::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology))
//! parameterized by their registered `(name, version, committed .prx)` —
//! zero lexicon-specific projection code, the `us_legal_lexicon` precedent.

pub mod caregiving_lexicon;
pub mod hcbs_compliance_lexicon;
