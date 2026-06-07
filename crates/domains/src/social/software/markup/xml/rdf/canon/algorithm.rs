//! The four RDFC-1.0 sub-algorithms, implemented exactly per the numbered
//! steps of the W3C *RDF Dataset Canonicalization* Recommendation
//! (REC-rdf-canon-20240521 — the URDNA2015 successor). Step ids in the
//! comments (`ca.N`, `h1dq.N`, `iia.N`, `hrbn.N`, `hndq.N`) are the spec's
//! own `<li id="…">` markers.
//!
//! - **§4.4.3 Canonicalization Algorithm** — [`canonicalize`].
//! - **§4.5 Issue Identifier Algorithm** — `IdentifierIssuer::issue`.
//! - **§4.6.3 Hash First Degree Quads** — `hash_first_degree_quads`.
//! - **§4.7 Hash Related Blank Node** — `hash_related_blank_node`.
//! - **§4.8.3 Hash N-Degree Quads** — `hash_n_degree_quads` (the
//!   recursive gossip-path walk over permutations of the related
//!   blank-node list — the subtle, super-polynomial part the DoS cap
//!   defends).
//!
//! Hash function: SHA-256 (REC default, [FIPS-180-4]) via `sha2::Sha256`,
//! with the SHA-384 variant the suite's `test075` exercises. Selected by
//! [`HashAlgorithm`].
//!
//! `no_std` + `alloc`; `BTreeMap`/`BTreeSet` give the deterministic
//! code-point ordering the spec's "code point ordered" steps require
//! (UTF-8 byte order == Unicode code point order, REC §2). No `unwrap` /
//! `expect` / `panic` on adversarial input — every fallible step returns
//! [`CanonError`].

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use sha2::{Digest, Sha256, Sha384};

use super::super::term::RdfTerm;
use super::nquads::{Quad, serialize_iri, serialize_quad, serialize_term};
use super::{CanonError, CanonLimits};

/// The cryptographic hash backing a canonicalization run. SHA-256 is the
/// RDFC-1.0 default (REC §"Overview", [FIPS-180-4]); SHA-384 is the
/// alternative the suite's `test075` selects via `rdfc:hashAlgorithm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 — the RDFC-1.0 default.
    Sha256,
    /// SHA-384 — the suite's documented alternative.
    Sha384,
}

impl HashAlgorithm {
    /// The lower-case hex digest of `bytes`. REC §"Overview": *the hash of
    /// a string s is the lower-case, hexadecimal representation of the
    /// result of passing s through a cryptographic hash function.*
    fn hex(self, bytes: &[u8]) -> String {
        match self {
            HashAlgorithm::Sha256 => {
                let mut h = Sha256::new();
                h.update(bytes);
                hex_lower(&h.finalize())
            }
            HashAlgorithm::Sha384 => {
                let mut h = Sha384::new();
                h.update(bytes);
                hex_lower(&h.finalize())
            }
        }
    }
}

/// Lower-case hex without pulling a formatter per byte.
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0F) as usize] as char);
    }
    s
}

/// **§4.5 Issue Identifier Algorithm** state — an *identifier issuer*
/// (REC §"Blank Node Identifier Issuer State"): a prefix, a monotonically
/// increasing counter, and the *issued identifiers map* recording, in
/// issuance order, the identifier minted for each existing label.
#[derive(Debug, Clone)]
struct IdentifierIssuer {
    prefix: &'static str,
    counter: u64,
    /// existing label → issued identifier.
    issued: BTreeMap<String, String>,
    /// Issuance order of existing labels (ca.5.3 needs "the same order").
    order: Vec<String>,
}

impl IdentifierIssuer {
    fn new(prefix: &'static str) -> Self {
        Self {
            prefix,
            counter: 0,
            issued: BTreeMap::new(),
            order: Vec::new(),
        }
    }

    /// **§4.5** Issue (or recall) the canonical/temporary identifier for
    /// `existing`.
    fn issue(&mut self, existing: &str) -> String {
        // iia.1: return any already-issued identifier.
        if let Some(id) = self.issued.get(existing) {
            return id.clone();
        }
        // iia.2: prefix ++ counter.
        let issued = format!("{}{}", self.prefix, self.counter);
        // iia.3 / iia.4: record and increment.
        self.issued.insert(existing.to_string(), issued.clone());
        self.order.push(existing.to_string());
        self.counter += 1;
        // iia.5.
        issued
    }

    /// Whether `existing` already has an issued identifier (hndq.5.4.4.2.1).
    fn has(&self, existing: &str) -> bool {
        self.issued.contains_key(existing)
    }

    fn get(&self, existing: &str) -> Option<&String> {
        self.issued.get(existing)
    }
}

/// The **§4.3 canonicalization state**: the input dataset plus the
/// blank-node-to-quads map and the canonical issuer. Built once per
/// [`canonicalize`] call; the N-degree algorithm reads it by shared
/// reference and mutates only the canonical issuer through the owning
/// [`Canonicalizer`].
struct Canonicalizer<'a> {
    quads: &'a [Quad],
    /// **blank node to quads map** (REC §4.3): blank-node label → indices
    /// into `quads` of every quad mentioning it.
    bn_to_quads: BTreeMap<String, Vec<usize>>,
    /// **canonical issuer** (prefix `c14n`).
    canonical: IdentifierIssuer,
    limits: CanonLimits,
    /// Running tally of Hash N-Degree Quads invocations across the whole
    /// run — the DoS budget (see [`CanonLimits`]).
    hndq_calls: u64,
}

/// Result of one Hash N-Degree Quads call: the hash plus the issuer that
/// distributed the temporary identifiers along the chosen gossip path
/// (REC §4.8.3, step 6 returns both).
struct NDegreeResult {
    hash: String,
    issuer: IdentifierIssuer,
}

/// **§4.4.3 Canonicalization Algorithm** — the entry point.
///
/// Returns the *serialized canonical form* (canonical N-Quads, REC ca.7)
/// and the *issued identifiers map* (input label → canonical label, ca.6)
/// the `*-rdfc10map.json` suite fixtures check.
pub fn canonicalize(
    quads: &[Quad],
    limits: CanonLimits,
    algorithm: HashAlgorithm,
) -> Result<(String, BTreeMap<String, String>), CanonError> {
    // ca.1: create the canonicalization state.
    let mut state = Canonicalizer {
        quads,
        bn_to_quads: BTreeMap::new(),
        canonical: IdentifierIssuer::new("c14n"),
        limits,
        hndq_calls: 0,
    };

    // ca.2: for every quad Q, for each blank-node component, record Q
    // under that blank node's input label.
    for (i, quad) in quads.iter().enumerate() {
        for label in blank_components(quad) {
            let entry = state.bn_to_quads.entry(label).or_default();
            // The map relates a blank node to the *set* of quads it is a
            // component of; the suite contains duplicate input lines
            // (test076/077) that collapse to one quad — guard against
            // recording the same quad index twice for one blank node.
            if !entry.contains(&i) {
                entry.push(i);
            }
        }
    }

    // ca.3: first-degree hash of every blank node → hash to blank nodes map.
    let mut hash_to_bns: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let labels: Vec<String> = state.bn_to_quads.keys().cloned().collect();
    for n in &labels {
        let hf = hash_first_degree_quads(&state, n, algorithm)?;
        hash_to_bns.entry(hf).or_default().push(n.clone());
    }

    // ca.4: blank nodes with a UNIQUE first-degree hash get a canonical id
    // immediately, in code-point hash order. `BTreeMap` iterates keys in
    // code-point order already. Removed entries are tracked so ca.5 skips
    // them.
    let mut remaining: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (hash, identifier_list) in &hash_to_bns {
        if identifier_list.len() > 1 {
            // ca.4.1: more than one entry — defer to ca.5.
            remaining.insert(hash.clone(), identifier_list.clone());
            continue;
        }
        // ca.4.2: issue for the single blank node. ca.4.3 (removal) is
        // realized by simply not carrying it into `remaining`.
        state.canonical.issue(&identifier_list[0]);
    }

    // ca.5: blank nodes sharing a first-degree hash, in code-point hash
    // order (BTreeMap::values iterates in key order).
    for identifier_list in remaining.values() {
        // ca.5.1: hash path list.
        let mut hash_path_list: Vec<NDegreeResult> = Vec::new();
        // ca.5.2: for each blank node n in the identifier list.
        for n in identifier_list {
            // ca.5.2.1: skip those already issued a canonical identifier.
            if state.canonical.has(n) {
                continue;
            }
            // ca.5.2.2: temporary issuer, prefix "b".
            let mut temporary = IdentifierIssuer::new("b");
            // ca.5.2.3: issue b_n to n.
            temporary.issue(n);
            // ca.5.2.4: run Hash N-Degree Quads, append the result.
            let result = hash_n_degree_quads(&mut state, n, &mut temporary, algorithm)?;
            hash_path_list.push(result);
        }

        // ca.5.3: process the hash path list in code-point order of the
        // returned N-degree hash; issue canonical identifiers to the
        // temporary identifiers in each result, in their issuance order.
        hash_path_list.sort_by(|a, b| a.hash.cmp(&b.hash));
        for result in &hash_path_list {
            // ca.5.3.1: for each existing identifier that was issued a
            // temporary identifier in `result`, in the same order.
            for existing in &result.issuer.order {
                state.canonical.issue(existing);
            }
        }
    }

    // ca.6: the issued identifiers map (input label → canonical label).
    let issued_map = state.canonical.issued.clone();

    // ca.7: serialized canonical form — replace blank-node labels with
    // their canonical identifiers, serialize each quad, sort the lines in
    // code-point order, concatenate.
    let mut lines: Vec<String> = Vec::with_capacity(quads.len());
    for quad in quads {
        let relabeled = relabel_quad(quad, &issued_map)?;
        lines.push(serialize_quad(&relabeled));
    }
    lines.sort();
    // De-duplicate: a dataset is a *set* of quads; two input lines that
    // are syntactically distinct but canonicalize identically (e.g. the
    // duplicate-triple tests test076/077) collapse to one line.
    lines.dedup();
    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
    }

    Ok((output, issued_map))
}

/// The blank-node *labels* that are components of `quad` (subject, object,
/// graph; predicate is always an IRI). RDFC §4.4.3 step 2 treats a blank
/// graph name as a blank-node component.
fn blank_components(quad: &Quad) -> Vec<String> {
    let mut out = Vec::new();
    if let RdfTerm::Blank(l) = quad.subject() {
        out.push(l.clone());
    }
    if let RdfTerm::Blank(l) = quad.object() {
        out.push(l.clone());
    }
    if let Some(RdfTerm::Blank(l)) = quad.graph() {
        out.push(l.clone());
    }
    out
}

/// Rewrite every blank-node component of `quad` to its canonical
/// identifier (REC ca.7 / §"Serialized canonical form"). Every blank node
/// in the input MUST have received a canonical identifier; a missing one
/// is an internal invariant violation surfaced as an error rather than a
/// panic.
fn relabel_quad(quad: &Quad, issued: &BTreeMap<String, String>) -> Result<Quad, CanonError> {
    let relabel = |t: &RdfTerm| -> Result<RdfTerm, CanonError> {
        match t {
            RdfTerm::Blank(label) => {
                let canon = issued.get(label).ok_or_else(|| {
                    CanonError::Internal(format!("no canonical identifier for blank node {label}"))
                })?;
                Ok(RdfTerm::Blank(canon.clone()))
            }
            other => Ok(other.clone()),
        }
    };
    let subject = relabel(quad.subject())?;
    let object = relabel(quad.object())?;
    let graph = match quad.graph() {
        Some(g) => Some(relabel(g)?),
        None => None,
    };
    Ok(Quad::in_graph(
        super::super::term::Triple {
            subject,
            predicate: quad.predicate().to_string(),
            object,
        },
        graph,
    ))
}

/// **§4.6.3 Hash First Degree Quads** — the first-degree hash of the blank
/// node `reference`.
///
/// h1dq.1..h1dq.5: serialize every quad mentioning `reference` in
/// canonical N-Quads form, but with each blank-node component replaced by
/// the marker `a` (if it *is* `reference`) or `z` (otherwise); sort the
/// resulting lines in code-point order; hash the concatenation.
fn hash_first_degree_quads(
    state: &Canonicalizer<'_>,
    reference: &str,
    algorithm: HashAlgorithm,
) -> Result<String, CanonError> {
    // h1dq.1 / h1dq.2.
    let mut nquads: Vec<String> = Vec::new();
    let quad_indices = match state.bn_to_quads.get(reference) {
        Some(v) => v,
        None => &Vec::new(),
    };
    // h1dq.3: serialize each quad with the a/z special rule.
    for &i in quad_indices {
        let quad = &state.quads[i];
        let line = serialize_quad_with_markers(quad, reference);
        nquads.push(line);
    }
    // h1dq.4.
    nquads.sort();
    // h1dq.5.
    let mut buf = String::new();
    for line in &nquads {
        buf.push_str(line);
    }
    Ok(algorithm.hex(buf.as_bytes()))
}

/// Serialize `quad` for Hash First Degree Quads (h1dq.3.1): blank-node
/// components become `_:a` (the reference) or `_:z` (any other blank node);
/// IRIs and literals serialize normally.
fn serialize_quad_with_markers(quad: &Quad, reference: &str) -> String {
    let marker = |t: &RdfTerm| -> String {
        match t {
            RdfTerm::Blank(label) => {
                if label == reference {
                    "_:a".to_string()
                } else {
                    "_:z".to_string()
                }
            }
            other => serialize_term(other),
        }
    };
    let mut out = String::new();
    out.push_str(&marker(quad.subject()));
    out.push(' ');
    out.push_str(&serialize_iri(quad.predicate()));
    out.push(' ');
    out.push_str(&marker(quad.object()));
    out.push(' ');
    if let Some(g) = quad.graph() {
        out.push_str(&marker(g));
        out.push(' ');
    }
    out.push('.');
    out.push('\n');
    out
}

/// **§4.7 Hash Related Blank Node** — a hash for how `related` sits in
/// `quad` relative to the blank node currently being processed.
///
/// hrbn.1..hrbn.5: `input = position [ "<" predicate ">" if position != g ]
/// [ "_:"+id if known else hash_first_degree_quads(related) ]`, then hash.
fn hash_related_blank_node(
    state: &Canonicalizer<'_>,
    related: &str,
    quad: &Quad,
    issuer: &IdentifierIssuer,
    position: char,
    algorithm: HashAlgorithm,
) -> Result<String, CanonError> {
    // hrbn.1.
    let mut input = String::new();
    input.push(position);
    // hrbn.2.
    if position != 'g' {
        input.push('<');
        input.push_str(quad.predicate());
        input.push('>');
    }
    // hrbn.3 / hrbn.4: canonical id, else temporary id, else 1st-degree hash.
    if let Some(canon) = state.canonical.get(related) {
        input.push_str("_:");
        input.push_str(canon);
    } else if let Some(temp) = issuer.get(related) {
        input.push_str("_:");
        input.push_str(temp);
    } else {
        input.push_str(&hash_first_degree_quads(state, related, algorithm)?);
    }
    // hrbn.5.
    Ok(algorithm.hex(input.as_bytes()))
}

/// **§4.8.3 Hash N-Degree Quads** — the recursive gossip-path hash.
///
/// Follows the spec steps hndq.1..hndq.6 verbatim, including the
/// permutation loop (hndq.5.4) over the related blank-node list, the
/// early-exit comparisons (hndq.5.4.4.3 / hndq.5.4.5.5), the recursion
/// (hndq.5.4.5.1), and the chosen-path / chosen-issuer selection
/// (hndq.5.4.6 / hndq.5.6).
///
/// **DoS cap (REC §"Dataset Poisoning"):** every entry counts one unit of
/// the global Hash-N-Degree-Quads budget, and every permutation set is
/// length-capped, so an adversarial *poison*/clique dataset terminates
/// with [`CanonError::ComplexityCapExceeded`] instead of running for
/// super-polynomial time — exactly the mitigation the spec mandates
/// ("a configurable limit on the number of iterations … particularly
/// recursive steps and permutations of long lists").
fn hash_n_degree_quads(
    state: &mut Canonicalizer<'_>,
    identifier: &str,
    issuer: &mut IdentifierIssuer,
    algorithm: HashAlgorithm,
) -> Result<NDegreeResult, CanonError> {
    // DoS budget: bound the total number of N-degree invocations.
    state.hndq_calls += 1;
    if state.hndq_calls > state.limits.max_hndq_calls {
        return Err(CanonError::ComplexityCapExceeded {
            what: "Hash N-Degree Quads invocations",
            limit: state.limits.max_hndq_calls,
        });
    }

    // hndq.1: Hn — related hash → related blank-node labels.
    let mut hn: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // hndq.2: the quads mentioning `identifier`.
    let quad_indices = state
        .bn_to_quads
        .get(identifier)
        .cloned()
        .unwrap_or_default();

    // hndq.3: for each quad, for each blank-node component != identifier,
    // add its related hash → label to Hn.
    for &i in &quad_indices {
        let quad = state.quads[i].clone();
        // subject (position 's'), object ('o'), graph name ('g').
        let components: [(Option<&RdfTerm>, char); 3] = [
            (Some(quad.subject()), 's'),
            (Some(quad.object()), 'o'),
            (quad.graph(), 'g'),
        ];
        for (comp, position) in components {
            if let Some(RdfTerm::Blank(label)) = comp {
                if label == identifier {
                    continue;
                }
                // hndq.3.1.1.
                let hash =
                    hash_related_blank_node(state, label, &quad, issuer, position, algorithm)?;
                // hndq.3.1.2.
                hn.entry(hash).or_default().push(label.clone());
            }
        }
    }

    // hndq.4.
    let mut data_to_hash = String::new();

    // hndq.5: for each related hash → blank-node list, code-point ordered.
    for (related_hash, blank_node_list) in &hn {
        // hndq.5.1.
        data_to_hash.push_str(related_hash);
        // hndq.5.2 / hndq.5.3.
        let mut chosen_path = String::new();
        let mut chosen_issuer: Option<IdentifierIssuer> = None;

        // hndq.5.4: for each permutation p of the blank-node list. The
        // permutation count is k! for a list of length k; cap it so a
        // clique (the suite's test074) cannot detonate here.
        let perms = permutations(blank_node_list, state.limits.max_permutations)?;
        for p in perms {
            // hndq.5.4.1: copy of issuer.
            let mut issuer_copy = issuer.clone();
            // hndq.5.4.2 / hndq.5.4.3.
            let mut path = String::new();
            let mut recursion_list: Vec<String> = Vec::new();

            // hndq.5.4.4: for each related in p.
            let mut skip_permutation = false;
            for related in &p {
                if let Some(canon) = state.canonical.get(related) {
                    // hndq.5.4.4.1: canonical id known.
                    path.push_str("_:");
                    path.push_str(canon);
                } else {
                    // hndq.5.4.4.2.1: not yet issued by issuer_copy → recurse later.
                    if !issuer_copy.has(related) {
                        recursion_list.push(related.clone());
                    }
                    // hndq.5.4.4.2.2: issue a temporary id, append it.
                    let temp = issuer_copy.issue(related);
                    path.push_str("_:");
                    path.push_str(&temp);
                }
                // hndq.5.4.4.3: early-exit if path already worse than chosen.
                if !chosen_path.is_empty()
                    && path.len() >= chosen_path.len()
                    && path.as_str() > chosen_path.as_str()
                {
                    skip_permutation = true;
                    break;
                }
            }
            if skip_permutation {
                continue;
            }

            // hndq.5.4.5: for each related in the recursion list, recurse.
            for related in &recursion_list {
                // hndq.5.4.5.1.
                let result = hash_n_degree_quads(state, related, &mut issuer_copy, algorithm)?;
                // hndq.5.4.5.2.
                let temp = issuer_copy.issue(related);
                path.push_str("_:");
                path.push_str(&temp);
                // hndq.5.4.5.3.
                path.push('<');
                path.push_str(&result.hash);
                path.push('>');
                // hndq.5.4.5.4.
                issuer_copy = result.issuer;
                // hndq.5.4.5.5.
                if !chosen_path.is_empty()
                    && path.len() >= chosen_path.len()
                    && path.as_str() > chosen_path.as_str()
                {
                    skip_permutation = true;
                    break;
                }
            }
            if skip_permutation {
                continue;
            }

            // hndq.5.4.6: keep the shortest / code-point-least path.
            if chosen_path.is_empty() || path.as_str() < chosen_path.as_str() {
                chosen_path = path;
                chosen_issuer = Some(issuer_copy);
            }
        }

        // hndq.5.5.
        data_to_hash.push_str(&chosen_path);
        // hndq.5.6: adopt the chosen issuer.
        if let Some(ci) = chosen_issuer {
            *issuer = ci;
        }
    }

    // hndq.6.
    Ok(NDegreeResult {
        hash: algorithm.hex(data_to_hash.as_bytes()),
        issuer: issuer.clone(),
    })
}

/// All permutations of `items`, in a deterministic order, capped at
/// `max_permutations` (the DoS guard for hndq.5.4). For k items the count
/// is k!; the moment the *factorial* of the list length would exceed the
/// cap we refuse rather than materialize the list — so the clique poison
/// graph (test074) errors here instead of allocating 9!·… permutations.
///
/// The order matches a recursive lexicographic expansion of the input
/// order, which (since the input list order does not affect the *chosen*
/// path — the spec selects the code-point-least path across *all*
/// permutations) is sufficient for conformance and deterministic.
fn permutations(items: &[String], max_permutations: u64) -> Result<Vec<Vec<String>>, CanonError> {
    let k = items.len();
    // Refuse before building if k! would exceed the cap. factorial()
    // saturates, so an enormous k is caught immediately.
    let count = factorial_saturating(k as u64, max_permutations);
    if count > max_permutations {
        return Err(CanonError::ComplexityCapExceeded {
            what: "permutations of a related blank-node list",
            limit: max_permutations,
        });
    }
    let mut out = Vec::new();
    let mut current = Vec::with_capacity(k);
    let mut used = vec![false; k];
    permute_rec(items, &mut used, &mut current, &mut out);
    Ok(out)
}

/// Heap-free recursive permutation enumeration. `out` collects each full
/// permutation; `used` marks consumed indices.
fn permute_rec(
    items: &[String],
    used: &mut [bool],
    current: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    if current.len() == items.len() {
        out.push(current.clone());
        return;
    }
    for i in 0..items.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        current.push(items[i].clone());
        permute_rec(items, used, current, out);
        current.pop();
        used[i] = false;
    }
}

/// `n!`, saturating at `cap.saturating_add(1)` so the caller can compare
/// against `cap` without overflow even for large `n`.
fn factorial_saturating(n: u64, cap: u64) -> u64 {
    let ceiling = cap.saturating_add(1);
    let mut acc: u64 = 1;
    let mut i = 2;
    while i <= n {
        acc = acc.saturating_mul(i);
        if acc > ceiling {
            return ceiling;
        }
        i += 1;
    }
    acc
}
