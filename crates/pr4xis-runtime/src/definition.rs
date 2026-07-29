//! Definition-bearing addressing — a node's identity is the content address of
//! its DEFINITION, not its name.
//!
//! This is the head of the build spine: it closes the **G5 wire gap**. Under
//! name-only addressing (`hash(kind, name)`) two nodes that share a name but
//! differ in structure collide to the same address, so peers could "agree" on a
//! node while meaning different things. Addressing the *definition* — kind +
//! name + outgoing edges + governing axioms + lexical grounding — makes
//! agreement mean agreement on what the node actually IS.
//!
//! References to other nodes (edge targets, axioms) are by NAME at this layer,
//! resolved within the node's own ontology and bound by the ontology's Merkle
//! root. The stronger *recursive* form — where this address depends on the
//! targets' own addresses — is the Merkle-DAG layer, which must additionally
//! handle cyclic graphs (e.g. symmetric `opposition`) via strongly-connected-
//! component hashing. This module is the cycle-safe floor that layer builds on.

use core::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::address::ContentAddress;
use crate::codec::{self, CodecError};

/// The endpoint of a typed edge.
///
/// Most edges are LOCAL: a name resolved within the node's OWN ontology, bound
/// by that ontology's Merkle root (the form every `.prx` has carried so far). A
/// GROUNDED edge is the foreign-atom slot — it crosses INTO another ontology,
/// naming an atom by its content [`ContentAddress`] in a declared connected
/// ontology. It is the typed cross-ontology morphism the grounding vocabulary
/// rides (lexical `denotes` and the other kinds): a span endpoint resolved by
/// content-address agreement across archives, not by name within one.
///
/// # Byte-exactness
///
/// `Local` serializes byte-IDENTICALLY to the bare string target it replaced, so
/// every node minted before grounding existed keeps its exact content address —
/// the codec migration is invisible to all-local archives (the committed pins
/// re-verify unchanged). `Grounded` serializes as a CBOR map, which the decoder
/// tells apart from a string by major type (`deserialize_any`); the common local
/// case spends no discriminator byte.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeTarget {
    /// A same-ontology target, by name — resolved within the node's ontology.
    Local(String),
    /// A cross-ontology target: an atom of `ontology` named by its content
    /// `atom` address (the foreign-atom slot; resolved by address agreement).
    Grounded {
        /// The connected ontology the atom lives in.
        ///
        /// This is the ARCHIVED WIRE form — a plain `String`, kept here (not lifted
        /// to `OntologyName`) as the codebase's single "strings are wire, lifted
        /// once" lowering boundary (the same doctrine `kind` follows): `OntologyName`
        /// is a `Cow<'static, str>` that does not cleanly derive `rkyv::Archive`, so
        /// forcing it into this rkyv-archived / DAG-CBOR-serialized wire type would
        /// add friction for no gain. It is LIFTED to the typed
        /// [`OntologyName`](pr4xis::ontology::meta::OntologyName) exactly once, at the
        /// in-memory `GroundedEdge` (see `ontology::GroundedEdge::ontology`).
        ontology: String,
        /// The content address of the foreign atom.
        atom: ContentAddress,
    },
}

impl EdgeTarget {
    /// The local target name, or `None` if this edge grounds into another
    /// ontology. Traversers of the LOCAL graph use this; it forces the foreign
    /// case to be handled explicitly, never silently read as a local name.
    pub fn local_name(&self) -> Option<&str> {
        match self {
            EdgeTarget::Local(name) => Some(name),
            EdgeTarget::Grounded { .. } => None,
        }
    }
}

impl From<String> for EdgeTarget {
    fn from(name: String) -> Self {
        EdgeTarget::Local(name)
    }
}

impl From<&str> for EdgeTarget {
    fn from(name: &str) -> Self {
        EdgeTarget::Local(name.to_string())
    }
}

impl Serialize for EdgeTarget {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            // Byte-identical to the pre-grounding bare-string target.
            EdgeTarget::Local(name) => serializer.serialize_str(name),
            EdgeTarget::Grounded { ontology, atom } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("atom", &atom.to_hex())?;
                map.serialize_entry("ontology", ontology)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for EdgeTarget {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EdgeTargetVisitor;

        impl<'de> Visitor<'de> for EdgeTargetVisitor {
            type Value = EdgeTarget;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a local target name (string) or a grounded {ontology, atom} map")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<EdgeTarget, E> {
                Ok(EdgeTarget::Local(v.to_string()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<EdgeTarget, E> {
                Ok(EdgeTarget::Local(v))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<EdgeTarget, A::Error> {
                let mut ontology: Option<String> = None;
                let mut atom_hex: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "ontology" => ontology = Some(map.next_value()?),
                        "atom" => atom_hex = Some(map.next_value()?),
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let ontology = ontology.ok_or_else(|| de::Error::missing_field("ontology"))?;
                let atom_hex = atom_hex.ok_or_else(|| de::Error::missing_field("atom"))?;
                let atom = ContentAddress::from_hex(&atom_hex).ok_or_else(|| {
                    de::Error::custom("grounded edge atom is not a valid content address")
                })?;
                Ok(EdgeTarget::Grounded { ontology, atom })
            }
        }

        deserializer.deserialize_any(EdgeTargetVisitor)
    }
}

/// The canonical definition of a node — the structure its content-address is
/// taken over. Field order is the serialized order; the `edges`/`axioms` rows
/// are sorted + deduplicated by [`Definition::address`] before encoding, so the
/// address does not depend on assembly order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    /// The node's kind — a name resolved against the meta-ontology
    /// (e.g. `"Concept"`, `"Functor"`, `"Lens"`).
    pub kind: String,
    /// The node's name within its ontology.
    pub name: String,
    /// Outgoing typed edges, each `(relation-kind name, target)`. The target is
    /// usually [`EdgeTarget::Local`] (a name in this ontology); a
    /// [`EdgeTarget::Grounded`] target crosses into a connected ontology by
    /// content address. An all-local node's encoding — hence its address — is
    /// unchanged from when targets were bare strings.
    pub edges: Vec<(String, EdgeTarget)>,
    /// Names of the axioms that constrain this node.
    pub axioms: Vec<String>,
    /// The node's lexical grounding (canonical English form / Lemon entry), if
    /// any — part of the definition because in praxis "everything is Lemon".
    pub lexical: Option<String>,
}

impl Definition {
    /// The content address of this node — its definition-bearing identity.
    ///
    /// Canonical: the `edges` and `axioms` rows are sorted + deduplicated before
    /// the DAG-CBOR encoding, so two definitions with the same content assembled
    /// in different orders share the same address.
    pub fn address(&self) -> Result<ContentAddress, CodecError> {
        let mut canon = self.clone();
        canon.edges.sort();
        canon.edges.dedup();
        canon.axioms.sort();
        canon.axioms.dedup();
        codec::address_of(&canon)
    }
}

/// The praxis kind of a written-form atom — an `ontolex:Form` (its `writtenRep`),
/// carrying NO sense. A `Form` node is a bare surface: it grounds "this written
/// form occurred", never a meaning. It is the honest target a lexical surface
/// edge (`canonicalForm` / `otherForm` / `denotes`) points AT, and the surface
/// the composed reasoner indexes as queryable text distinct from a node's
/// identifier. The wire constant lives here (beside [`Definition`]) so every
/// producer — the English/OWL/USC bridges AND the compiled-ontology emitter —
/// agrees on the exact kind string the reasoner filters on.
pub const FORM_KIND: &str = "Form";

/// The Lemon `ontolex:canonicalForm` lexicalization role — the edge from a
/// concept to its one canonical written surface ([`FORM_KIND`] atom). Wire DATA.
pub const CANONICAL_FORM_REL: &str = "canonicalForm";

/// The `ontolex:Form` atom for a written representation — a bare surface-form
/// node carrying its `writtenRep` as both `name` and `lexical`, and NOTHING
/// else: no sense, no synset edge. Its content [`address`](Definition::address)
/// is what a lexical surface edge points AT.
///
/// Sense-deferral is STRUCTURAL: a Form has no senses, so a pointer that resolves
/// to one cannot have over-committed to a meaning (the written-form floor's
/// honesty tripwire — a surface edge must land on a `Form`, never a synset).
pub fn form_atom(written_rep: &str) -> Definition {
    Definition {
        kind: FORM_KIND.to_string(),
        name: written_rep.to_string(),
        edges: Vec::new(),
        axioms: Vec::new(),
        lexical: Some(written_rep.to_string()),
    }
}

/// The praxis kind of a CITED-SOURCE atom — a `dcterms:BibliographicResource`
/// ("a book, article, or other documentary resource", DCMI Metadata Terms
/// 2020-01-20). A `BibliographicResource` node is a bare citation: it grounds
/// "this document was cited", never a meaning and never a queryable natural
/// surface. It is the honest target a [`DEFINITION_SOURCE_REL`] edge points AT.
///
/// Sibling of [`FORM_KIND`] and here for the SAME reason: the wire constant must
/// be agreed by the producer (a lexicon bridge) and the consumer (the composed
/// reasoner, which filters this kind OUT of the concept universe exactly as it
/// filters `Form` out — a citation is not a concept a question can be about).
pub const SOURCE_KIND: &str = "BibliographicResource";

/// The DCMI `dcterms:source` role — the edge from a definition-bearing node to
/// the documentary resource its definition was DERIVED FROM ("a related resource
/// from which the described resource is derived", DCMI Metadata Terms
/// 2020-01-20). Wire DATA.
///
/// This is provenance OF THE DEFINITION, categorically distinct from the
/// ontologies a query reasoned OVER: "this gloss was authored from 42 USC
/// 300ii(7)" is a claim about where the words came from, not a claim that the
/// engine opened Title 42. Carrying it as an edge is what makes the two
/// tellable apart at all — a citation living inside the gloss STRING is
/// inspectable by nothing.
pub const DEFINITION_SOURCE_REL: &str = "source";

/// The `dcterms:BibliographicResource` atom for one citation — a bare
/// documentary-resource node carrying the citation as both `name` and `lexical`,
/// and NOTHING else: no edges, no axioms. Its content
/// [`address`](Definition::address) is what a [`DEFINITION_SOURCE_REL`] edge
/// points AT.
///
/// One atom per DISTINCT citation, shared by every node that cites it — so a
/// multi-source definition is N edges into the graph, never one delimited
/// string a reader would have to re-split.
pub fn source_atom(citation: &str) -> Definition {
    Definition {
        kind: SOURCE_KIND.to_string(),
        name: citation.to_string(),
        edges: Vec::new(),
        axioms: Vec::new(),
        lexical: Some(citation.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Definition {
        Definition {
            kind: "Concept".into(),
            name: "Employer".into(),
            edges: vec![("Subsumption".into(), "Agent".into())],
            axioms: vec!["EmployerIsAgent".into()],
            lexical: Some("employer".into()),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn identical_definitions_share_an_address() {
        assert_eq!(base().address().unwrap(), base().address().unwrap());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn changing_an_edge_changes_the_address() {
        let mut b = base();
        b.edges = vec![("Subsumption".into(), "Person".into())]; // different target
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn changing_an_axiom_changes_the_address() {
        let mut b = base();
        b.axioms = vec!["EmployerHiresEmployee".into()];
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn changing_the_lexical_changes_the_address() {
        let mut b = base();
        b.lexical = Some("boss".into());
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn same_name_different_definition_does_not_collide() {
        // The G5 fix: same name, different structure → different address.
        let mut b = base();
        b.edges.push(("Opposition".into(), "Employee".into()));
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn address_is_order_independent() {
        let mut a = base();
        a.edges = vec![
            ("Subsumption".into(), "Agent".into()),
            ("Opposition".into(), "Employee".into()),
        ];
        a.axioms = vec!["B".into(), "A".into()];
        let mut b = base();
        b.edges = vec![
            ("Opposition".into(), "Employee".into()),
            ("Subsumption".into(), "Agent".into()),
        ];
        b.axioms = vec!["A".into(), "B".into()];
        assert_eq!(a.address().unwrap(), b.address().unwrap());
    }

    // --- EdgeTarget: the byte-exact migration guarantee ---

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_local_target_encodes_byte_identically_to_a_bare_string() {
        // The crux of the codec migration: a Local target serializes to the SAME
        // canonical DAG-CBOR bytes as the bare string it replaced. So every node
        // minted before grounding existed keeps its exact content address.
        let local = EdgeTarget::Local("Agent".to_string());
        assert_eq!(
            codec::canonical_encode(&local).unwrap(),
            codec::canonical_encode(&"Agent".to_string()).unwrap(),
            "EdgeTarget::Local must encode as a bare CBOR string"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_all_local_definition_address_is_unchanged_by_the_edge_target_type() {
        // Concretely against the LEGACY shape: a node whose edges are Local
        // targets has the exact address it had when edges were (String, String).
        // This is the regression gate every committed pin relies on, proven at
        // the unit level (the corpus pin tests confirm it at scale).
        #[derive(serde::Serialize)]
        struct LegacyDefinition {
            kind: String,
            name: String,
            edges: Vec<(String, String)>,
            axioms: Vec<String>,
            lexical: Option<String>,
        }
        let legacy = LegacyDefinition {
            kind: "Concept".into(),
            name: "Employer".into(),
            edges: vec![("Subsumption".into(), "Agent".into())],
            axioms: vec!["EmployerIsAgent".into()],
            lexical: Some("employer".into()),
        };
        assert_eq!(
            base().address().unwrap(),
            codec::address_of(&legacy).unwrap(),
            "an all-Local Definition must address identically to the pre-migration shape"
        );
    }

    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn a_grounded_target_round_trips_and_is_distinct_from_a_local_one() {
        // A foreign-atom target encodes as a CBOR map (not a string), so it is
        // unambiguous on decode and never collides with a local name.
        let atom = ContentAddress::of(b"a connected ontology's atom definition");
        let grounded = EdgeTarget::Grounded {
            ontology: "english_wordnet".to_string(),
            atom,
        };
        let bytes = codec::canonical_encode(&grounded).unwrap();
        let back: EdgeTarget = codec::canonical_decode(&bytes).unwrap();
        assert_eq!(back, grounded, "a grounded target must round-trip");
        assert_ne!(
            bytes,
            codec::canonical_encode(&EdgeTarget::Local("english_wordnet".to_string())).unwrap(),
            "a grounded target must not encode like a local string of the same text"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_grounded_edge_changes_a_nodes_address() {
        // Grounding is identity-bearing: adding a foreign-atom edge changes the
        // node's content address (it asserts a new cross-ontology connection).
        let atom = ContentAddress::of(b"some english form");
        let mut b = base();
        b.edges.push((
            "denotes".to_string(),
            EdgeTarget::Grounded {
                ontology: "english_wordnet".to_string(),
                atom,
            },
        ));
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }
}
