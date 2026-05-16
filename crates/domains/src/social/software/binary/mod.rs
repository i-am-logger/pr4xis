//! Binary file formats — opaque-byte-stream content types whose structural
//! decoding depends on a format-specific specification.
//!
//! Sibling to `software::markup` (XML-family, ECMA-376, OOXML) and
//! `software::protocols` (HTTP, etc.). Where markup formats expose
//! structure via printable delimiters and a single grammar, binary
//! formats are decoded against a format-specific byte-level
//! specification (PDF: ISO 32000-2; future siblings: ZIP/EPUB, PNG,
//! TIFF, …) before any higher-layer reading can run.
//!
//! Each binary format is its own praxis-way ontology — concept enum,
//! kinded relations, axioms grounded in the spec, runtime decoder
//! that materializes typed values. See `pdf::ontology` for the
//! reference instance.

pub mod pdf;
