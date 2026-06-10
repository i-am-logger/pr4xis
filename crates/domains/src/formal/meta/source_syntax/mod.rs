//! Source-syntax stratum — the concrete-syntax decisions a byte-exact
//! serialization records beyond the abstract Information Set (STAGE 2.1 of
//! the universal compiler / task #34).
//!
//! - [`ontology`] — the VOCABULARY: the kinds of byte-affecting decision
//!   (`AttributeOrder`, `WhitespaceFormatting`, `EmptyElementForm`, …) the
//!   XML Information Set underdetermines and Canonical XML normalizes away;
//!   cited, format-agnostic, always compiled, holds no instance data.
//!
//! The per-node instance decisions — stored in the per-source `.prx`
//! envelope with their own content-address, deliberately OUT of the
//! ontology's identity root — and the byte-exact writers that consume them
//! to retire the stored complement are STAGE 2.1b / 2.3.

pub mod ontology;
