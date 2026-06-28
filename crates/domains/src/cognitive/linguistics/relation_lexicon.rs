//! The relation lexicon — the loaded surface→relation map (today: `"part of"` ↦
//! Parthood), carried AS `.prx` DATA. Subsumption ("is a") is the copula DEFAULT
//! and is deliberately NOT lexicalized here (see the body note below).
//!
//! A relational question ("is X part of Y") must name WHICH relation it asserts.
//! That surface→relation mapping is ontological vocabulary, so it is LOADED, not
//! a Rust `match predicate { "part of" => … }` (Rule 11/12). It rides the §9
//! lexicalization channel verbatim: each relation is a `Concept` node whose
//! `ontolex:Form` atoms (`writtenRep`) are the natural surfaces a person types;
//! [`relation_surface_index`] indexes those surfaces to the relation kind exactly
//! as the composed reasoner indexes a USC section's heading to the section.
//!
//! The concept node NAMES (`Parthood`, `Subsumption`) are the wire strings that
//! cross to a typed [`ConceptRef`] exactly once, through the blessed
//! [`relations_kind`] lowering — so a surface resolves to the SAME relation kind
//! the materialized closure keys on.
//!
//! The map LIVES in the committed `data/projections/relation_lexicon.prx`, loaded
//! fail-closed against its baked Merkle root — never a Rust literal (the pin IS
//! the integrity, the [`english_functor`](super::english::bridge) `.prx`
//! convention).
//!
//! The lexicon carries only NON-DEFAULT, complement-headed relational surfaces.
//! Subsumption is the copula DEFAULT — "is X a Y" is resolved by the grammar (the
//! copula "is" → the Subsumption fallback at dispatch), so it needs no lexicon
//! entry, and listing "is a" here would make the multi-word recognizer wrongly
//! COLLAPSE the copula+determiner sequence. So the lexicon holds exactly the
//! relations a person names with a phrasal head the base grammar can't parse —
//! today: Parthood ("part of"). (Inverse surfaces like "has part" await an
//! inverse-relation kind in the closure; omitted until then.)
//!
//! Authority (every surface cited):
//! - `"part of"` ↦ Parthood — OBO Relation Ontology / BFO `part of`
//!   (`BFO:0000050`, `rdfs:label "part of"`, `owl:TransitiveProperty`); Smith et
//!   al. (2005) *Genome Biology* 6:R46 (DOI:10.1186/gb-2005-6-5-r46) — the
//!   all-some `part_of` schema, distinct from `is_a`; Casati & Varzi (1999).
//! - The surface↔concept link is the OntoLex-Lemon denotation floor (W3C 2016
//!   Lexicon Model for Ontologies): a relation is a valid reference target.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;

use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::ontology::{ConceptRef, relations_kind};

use crate::cognitive::linguistics::english::bridge::FORM_KIND;

/// The default relation a relational question asserts when its surface names no
/// other — the is-a kind (`rdfs:subClassOf`). Re-exported here, the relation
/// vocabulary's home, so the chat dispatch can resolve the fallback without a
/// direct `pr4xis-runtime` dependency (it reasons over relation kinds only
/// through this module + [`LexicalReasoner`](super::english::LexicalReasoner)).
pub use pr4xis_runtime::ontology::subsumption_kind;

/// The committed relation lexicon — the `.prx` bytes the surface→relation map
/// LIVES in, embedded at build time. A node-bearing [`Archive`]: relation
/// `Concept` nodes (today exactly one, `Parthood`) each pointing at the
/// `ontolex:Form` surface atoms (today `"part of"`) a person types. Subsumption
/// is the un-lexicalized copula default, so it has no node here.
const RELATION_LEXICON_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/projections/relation_lexicon.prx"
));

/// The trusted Merkle root of [`RELATION_LEXICON_PRX`] — the integrity pin the
/// fail-closed load checks against (file ⇔ pin coherence is asserted in tests).
const RELATION_LEXICON_ROOT_HEX: &str =
    "4f8edf9c29ef21aa0856bb26f2a4a8b120f444eaf72dec7ddc373fdac707f72c";

/// Load the relation lexicon from its committed `.prx` ([`RELATION_LEXICON_PRX`])
/// — FAIL-CLOSED: the embedded bytes are admitted only if they re-derive to
/// [`RELATION_LEXICON_ROOT_HEX`], so a tampered or stale lexicon is refused.
/// Reuses the kernel [`load`](pr4xis_runtime::load::load); no new runtime API. A
/// load failure here is a build-time invariant violation (the bytes ship embedded
/// in the binary), exactly like the `english_functor.prx` load.
fn relation_lexicon() -> Archive {
    let root = ContentAddress::from_hex(RELATION_LEXICON_ROOT_HEX)
        .expect("RELATION_LEXICON_ROOT_HEX is valid 64-hex");
    pr4xis_runtime::load::load(RELATION_LEXICON_PRX, root)
        .expect("committed relation_lexicon.prx must load against its baked root")
}

/// The loaded surface→relation-kind index: today `"part of"` → the Parthood
/// [`ConceptRef`] (and any future non-default phrasal relations). It does NOT map
/// `"is a"` — Subsumption is the copula default resolved at dispatch, so
/// `relation_surface_index().get("is a")` is `None`. Built by
/// walking the committed lexicon archive the SAME way the composed reasoner walks
/// a loaded corpus (§9): a relation `Concept`'s queryable surfaces are the
/// `ontolex:Form` atoms it points at, detected by FORM-target-ness (a data
/// property of the archive), never a hardcoded role allow-list.
///
/// Each concept node's NAME crosses to its typed relation kind exactly once,
/// through [`relations_kind`] — so the value is the SAME kind the materialized
/// closure keys on (`reaches(.., relations_kind("Parthood"))`).
pub fn relation_surface_index() -> BTreeMap<String, ConceptRef> {
    let archive = relation_lexicon();
    // The `ontolex:Form` atoms — their NAMES are the natural-language surfaces.
    let form_names: BTreeSet<&str> = archive
        .nodes
        .iter()
        .filter(|n| n.kind == FORM_KIND)
        .map(|n| n.name.as_str())
        .collect();

    let mut index = BTreeMap::new();
    for node in &archive.nodes {
        // A Form atom is a surface, not a relation concept.
        if node.kind == FORM_KIND {
            continue;
        }
        // The blessed wire→kind crossing: the concept name ("Parthood") lowers to
        // its relation kind in the one Relations vocab the closure is keyed on.
        let kind = relations_kind(node.name.clone());
        for (_role, target) in &node.edges {
            if let Some(form) = target.local_name()
                && form_names.contains(form)
            {
                index.insert(form.to_lowercase(), kind.clone());
            }
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_lexicon_maps_part_of_to_parthood_fail_closed() {
        // The loaded map resolves the phrasal relational surface to the SAME
        // ConceptRef the materialized closure keys on — "part of" → Parthood.
        let index = relation_surface_index();
        assert_eq!(index.get("part of"), Some(&relations_kind("Parthood")));
        // Subsumption is the copula default, NOT lexicalized here (else the
        // multi-word recognizer would wrongly collapse "is a").
        assert_eq!(index.get("is a"), None);
        assert_ne!(
            index.get("part of"),
            Some(&relations_kind("Subsumption")),
            "part of is Parthood, distinct from Subsumption (Smith et al. 2005)"
        );

        // File ⇔ pin coherence + fail-closed: the committed bytes re-derive to the
        // baked root, and a WRONG root is refused.
        let pin = ContentAddress::from_hex(RELATION_LEXICON_ROOT_HEX).unwrap();
        assert_eq!(
            pr4xis_runtime::load::load(RELATION_LEXICON_PRX, pin)
                .unwrap()
                .root()
                .unwrap(),
            pin,
            "the committed .prx re-derives to its baked root"
        );
        assert!(
            pr4xis_runtime::load::load(RELATION_LEXICON_PRX, ContentAddress::of(b"wrong")).is_err(),
            "a wrong root is refused — the load is fail-closed"
        );
    }
}
