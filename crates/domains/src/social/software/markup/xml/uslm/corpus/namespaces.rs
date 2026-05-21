//! USLM-relevant XML namespace URIs.
//!
//! Element membership in USLM (and its co-namespaces) is determined
//! by W3C XML Namespaces 1.0 §6 ("Applying Namespaces to Elements
//! and Attributes") — an element is in a given namespace iff its
//! qualified-name's resolved namespace URI equals the relevant
//! constant. Mechanical heuristics ("no prefix means USLM") are not
//! equivalent: they coincide on LRC-conformant documents but the
//! spec-level membership rule is the namespace-URI match.

/// The XML namespace URI USLM elements live in.
///
/// Declared by the LRC on the `<uscDoc>` root via
/// `xmlns="http://xml.house.gov/schemas/uslm/1.0"`. Cited per the
/// LRC's USLM XML User Guide § "Namespaces".
pub const USLM_NAMESPACE_URI: &str = "http://xml.house.gov/schemas/uslm/1.0";

/// Dublin Core element namespace, used by USLM `<meta>` blocks
/// (e.g. `<dc:title>`, `<dc:creator>`). Defined by DCMI Metadata
/// Terms (Dublin Core Metadata Initiative, ISO 15836-1:2017).
/// Distinct from USLM despite the local-name collision on
/// `<title>`; the namespace URI is the load-bearing discriminator.
pub const DUBLIN_CORE_NAMESPACE_URI: &str = "http://purl.org/dc/elements/1.1/";

/// XHTML namespace URI per W3C XHTML 1.0 (Second Edition, 2002).
/// USLM uses XHTML for `<table>` markup inside USC titles, retaining
/// the HTML tabular model rather than defining USLM-native rows.
pub const XHTML_NAMESPACE_URI: &str = "http://www.w3.org/1999/xhtml";
