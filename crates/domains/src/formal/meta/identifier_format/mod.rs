//! Identifier format — the syntactic form of an identifier string.
//!
//! Distinct from `artifact_identity` (which models identity-verification
//! schemes — content hashes, cryptographic signatures, persistent
//! identifiers with resolvers, self-describing metadata). This ontology
//! types the *syntactic form* of an identifier — independent of how
//! that identifier verifies an artifact.
//!
//! ```text
//! IdentifierFormat
//!   ├── Curie   — W3C Compact URI: `prefix:local` (e.g., `sox_1514a:a`)
//!   ├── Uuid    — RFC 4122 universally unique identifier (128 bits hex)
//!   ├── Uri     — RFC 3986 generic URI (`scheme:hierarchical-part`)
//!   └── Oid     — ISO 8824-1 / ITU-T X.660 object identifier (dot-numeric)
//! ```
//!
//! # Why these four
//!
//! Each leaf names a distinct, widely-published syntactic specification
//! with its own grammar:
//!
//! - **CURIE** (W3C Note 2010-12-16) — compact form for tools and
//!   markup; widely used in RDF, linked-data, OWL, and as a
//!   namespace-qualified key form in ontology serialisations.
//! - **UUID** (RFC 4122) — 128-bit value with five variants and several
//!   versions; globally unique without centralized issuance.
//! - **URI** (RFC 3986) — the generic identifier grammar from which
//!   URLs, URNs, DOIs, and most web-resolvable forms inherit.
//! - **OID** (ISO/IEC 8824-1; X.660 / X.680) — ITU-T's hierarchical
//!   integer-dotted identifier system used by SNMP, ASN.1, X.509,
//!   etc.
//!
//! # Literature
//!
//! - **W3C CURIE Syntax 1.0** (Birbeck & McCarron 2010), *W3C
//!   Working Group Note* — defines the `prefix:local` compact-URI
//!   syntax.
//! - **RFC 4122** (Leach, Mealling, Salz 2005) "A Universally Unique
//!   IDentifier (UUID) URN Namespace", IETF — UUID grammar and the five
//!   versions.
//! - **RFC 3986** (Berners-Lee, Fielding, Masinter 2005) "Uniform
//!   Resource Identifier (URI): Generic Syntax", IETF — the canonical
//!   URI grammar.
//! - **ISO/IEC 8824-1:2021** *Information technology — Abstract Syntax
//!   Notation One (ASN.1): Specification of basic notation* — OID
//!   grammar (also ITU-T X.680 §32).
//! - **ITU-T Recommendation X.660** (2011) *Information technology —
//!   Procedures for the operation of object identifier registration
//!   authorities: General procedures and top arcs of the international
//!   object identifier tree* — OID hierarchy semantics.

pub mod ontology;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
use alloc::{borrow::Cow, string::String, string::ToString, vec, vec::Vec};

use self::ontology::IdentifierFormatConcept;

/// A typed identifier value with its syntactic format. Reach the raw
/// string via [`Identifier::value`]; the `format` declares which
/// grammar the value conforms to. Runtime constructors validate the
/// value against the format's grammar; the const constructor
/// [`Identifier::from_codegen_static`] skips validation for values the
/// codegen emitter has already validated at build time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub format: IdentifierFormatConcept,
    value: Cow<'static, str>,
}

impl Identifier {
    /// Raw string form of the identifier.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Const constructor for codegen-emitted identifiers. The emitter
    /// validates the value at build time against the format grammar;
    /// the const form skips runtime re-validation. Never call this with
    /// hand-written literals — use [`Identifier::curie`] /
    /// [`Identifier::uslm_urn`] / etc. for runtime values so the
    /// grammar checks run.
    pub const fn from_codegen_static(format: IdentifierFormatConcept, value: &'static str) -> Self {
        Self {
            format,
            value: Cow::Borrowed(value),
        }
    }
}

/// Errors when constructing a typed `Identifier` from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierParseError {
    /// Value is empty.
    Empty,
    /// Value violates the format's grammar.
    InvalidGrammar {
        format: IdentifierFormatConcept,
        reason: &'static str,
    },
}

impl Identifier {
    /// Construct a CURIE-typed identifier. Validates `prefix:local`
    /// shape per W3C CURIE Syntax 1.0 §2 — exactly one `:` separator,
    /// prefix and local parts non-empty.
    pub fn curie(value: impl Into<String>) -> Result<Self, IdentifierParseError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        // CURIE: prefix ":" local, exactly one colon (no scheme-style "://"),
        // both halves non-empty.
        let parts: Vec<&str> = value.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Curie,
                reason: "CURIE requires exactly one ':' separator",
            });
        }
        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Curie,
                reason: "CURIE prefix and local part must both be non-empty",
            });
        }
        Ok(Self {
            format: IdentifierFormatConcept::Curie,
            value: Cow::Owned(value),
        })
    }

    /// Construct a UUID-typed identifier. Validates per RFC 4122 §3 —
    /// 36-character canonical form `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// (hex digits + four hyphens at fixed positions).
    pub fn uuid(value: impl Into<String>) -> Result<Self, IdentifierParseError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        if value.len() != 36 {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Uuid,
                reason: "UUID canonical form is 36 characters",
            });
        }
        for (i, c) in value.chars().enumerate() {
            let expected_hyphen = matches!(i, 8 | 13 | 18 | 23);
            if expected_hyphen {
                if c != '-' {
                    return Err(IdentifierParseError::InvalidGrammar {
                        format: IdentifierFormatConcept::Uuid,
                        reason: "UUID hyphens must be at positions 8, 13, 18, 23",
                    });
                }
            } else if !c.is_ascii_hexdigit() {
                return Err(IdentifierParseError::InvalidGrammar {
                    format: IdentifierFormatConcept::Uuid,
                    reason: "UUID non-hyphen positions must be hex digits",
                });
            }
        }
        Ok(Self {
            format: IdentifierFormatConcept::Uuid,
            value: Cow::Owned(value),
        })
    }

    /// Construct a URI-typed identifier. Validates per RFC 3986 §3 —
    /// the value must begin with a valid scheme (alpha followed by
    /// alphanumeric/+/-/.) and a `:`. Beyond that, no further validation
    /// is performed at this layer; consumers needing strict URI parsing
    /// can apply a dedicated parser.
    pub fn uri(value: impl Into<String>) -> Result<Self, IdentifierParseError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        let mut chars = value.chars();
        let first = chars.next();
        if !matches!(first, Some(c) if c.is_ascii_alphabetic()) {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Uri,
                reason: "URI scheme must begin with an alphabetic character",
            });
        }
        let mut saw_colon = false;
        for c in chars {
            if c == ':' {
                saw_colon = true;
                break;
            }
            if !(c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
                return Err(IdentifierParseError::InvalidGrammar {
                    format: IdentifierFormatConcept::Uri,
                    reason: "URI scheme contains an invalid character",
                });
            }
        }
        if !saw_colon {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Uri,
                reason: "URI must contain a scheme separator ':'",
            });
        }
        Ok(Self {
            format: IdentifierFormatConcept::Uri,
            value: Cow::Owned(value),
        })
    }

    /// Construct a USLM-URN-typed identifier per LRC USLM XML User
    /// Guide §11.5 "Identifiers" (path per §13 Referencing Model).
    ///
    /// USLM URNs are hierarchical relative-reference paths beginning
    /// with `/us/` that identify a U.S. Code component:
    ///
    /// ```text
    /// /us/usc/t18                       — Title
    /// /us/usc/t18/s1514A                — Section
    /// /us/usc/t18/s1514A/a/1/A          — Subdivision (a)(1)(A)
    /// ```
    ///
    /// Grammar (LRC USLM User Guide §11.5 Identifiers): the path starts with `/us/`, is
    /// composed of segments separated by `/`, each segment is a
    /// non-empty ASCII run (alphanumeric plus `-`, `_`, `.`). The
    /// USLM URN namespace is documented at uscode.house.gov and
    /// resolves to the LRC's authoritative XML.
    pub fn uslm_urn(value: impl Into<String>) -> Result<Self, IdentifierParseError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        if !value.starts_with("/us/") {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::UslmUrn,
                reason: "USLM URN must begin with `/us/` per LRC USLM User Guide §11.5 Identifiers",
            });
        }
        for segment in value.trim_start_matches('/').split('/') {
            if segment.is_empty() {
                return Err(IdentifierParseError::InvalidGrammar {
                    format: IdentifierFormatConcept::UslmUrn,
                    reason: "USLM URN segments must be non-empty",
                });
            }
            for c in segment.chars() {
                if !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
                    return Err(IdentifierParseError::InvalidGrammar {
                        format: IdentifierFormatConcept::UslmUrn,
                        reason: "USLM URN segments may only contain ASCII alphanumeric, `-`, `_`, `.`",
                    });
                }
            }
        }
        Ok(Self {
            format: IdentifierFormatConcept::UslmUrn,
            value: Cow::Owned(value),
        })
    }

    /// Construct an OID-typed identifier. Validates per ISO 8824-1
    /// §32 — dot-separated non-negative integer arcs, at least two arcs.
    pub fn oid(value: impl Into<String>) -> Result<Self, IdentifierParseError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        let arcs: Vec<&str> = value.split('.').collect();
        if arcs.len() < 2 {
            return Err(IdentifierParseError::InvalidGrammar {
                format: IdentifierFormatConcept::Oid,
                reason: "OID requires at least two dot-separated arcs",
            });
        }
        for arc in arcs {
            if arc.is_empty() || !arc.chars().all(|c| c.is_ascii_digit()) {
                return Err(IdentifierParseError::InvalidGrammar {
                    format: IdentifierFormatConcept::Oid,
                    reason: "OID arcs must be non-empty decimal-digit sequences",
                });
            }
        }
        Ok(Self {
            format: IdentifierFormatConcept::Oid,
            value: Cow::Owned(value),
        })
    }
}
