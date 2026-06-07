//! **RDF Dataset Canonicalization 1.0 (RDFC-1.0)** — an in-house,
//! conformant implementation of the W3C Recommendation
//! [REC-rdf-canon-20240521] (formerly URDNA2015).
//!
//! RDFC-1.0 maps any RDF dataset to a single *serialized canonical form*
//! (sorted canonical N-Quads) by assigning deterministic canonical
//! blank-node identifiers via iterative hashing. Two datasets canonicalize
//! to byte-identical output **iff** they are isomorphic (RDF 1.1 Concepts
//! dataset isomorphism) — including the cyclic and symmetric blank-node
//! structures that a naive content-addressed labelling cannot tell apart.
//!
//! ## What lives here
//!
//! - [`nquads`] — an N-Quads parser (reads the suite's `*-in.nq`) and the
//!   **canonical** N-Quads serializer (REC §"A Canonical form of
//!   N-Quads"), over praxis [`Quad`] / [`RdfTerm`](crate::social::software::markup::xml::rdf::term::RdfTerm) / [`Triple`](crate::social::software::markup::xml::rdf::term::Triple).
//! - [`algorithm`] — the four sub-algorithms (§4.4.3 Canonicalization,
//!   §4.5 Issue Identifier, §4.6.3 Hash First Degree Quads, §4.8.3 Hash
//!   N-Degree Quads, plus §4.7 Hash Related Blank Node), hashed with
//!   SHA-256 (default) or SHA-384 via [`HashAlgorithm`].
//!
//! ## Type reuse
//!
//! The term model is praxis [`RdfTerm`](crate::social::software::markup::xml::rdf::term::RdfTerm)/[`Triple`](crate::social::software::markup::xml::rdf::term::Triple)
//! (`super::term`) — this module adds only the [`Quad`] (a triple plus an
//! optional graph-name component, the RDF 1.1 dataset extension) and never
//! forks the term enum.
//!
//! ## DoS / complexity cap
//!
//! RDFC-1.0's §4.8.3 recursion is super-polynomial in the worst case, and
//! the spec's §"Dataset Poisoning" warns of adversarial datasets crafted
//! to never terminate in reasonable time. Per the spec's mandated
//! mitigation ("a configurable limit on the number of iterations of steps
//! performed in the algorithm, particularly recursive steps and
//! permutations of long lists") this implementation carries a typed
//! [`CanonLimits`] cap and returns [`CanonError::ComplexityCapExceeded`]
//! — never hangs, never `unwrap`s — on a poison graph. The official test
//! suite's `test074` (a 10-node blank-node clique) is precisely such a
//! graph and is expected to *error*.
//!
//! ## Scope
//!
//! This module is the conformant algorithm + its public API. It is **not**
//! wired into the OWL canonical form / `canonical_signatures` here; doing
//! so would change OWL graph identity and is a deliberate, separate
//! follow-up.
//!
//! `no_std` + `alloc`, wasm32-clean.
//!
//! [REC-rdf-canon-20240521]: https://www.w3.org/TR/rdf-canon/

use alloc::{collections::BTreeMap, string::String};

pub mod algorithm;
pub mod nquads;

#[cfg(test)]
mod tests;

pub use algorithm::HashAlgorithm;
pub use nquads::{Quad, parse_nquads};

/// A typed error from any stage of canonicalization. No path through the
/// algorithm panics or unwraps on adversarial input — malformed N-Quads,
/// an internal invariant breach, or a DoS-cap trip all surface here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    /// The input N-Quads document was malformed at the described point.
    Parse(String),
    /// An internal invariant did not hold (e.g. a blank node reached
    /// serialization without a canonical identifier). Indicates a bug, not
    /// bad input; surfaced rather than panicked so a host never crashes.
    Internal(String),
    /// The DoS / complexity cap was exceeded — a *poison* dataset
    /// (REC §"Dataset Poisoning"). `what` names the bounded quantity and
    /// `limit` the configured ceiling. The canonicalization is therefore a
    /// *partial* function on this input (REC §3 terminology), exactly as
    /// the spec permits for inputs that "prevent this algorithm from
    /// terminating in a reasonable amount of time".
    ComplexityCapExceeded {
        /// Human-readable name of the work that hit the ceiling.
        what: &'static str,
        /// The configured ceiling.
        limit: u64,
    },
}

impl core::fmt::Display for CanonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CanonError::Parse(m) => write!(f, "N-Quads parse error: {m}"),
            CanonError::Internal(m) => write!(f, "internal canonicalization error: {m}"),
            CanonError::ComplexityCapExceeded { what, limit } => write!(
                f,
                "RDFC-1.0 complexity cap exceeded ({what} > {limit}); \
                 input is likely a poison dataset"
            ),
        }
    }
}

/// The DoS / complexity budget for one canonicalization run
/// (REC §"Dataset Poisoning").
///
/// Two independent ceilings bound the §4.8.3 worst case:
///
/// - `max_hndq_calls` — the total number of Hash N-Degree Quads
///   invocations across the whole run (including recursive ones). For a
///   well-formed dataset this is small — the spec notes "more than a
///   couple of iterations on Hash N-Degree Quads per blank node would be
///   unusual". A poison/clique graph drives it unbounded, so this is the
///   primary stop.
/// - `max_permutations` — the size of any one permutation set of a related
///   blank-node list (hndq.5.4). A list of length k yields k! permutations;
///   this refuses *before* materializing the list when k! would exceed the
///   ceiling, so a symmetric clique (the suite's `test074`, where one
///   blank node relates to nine others under a shared hash) errors instead
///   of enumerating 9! orderings.
#[derive(Debug, Clone, Copy)]
pub struct CanonLimits {
    /// Maximum total Hash N-Degree Quads invocations.
    pub max_hndq_calls: u64,
    /// Maximum permutations of a single related blank-node list.
    pub max_permutations: u64,
}

impl Default for CanonLimits {
    /// Defaults sized to admit every *computable* suite fixture — including
    /// the deliberately expensive `test044`/`045`/`046` poison graphs the
    /// manifest marks "computable given defined limits" (complexity 39) —
    /// while still rejecting the `test074` clique (complexity 40) that is
    /// designed to be non-terminating.
    ///
    /// - `max_hndq_calls = 4_000_000`: comfortably above what the
    ///   computable poison fixtures need, far below the clique's blow-up.
    /// - `max_permutations = 40_320` = 8!. A related list of 9+ symmetric
    ///   blank nodes (the clique) needs 9! = 362_880 permutations and is
    ///   refused; the computable fixtures never form a symmetric list that
    ///   large.
    fn default() -> Self {
        Self {
            max_hndq_calls: 4_000_000,
            max_permutations: 40_320,
        }
    }
}

/// Canonicalize an in-memory dataset of [`Quad`]s with the default
/// [`CanonLimits`] and SHA-256, returning the *serialized canonical form*
/// (sorted canonical N-Quads, REC ca.7).
pub fn canonicalize(quads: &[Quad]) -> Result<String, CanonError> {
    let (output, _map) =
        algorithm::canonicalize(quads, CanonLimits::default(), HashAlgorithm::Sha256)?;
    Ok(output)
}

/// Canonicalize an N-Quads document string (parse → canonicalize) with the
/// default limits and SHA-256. Returns the serialized canonical form.
pub fn canonicalize_nquads(input: &str) -> Result<String, CanonError> {
    let quads = parse_nquads(input)?;
    canonicalize(&quads)
}

/// Full-control entry point: choose the hash algorithm and the DoS limits,
/// and receive both the serialized canonical form **and** the
/// *issued identifiers map* (input blank-node label → canonical label,
/// REC ca.6) that the suite's `*-rdfc10map.json` fixtures check.
pub fn canonicalize_with(
    quads: &[Quad],
    limits: CanonLimits,
    algorithm: HashAlgorithm,
) -> Result<(String, BTreeMap<String, String>), CanonError> {
    algorithm::canonicalize(quads, limits, algorithm)
}
