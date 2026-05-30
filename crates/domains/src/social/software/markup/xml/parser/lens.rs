//! `XmlLens` — the [`WellBehavedLens`] binding the literature-
//! grounded parser ([`super::grammar::parse_document`]) to the
//! symmetric serializer ([`super::serializer::serialize_document`])
//! with W3C XML Canonicalization 1.1 (Boyer & Marcy 2008) as the
//! canonical-form witness.
//!
//! Per Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS
//! 29(3) §2.2, the lens laws are:
//!
//! - **GetPut** — `get(put(t)) = t`. The parser/serializer pair is
//!   designed so that an [`XmlDocument`] re-serialized and re-parsed
//!   yields the same typed value modulo CharData escape-form
//!   normalization (e.g. `&gt;` ↔ `>` outside CDATA, which
//!   `parse_document` collapses to the literal character).
//!
//! - **PutGet** — `canonical(put(get(s))) = canonical(s)`. The
//!   serializer emits the C14N 1.1 §3.5 escape forms directly so
//!   that running the canonicalizer over its output produces
//!   bytes identical to the canonicalizer's output over the
//!   original input. The
//!   [`XmlLens::assert_put_get_law`](
//!   crate::formal::meta::well_behaved_lens::WellBehavedLens::assert_put_get_law)
//!   default implementation tests this.
//!
//! - **PutPut** (idempotency) — `put` is a pure function of its
//!   `XmlDocument` argument, so `put(t) = put(t)` byte-identical
//!   trivially holds. The harness covers this via the M4.θ.2
//!   harness's `apply_put_after_get`.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use super::super::ontology::XmlDocument;
use super::grammar::{XmlParseError, parse_document};
use super::serializer::serialize_document;
use crate::formal::meta::well_behaved_lens::{
    WellBehavedLens, canonical::CanonicalizationError, canonical::xml::canonicalize,
};

/// Lens for `bytes ↔ XmlDocument` per the W3C XML 1.0
/// Infoset, with W3C C14N 1.1 as the canonical form.
#[derive(Debug)]
pub struct XmlLens;

/// Wrapper for [`super::grammar::XmlParseError`] /
/// [`crate::formal::meta::well_behaved_lens::canonical::CanonicalizationError`]
/// that the lens trait's `Error` associated type can carry.
#[derive(Debug)]
pub enum XmlLensError {
    Parse(XmlParseError),
    Canonical(CanonicalizationError),
}

impl core::fmt::Display for XmlLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "XML parse: {e}"),
            Self::Canonical(e) => write!(f, "XML canonicalize: {e}"),
        }
    }
}

impl std::error::Error for XmlLensError {}

impl From<XmlParseError> for XmlLensError {
    fn from(e: XmlParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<CanonicalizationError> for XmlLensError {
    fn from(e: CanonicalizationError) -> Self {
        Self::Canonical(e)
    }
}

impl WellBehavedLens for XmlLens {
    type Target = XmlDocument;
    type Error = XmlLensError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        parse_document(bytes).map_err(Into::into)
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(serialize_document(target))
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        canonicalize(bytes).map_err(Into::into)
    }
}
