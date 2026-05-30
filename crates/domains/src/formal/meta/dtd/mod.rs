//! DTD — W3C XML 1.0 Document Type Definitions as a Praxis ontology.
//!
//! Parallel to [`super::xsd`] for XSD. DTDs predate XSD as the
//! original schema language for XML, defined in W3C XML 1.0 §2.8 +
//! §3.2–§3.4 + §4 (Bray, Paoli, Sperberg-McQueen, Maler & Yergeau
//! 2008). Loaded so vocabularies that publish their schema as DTD
//! (Global WordNet's WN-LMF, HTML 4.01, DocBook, NewsML, …) become
//! first-class praxis ontology instances through a single
//! `parse_dtd` functor, the same shape as the XSD reader's
//! [`super::xsd::from_xml::project_from_xml_document`].
//!
//! ## Module layout
//!
//! - [`ontology`] — the DTD concept inventory (DocumentTypeDefinition,
//!   ElementDecl, AttListDecl, EntityDecl, NotationDecl + sub-kinds),
//!   cited section-by-section to W3C XML 1.0 Fifth Edition.
//! - [`parser`] — `parse_dtd(bytes) -> DtdSchema` recognising the
//!   four declaration kinds in document order. Parameter-entity
//!   expansion and conditional sections (§4.4 + §3.4) are deferred —
//!   no entry in the praxis source registry uses them.
//! - [`lens`] — [`lens::DtdLens`] binding the byte hop into the
//!   round-trip harness ([`crate::formal::meta::well_behaved_lens::WellBehavedLens`]).
//!
//! ## Citations
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (2008) *Extensible Markup Language (XML) 1.0
//!   (Fifth Edition)*, W3C Recommendation 26 November 2008 — §2.8
//!   document type declaration, §3.2 element-type declarations,
//!   §3.3 attribute-list declarations, §4.2 entity declarations,
//!   §4.7 notation declarations.

pub mod lens;
pub mod ontology;
pub mod parser;

#[doc(inline)]
pub use lens::{DtdLens, DtdLensError, DtdSchema};
#[doc(inline)]
pub use ontology::{DtdConcept, EntityKind};
#[doc(inline)]
pub use parser::{DtdDecl, DtdDeclaration, parse_dtd};
