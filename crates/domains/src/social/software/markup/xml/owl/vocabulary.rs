//! Runtime corpus for a loaded OWL vocabulary — the hydration target a
//! later rkyv `.prx` archive loads into, and the praxis [`Category`]
//! over its class/object-property subsumption graph.
//!
//! ## Layer position
//!
//! [`LoadedOwlVocabulary`] is the OWL analogue of
//! [`crate::social::software::markup::xml::uslm::corpus::UsCode`] (USC
//! sections) and [`crate::cognitive::linguistics::english::English`]
//! (WordNet senses): a hand-written runtime struct materialised from the
//! frozen [`CodegenData<P>`] slices a build step emits.
//!
//! ```text
//! OWL XML  ─►  owl::owl_vocabulary::owl_to_builder  ─►  CodegenData<LoadedOwlVocabulary>
//!                                                                │
//!                                                                ▼
//!                                            LoadedOwlVocabulary::from_codegen
//! ```
//!
//! The phantom marker on the data type is this same
//! [`LoadedOwlVocabulary`] struct, so handles emitted for one vocabulary
//! cannot index into another corpus.
//!
//! ## What it holds
//!
//! One owned record per OWL entity — every `owl:Class` and every
//! `owl:ObjectProperty` declared in the vocabulary, keyed by its IRI
//! (W3C OWL 2 §5). Plus the union subsumption graph: `rdfs:subClassOf`
//! (RDF Schema §2.1) on classes ∪ `rdfs:subPropertyOf` (RDF Schema
//! §5.1.7) on object properties, as `(child, parent)` index edges. This
//! is exactly the taxonomy `super::owl_vocabulary::owl_to_builder` (codegen feature)
//! produces; `from_codegen` walks the static IR back into owned indices.
//!
//! ## The Category
//!
//! [`OwlVocabularyCategory`] is the praxis [`Category`] whose objects are
//! the vocabulary's entities ([`OwlEntity`], an index handle into the
//! active vocabulary) and whose morphisms ([`OwlSubsumption`]) are the
//! `is_a` subsumption edges plus identities. Subsumption is transitive
//! (OBO-RO `transitive_over`, Smith et al. 2005; the same property the
//! W3C grounds for `rdfs:subClassOf` / `rdfs:subPropertyOf` in RDF
//! Schema §2.1 / §5.1.7), so `compose` of two subsumption edges is the
//! transitive edge. As with [`super::ontology::OwlCategory`], the
//! category is *partial* (#166): `compose` returns `Some` only when the
//! composite is itself a declared morphism, and `morphisms()` is the
//! Warshall (1962) transitive closure so every composable pair resolves
//! inside it.
//!
//! Because praxis's [`Category`]/[`Concept`] traits expose their objects
//! and morphisms through *associated functions* (no `&self`), a runtime
//! vocabulary drives them through a process-lifetime singleton — the
//! same shape [`crate::social::software::markup::xml::uslm::corpus::loaded`]
//! uses for the USC corpus. [`OwlVocabularyCategory::install`] seeds the
//! active vocabulary once; `variants()` / `morphisms()` read it back.
//!
//! ## Citations
//!
//! - **Mac Lane, S.** (1971) *Categories for the Working Mathematician*,
//!   Springer GTM 5, §I.1 (identities, composition), §I.3 (functors).
//! - **W3C OWL 2 Web Ontology Language: Structural Specification and
//!   Functional-Style Syntax (2nd ed.)**, W3C Recommendation 2012-12-11,
//!   §5 (Entities), §9.2.1 (object-property hierarchy).
//!   <https://www.w3.org/TR/owl2-syntax/>.
//! - **RDF Schema 1.1**, Brickley & Guha (eds.), W3C Recommendation
//!   2014-02-25, §2.1 (`rdfs:subClassOf`), §5.1.7 (`rdfs:subPropertyOf`).
//!   <https://www.w3.org/TR/rdf-schema/>.
//! - **Smith, B. et al.** (2005) "Relations in biomedical ontologies",
//!   *Genome Biology* 6:R46 — OBO-RO `transitive_over` for `is_a`.
//! - **Warshall, S.** (1962) "A Theorem on Boolean Matrices", *J. ACM*
//!   9(1) — transitive-closure construction of `morphisms()`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::hash::Hash;

use hashbrown::HashMap;

use pr4xis::category::entity::{Concept, FinitelyGenerated};
use pr4xis::category::{Arrow, Category};
use pr4xis::codegen_data::CodegenData;
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

/// The `entity_kind` tag for an OWL class entity — the W3C OWL 2
/// metaclass name (`owl:Class`, §5.1). Must match the tag
/// [`super::owl_vocabulary`] writes into the builder's `pos` field.
const KIND_CLASS: &str = "Class";
/// The `entity_kind` tag for an OWL object-property entity — the W3C
/// OWL 2 metaclass name (`owl:ObjectProperty`, §5.4).
const KIND_OBJECT_PROPERTY: &str = "ObjectProperty";

/// Local name of an IRI: the substring after the last `#` or `/`.
///
/// W3C OWL 2 §5.5 (IRIs) and RDF 1.1 Concepts §3.2 identify a term by its
/// full IRI; the local name is the human-facing label fallback used when
/// no `rdfs:label` is supplied (RDF Schema §2.4). Mirrors the identical
/// helper `owl_to_builder` applies, so the codegen path and the direct
/// [`LoadedOwlVocabulary::from_owl_ontology`] path assign the same label.
/// SPAR terms are slash-delimited
/// (`http://purl.org/spar/cito/citesAsEvidence`); the OWL/RDFS vocabulary
/// is hash-delimited (`http://www.w3.org/2002/07/owl#Class`) — handle both.
fn local_name(iri: &str) -> &str {
    match iri.rsplit_once('#') {
        Some((_, local)) if !local.is_empty() => local,
        _ => match iri.rsplit_once('/') {
            Some((_, local)) if !local.is_empty() => local,
            _ => iri,
        },
    }
}

/// The metaclass of a loaded OWL entity (W3C OWL 2 §5): either a named
/// class or a named object property. These are the only two kinds
/// `super::owl_vocabulary::owl_to_builder` (codegen feature) emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwlEntityKind {
    /// `owl:Class` (W3C OWL 2 §5.1).
    Class,
    /// `owl:ObjectProperty` (W3C OWL 2 §5.4).
    ObjectProperty,
}

impl OwlEntityKind {
    /// Parse the `entity_kind` IR tag back into the typed metaclass.
    /// `None` for any tag other than the two [`owl_to_builder`] emits.
    ///
    /// [`owl_to_builder`]: super::owl_vocabulary::owl_to_builder
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            KIND_CLASS => Some(Self::Class),
            KIND_OBJECT_PROPERTY => Some(Self::ObjectProperty),
            _ => None,
        }
    }
}

/// One loaded OWL entity — a named class or object property (W3C OWL 2
/// §5). Owned record; the runtime counterpart of the static
/// `ENTITY_IDS[i]` / `ENTITY_KIND[i]` / `ENTITY_LABELS[i]` /
/// `ENTITY_DEFS[i]` columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwlEntityRecord {
    /// The entity IRI verbatim (W3C OWL 2 §5.5) — the identity key.
    pub iri: String,
    /// `owl:Class` or `owl:ObjectProperty`.
    pub kind: OwlEntityKind,
    /// `rdfs:label`, falling back to the IRI local name (RDF Schema §2.4).
    pub label: String,
    /// `rdfs:comment`, empty when the source had none (RDF Schema §2.5).
    pub definition: String,
}

/// The loaded OWL vocabulary corpus.
///
/// Materialised by [`LoadedOwlVocabulary::from_codegen`] from the
/// build-time [`CodegenData<LoadedOwlVocabulary>`] static. The phantom
/// marker on the data type is this same struct, keeping codegen and
/// runtime aligned at compile time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedOwlVocabulary {
    /// Entities in IR order (the order `owl_to_builder` declared them:
    /// classes first, then object properties).
    entities: Vec<OwlEntityRecord>,
    /// IRI → index into `entities`.
    by_iri: HashMap<String, usize>,
    /// `(child, parent)` subsumption edges as indices into `entities` —
    /// `rdfs:subClassOf` ∪ `rdfs:subPropertyOf` (RDF Schema §2.1 / §5.1.7).
    subsumption: Vec<(usize, usize)>,
}

impl LoadedOwlVocabulary {
    /// Functor: `CodegenData<LoadedOwlVocabulary>` → `LoadedOwlVocabulary`.
    ///
    /// Mirrors [`crate::social::software::markup::xml::uslm::corpus::UsCode::from_codegen`]
    /// and `English::from_codegen`. Walks `entity_ids` / `entity_kind` /
    /// `entity_labels` / `entity_defs` to build the owned entity records
    /// and the IRI index, then re-keys the `taxonomy` edges (whose
    /// endpoints are [`pr4xis::EntityRef`] handles carrying the same `u64`
    /// indices the codegen emitter assigned) into owned `(child, parent)`
    /// index pairs.
    ///
    /// An entity whose `entity_kind` tag is neither `"Class"` nor
    /// `"ObjectProperty"` is skipped (the OWL builder never produces such
    /// a tag, but the empty stub a build step emits when no OWL file is on
    /// disk has zero entities, so the loop is simply empty there).
    pub fn from_codegen(data: &CodegenData<LoadedOwlVocabulary>) -> Self {
        let mut entities: Vec<OwlEntityRecord> = Vec::with_capacity(data.entity_count);
        // Map the IR index → the index in `entities`. Identity unless an
        // entity was skipped, in which case later indices shift down; the
        // taxonomy re-keying below consults this map so edges stay valid.
        let mut ir_to_owned: HashMap<usize, usize> = HashMap::with_capacity(data.entity_count);
        let mut by_iri: HashMap<String, usize> = HashMap::with_capacity(data.entity_count);

        for ir_idx in 0..data.entity_count {
            let Some(kind) = OwlEntityKind::from_tag(data.entity_kind[ir_idx]) else {
                continue;
            };
            let iri = data.entity_ids[ir_idx].to_string();
            let owned_idx = entities.len();
            by_iri.insert(iri.clone(), owned_idx);
            ir_to_owned.insert(ir_idx, owned_idx);
            entities.push(OwlEntityRecord {
                iri,
                kind,
                label: data.entity_labels[ir_idx].to_string(),
                definition: data.entity_defs[ir_idx].to_string(),
            });
        }

        // Re-key taxonomy edges. Each `EntityRef` carries the IR index as
        // its `u64` value (see `pr4xis::codegen::generate` —
        // `EntityRef::<Marker>::new(idx u64)`). Drop any edge whose
        // endpoint was skipped above (cannot happen for OWL input, but the
        // re-key is total either way).
        let mut subsumption: Vec<(usize, usize)> = Vec::with_capacity(data.taxonomy.len());
        for (child_ref, parent_ref) in data.taxonomy {
            let child_ir = child_ref.value() as usize;
            let parent_ir = parent_ref.value() as usize;
            if let (Some(&c), Some(&p)) = (ir_to_owned.get(&child_ir), ir_to_owned.get(&parent_ir))
            {
                subsumption.push((c, p));
            }
        }

        Self {
            entities,
            by_iri,
            subsumption,
        }
    }

    /// Functor: [`OwlOntology`] → [`LoadedOwlVocabulary`], **without**
    /// the `codegen` build path.
    ///
    /// [`from_codegen`][Self::from_codegen] hydrates from the frozen
    /// [`CodegenData<P>`] a build step emits; that path requires
    /// `owl_to_builder` + `pr4xis::codegen` (the `codegen` feature, off
    /// the WASM build). This is the parallel runtime path: it walks a
    /// [`read_owl`]-parsed [`OwlOntology`] straight into the same owned
    /// indices, so a WASM host that fetched the authoritative RDF/XML
    /// can materialise the corpus with only [`read_owl`] (pure-Rust,
    /// wasm-OK) and the runtime types here — no codegen, no
    /// [`CodegenData`].
    ///
    /// It reproduces exactly the entity set and surviving edges of the
    /// `read_owl → owl_to_builder → CodegenData → from_codegen` chain:
    ///
    /// - **Entities.** Classes first, then object properties (the order
    ///   `owl_to_builder` declares them), deduplicated by IRI across both
    ///   kinds with a single seen-set, empty IRIs skipped. `label` is
    ///   `rdfs:label` falling back to the IRI local name (the substring
    ///   after the last `#` or `/`); `definition` is `rdfs:comment`,
    ///   empty when absent (W3C OWL 2 §5; RDF Schema §2.4 / §2.5).
    /// - **Subsumption.** The union of [`OwlOntology`]'s `taxonomy`
    ///   (`rdfs:subClassOf`) and `property_taxonomy`
    ///   (`rdfs:subPropertyOf`) fields, re-keyed to entity indices; an edge whose
    ///   endpoint is not a declared entity (e.g. a subclass of the
    ///   external `owl:Thing`) is dropped — the same drop
    ///   `pr4xis::codegen::generate`'s id-resolution makes, so the edge
    ///   count matches the `from_codegen` path's.
    ///
    /// [`OwlOntology`]: super::ontology::OwlOntology
    /// [`read_owl`]: super::reader::read_owl
    pub fn from_owl_ontology(ont: &super::ontology::OwlOntology) -> Self {
        let mut entities: Vec<OwlEntityRecord> = Vec::new();
        let mut by_iri: HashMap<String, usize> = HashMap::new();

        // Classes first, then object properties — the order
        // `owl_to_builder` declares entities. A single seen-set
        // deduplicates across both kinds, exactly as the builder does.
        for class in &ont.classes {
            if class.iri.is_empty() || by_iri.contains_key(&class.iri) {
                continue;
            }
            let label = class
                .label
                .clone()
                .unwrap_or_else(|| local_name(&class.iri).to_string());
            by_iri.insert(class.iri.clone(), entities.len());
            entities.push(OwlEntityRecord {
                iri: class.iri.clone(),
                kind: OwlEntityKind::Class,
                label,
                definition: class.comment.clone().unwrap_or_default(),
            });
        }
        for prop in &ont.properties {
            if prop.iri.is_empty() || by_iri.contains_key(&prop.iri) {
                continue;
            }
            let label = prop
                .label
                .clone()
                .unwrap_or_else(|| local_name(&prop.iri).to_string());
            by_iri.insert(prop.iri.clone(), entities.len());
            entities.push(OwlEntityRecord {
                iri: prop.iri.clone(),
                kind: OwlEntityKind::ObjectProperty,
                label,
                definition: prop.comment.clone().unwrap_or_default(),
            });
        }

        // Subsumption = subClassOf ∪ subPropertyOf, re-keyed to indices.
        // Drop an edge whose endpoint is not a declared entity — the same
        // dangling-edge drop the codegen id-resolution performs, so the
        // edge count equals the `from_codegen` path's.
        let mut subsumption: Vec<(usize, usize)> = Vec::new();
        for (child, parent) in ont.taxonomy.iter().chain(ont.property_taxonomy.iter()) {
            if let (Some(&c), Some(&p)) = (by_iri.get(child), by_iri.get(parent)) {
                subsumption.push((c, p));
            }
        }

        Self {
            entities,
            by_iri,
            subsumption,
        }
    }

    /// Number of loaded entities (classes + object properties).
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Number of subsumption edges (`subClassOf` ∪ `subPropertyOf`).
    pub fn subsumption_edge_count(&self) -> usize {
        self.subsumption.len()
    }

    /// The entity record at `index`, if any.
    pub fn entity(&self, index: usize) -> Option<&OwlEntityRecord> {
        self.entities.get(index)
    }

    /// All entity records, in load order.
    pub fn entities(&self) -> &[OwlEntityRecord] {
        &self.entities
    }

    /// The direct `(child, parent)` subsumption edges, as indices.
    pub fn subsumption_edges(&self) -> &[(usize, usize)] {
        &self.subsumption
    }

    /// Index of the entity with the given IRI, if loaded.
    pub fn find(&self, iri: &str) -> Option<usize> {
        self.by_iri.get(iri).copied()
    }

    /// The label of the entity with the given IRI (W3C RDF Schema §2.4).
    pub fn label_of(&self, iri: &str) -> Option<&str> {
        self.find(iri).map(|i| self.entities[i].label.as_str())
    }

    /// The definition (`rdfs:comment`) of the entity with the given IRI.
    pub fn definition_of(&self, iri: &str) -> Option<&str> {
        self.find(iri).map(|i| self.entities[i].definition.as_str())
    }

    /// The IRIs of every loaded `owl:Class`, in load order.
    pub fn classes(&self) -> Vec<&str> {
        self.entities
            .iter()
            .filter(|e| e.kind == OwlEntityKind::Class)
            .map(|e| e.iri.as_str())
            .collect()
    }

    /// The IRIs of every loaded `owl:ObjectProperty`, in load order.
    pub fn properties(&self) -> Vec<&str> {
        self.entities
            .iter()
            .filter(|e| e.kind == OwlEntityKind::ObjectProperty)
            .map(|e| e.iri.as_str())
            .collect()
    }

    /// Does `child` subsume directly or transitively under `parent`?
    /// I.e. is there a `subClassOf` / `subPropertyOf` chain from `child`
    /// up to `parent` (RDF Schema §2.1 / §5.1.7; OBO-RO `transitive_over`
    /// closure). `false` for `child == parent` — this is strict `is_a`,
    /// not reflexive.
    pub fn subsumes(&self, child_iri: &str, parent_iri: &str) -> bool {
        let (Some(child), Some(parent)) = (self.find(child_iri), self.find(parent_iri)) else {
            return false;
        };
        if child == parent {
            return false;
        }
        // BFS up the parent edges from `child`.
        let mut frontier: Vec<usize> = alloc::vec![child];
        let mut seen: hashbrown::HashSet<usize> = hashbrown::HashSet::new();
        seen.insert(child);
        while let Some(node) = frontier.pop() {
            for &(c, p) in &self.subsumption {
                if c == node && seen.insert(p) {
                    if p == parent {
                        return true;
                    }
                    frontier.push(p);
                }
            }
        }
        false
    }

    /// Convenience alias for [`Self::subsumes`] phrased child-first:
    /// `vocab.is_a(child, parent)` reads as "child `is_a` parent".
    pub fn is_a(&self, child_iri: &str, parent_iri: &str) -> bool {
        self.subsumes(child_iri, parent_iri)
    }
}

// =============================================================================
// The Category over a loaded OWL vocabulary.
// =============================================================================

use std::sync::OnceLock;

/// Process-lifetime singleton holding the active vocabulary the
/// [`OwlVocabularyCategory`] reads. Seeded once via
/// [`OwlVocabularyCategory::install`]; the praxis [`Category`] /
/// [`Concept`] associated functions (no `&self`) consult it. Same shape
/// [`crate::social::software::markup::xml::uslm::corpus::loaded`] uses for
/// the USC corpus.
static ACTIVE: OnceLock<LoadedOwlVocabulary> = OnceLock::new();

/// An object of [`OwlVocabularyCategory`]: an index handle into the
/// active vocabulary's entity table. Chosen over an owned-IRI newtype
/// because the category's associated functions iterate the closed entity
/// set by index, equality / hashing are trivial, and the index is the
/// same key the subsumption edges use — no string churn inside the law
/// checks. Resolve the IRI via [`OwlEntity::iri`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OwlEntity(pub u32);

impl OwlEntity {
    /// The IRI of this entity in the active vocabulary, or `""` if no
    /// vocabulary is installed / the index is out of range. The active
    /// vocabulary lives for the process lifetime in the `OnceLock`, so
    /// its owned IRI strings are effectively `&'static`.
    pub fn iri(&self) -> &'static str {
        match OwlVocabularyCategory::active() {
            Some(v) => v
                .entity(self.0 as usize)
                .map(|e| e.iri.as_str())
                .unwrap_or(""),
            None => "",
        }
    }
}

impl Concept for OwlEntity {}
impl FinitelyGenerated for OwlEntity {
    fn variants() -> Vec<Self> {
        match OwlVocabularyCategory::active() {
            Some(v) => (0..v.entity_count() as u32).map(OwlEntity).collect(),
            None => Vec::new(),
        }
    }
}

/// Relation-kind tag for [`OwlSubsumption`] (OBO-RO, Smith et al. 2005):
/// `Identity` (Mac Lane §I.1) or `Subsumption` (the `is_a` edge —
/// `rdfs:subClassOf` / `rdfs:subPropertyOf`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwlSubsumptionKind {
    /// `id_A : A → A` (Mac Lane §I.1).
    Identity,
    /// `child is_a parent` — `rdfs:subClassOf` / `rdfs:subPropertyOf`,
    /// transitive (OBO-RO `transitive_over`).
    Subsumption,
}

/// A morphism of [`OwlVocabularyCategory`]: a subsumption edge
/// `child → parent` (or an identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OwlSubsumption {
    pub from: OwlEntity,
    pub to: OwlEntity,
    pub kind: OwlSubsumptionKind,
}

impl OwlSubsumption {
    /// Identity morphism on `entity` (Mac Lane §I.1).
    pub fn identity(entity: OwlEntity) -> Self {
        Self {
            from: entity,
            to: entity,
            kind: OwlSubsumptionKind::Identity,
        }
    }

    /// Subsumption (`is_a`) morphism `child → parent`.
    pub fn subsumption(child: OwlEntity, parent: OwlEntity) -> Self {
        Self {
            from: child,
            to: parent,
            kind: OwlSubsumptionKind::Subsumption,
        }
    }
}

impl Arrow for OwlSubsumption {
    type Object = OwlEntity;
    type Kind = OwlSubsumptionKind;

    fn source(&self) -> OwlEntity {
        self.from
    }
    fn target(&self) -> OwlEntity {
        self.to
    }
    fn kind(&self) -> OwlSubsumptionKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new("OwlSubsumption"),
            description: Label::new(
                "is_a subsumption edge (rdfs:subClassOf / rdfs:subPropertyOf) in a loaded OWL vocabulary",
            ),
            citation: Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician §I.1 (identities, \
                 composition); W3C OWL 2 (2012) Structural Specification §5, §9.2.1; W3C RDF \
                 Schema 1.1 (2014) §2.1 (subClassOf), §5.1.7 (subPropertyOf); Smith et al. (2005) \
                 Genome Biology 6:R46 OBO-RO (is_a transitive_over)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The praxis [`Category`] over a loaded OWL vocabulary's subsumption
/// graph — objects are entities, morphisms are `is_a` edges + identities.
///
/// Reads the active vocabulary installed via [`Self::install`]. Partial
/// composition (#166): `compose` of two subsumption edges yields the
/// transitive edge only when it is itself a declared morphism;
/// [`Self::morphisms`] is the Warshall (1962) transitive closure, so
/// every composable pair resolves inside it.
pub struct OwlVocabularyCategory;

impl OwlVocabularyCategory {
    /// Install `vocab` as the process-wide active vocabulary the category
    /// reasons over. Idempotent only in the sense that the *first* call
    /// wins (`OnceLock`); a second call returns `Err(vocab)` with the
    /// rejected value, mirroring `OnceLock::set`.
    pub fn install(vocab: LoadedOwlVocabulary) -> Result<(), LoadedOwlVocabulary> {
        ACTIVE.set(vocab)
    }

    /// The active vocabulary, if one has been installed.
    pub fn active() -> Option<&'static LoadedOwlVocabulary> {
        ACTIVE.get()
    }

    /// The active vocabulary, panicking with a clear message if none is
    /// installed. Used by the category functions, which are only
    /// meaningful once a vocabulary is present.
    fn require_active() -> &'static LoadedOwlVocabulary {
        ACTIVE
            .get()
            .expect("OwlVocabularyCategory used before install() — no active vocabulary")
    }

    /// The transitive closure of the active vocabulary's direct
    /// subsumption edges, as `(child, parent)` index pairs (Warshall
    /// 1962). Drives both [`Self::morphisms`] and the `compose`
    /// membership check.
    fn transitive_subsumption() -> hashbrown::HashSet<(u32, u32)> {
        let v = Self::require_active();
        let mut closure: hashbrown::HashSet<(u32, u32)> = v
            .subsumption_edges()
            .iter()
            .map(|&(c, p)| (c as u32, p as u32))
            .collect();
        loop {
            let mut added = false;
            let snap: Vec<(u32, u32)> = closure.iter().copied().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && a != c && closure.insert((a, c)) {
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        closure
    }
}

impl Category for OwlVocabularyCategory {
    type Object = OwlEntity;
    type Morphism = OwlSubsumption;

    fn identity(obj: &OwlEntity) -> OwlSubsumption {
        OwlSubsumption::identity(*obj)
    }

    fn compose(f: &OwlSubsumption, g: &OwlSubsumption) -> Option<OwlSubsumption> {
        if f.to != g.from {
            return None;
        }
        if f.kind == OwlSubsumptionKind::Identity {
            return Some(*g);
        }
        if g.kind == OwlSubsumptionKind::Identity {
            return Some(*f);
        }
        // Both Subsumption. The composite is the transitive is_a edge
        // f.from → g.to (OBO-RO transitive_over). Partial category
        // (#166): only return it if it is a declared morphism — i.e. in
        // the transitive closure that `morphisms()` enumerates.
        let candidate = OwlSubsumption::subsumption(f.from, g.to);
        if Self::transitive_subsumption().contains(&(f.from.0, g.to.0)) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<OwlSubsumption> {
        let mut out: Vec<OwlSubsumption> = OwlEntity::variants()
            .into_iter()
            .map(OwlSubsumption::identity)
            .collect();
        for (child, parent) in Self::transitive_subsumption() {
            // The closure excludes self-pairs (a != c guard above and the
            // strict edges), so no duplicate of an identity is emitted.
            out.push(OwlSubsumption::subsumption(
                OwlEntity(child),
                OwlEntity(parent),
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::owl::owl_vocabulary::owl_to_builder;
    use crate::social::software::markup::xml::owl::reader::read_owl;
    use pr4xis::EntityRef;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::codegen::OntologyBuilder;
    use proptest::prelude::*;
    use std::sync::Mutex;

    /// The bundled CiTO 2.8.1 OWL vocabulary (SPAR Ontologies), embedded
    /// at build time — the same source the codegen-side tests use.
    const CITO_2_8_1_OWL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/cito-2.8.1.owl"
    ));

    const CITES_AS_EVIDENCE_IRI: &str = "http://purl.org/spar/cito/citesAsEvidence";
    const CITES_IRI: &str = "http://purl.org/spar/cito/cites";

    /// Turn an [`OntologyBuilder`] into an owned `CodegenData` with
    /// `&'static` slices, the way the build-time emitter would but at
    /// test time. This is the load-bearing helper: `CodegenData<P>`'s
    /// fields are `&'static`, so we `Box::leak` the owned columns to
    /// match — exactly the leak-to-`'static` strategy
    /// `UsCode::from_uslm_titles_owned` uses for the runtime USC corpus
    /// (process-lifetime leaks, equivalent to build-emitted statics).
    /// The id→index resolution + dangling-edge drop mirror
    /// `pr4xis::codegen::generate::write_raw_relations`.
    fn codegen_data_from_builder(builder: &OntologyBuilder) -> CodegenData<LoadedOwlVocabulary> {
        fn leak_strs(items: Vec<String>) -> &'static [&'static str] {
            let refs: Vec<&'static str> = items
                .into_iter()
                .map(|s| -> &'static str { Box::leak(s.into_boxed_str()) })
                .collect();
            Box::leak(refs.into_boxed_slice())
        }

        let id_to_idx: HashMap<&str, u32> = builder
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.as_str(), i as u32))
            .collect();

        let entity_ids = leak_strs(builder.entities.iter().map(|e| e.id.clone()).collect());
        let entity_kind = leak_strs(
            builder
                .entities
                .iter()
                .map(|e| e.pos.clone().unwrap_or_default())
                .collect(),
        );
        let entity_labels = leak_strs(builder.entities.iter().map(|e| e.label.clone()).collect());
        let entity_defs = leak_strs(
            builder
                .entities
                .iter()
                .map(|e| e.definitions.first().cloned().unwrap_or_default())
                .collect(),
        );

        let taxonomy: Vec<(
            EntityRef<LoadedOwlVocabulary>,
            EntityRef<LoadedOwlVocabulary>,
        )> = builder
            .taxonomy
            .iter()
            .filter_map(|(c, p)| {
                let ci = *id_to_idx.get(c.as_str())?;
                let pi = *id_to_idx.get(p.as_str())?;
                Some((EntityRef::new(ci as u64), EntityRef::new(pi as u64)))
            })
            .collect();
        let taxonomy: &'static [(
            EntityRef<LoadedOwlVocabulary>,
            EntityRef<LoadedOwlVocabulary>,
        )] = Box::leak(taxonomy.into_boxed_slice());

        let empty_rel: &'static [(
            EntityRef<LoadedOwlVocabulary>,
            EntityRef<LoadedOwlVocabulary>,
        )] = &[];

        CodegenData {
            entity_count: builder.entities.len(),
            entity_ids,
            entity_kind,
            entity_labels,
            entity_defs,
            word_index: &[],
            taxonomy,
            mereology: empty_rel,
            opposition: empty_rel,
            equivalence: empty_rel,
            causation: empty_rel,
            references: empty_rel,
        }
    }

    /// Build the CiTO runtime vocabulary through the full
    /// reader → builder → CodegenData → from_codegen path.
    fn cito_vocabulary() -> LoadedOwlVocabulary {
        let ont = read_owl(CITO_2_8_1_OWL).expect("bundled CiTO must parse");
        let builder = owl_to_builder(&ont);
        let data = codegen_data_from_builder(&builder);
        LoadedOwlVocabulary::from_codegen(&data)
    }

    // ── from_codegen round-trip on the real bundled CiTO ─────────────

    #[test]
    fn from_codegen_round_trips_cito() {
        let vocab = cito_vocabulary();

        // CiTO declares dozens of citation-typing object properties plus
        // a few classes.
        assert!(
            vocab.entity_count() > 30,
            "expected >30 CiTO entities, got {}",
            vocab.entity_count()
        );

        // citesAsEvidence must be present as an ObjectProperty.
        let cae = vocab
            .find(CITES_AS_EVIDENCE_IRI)
            .expect("citesAsEvidence must be loaded");
        assert_eq!(
            vocab.entity(cae).unwrap().kind,
            OwlEntityKind::ObjectProperty,
            "citesAsEvidence must be an ObjectProperty"
        );

        // The subsumption taxonomy (subPropertyOf ∪ subClassOf) is rich.
        assert!(
            vocab.subsumption_edge_count() > 0,
            "CiTO subsumption taxonomy must be non-empty"
        );

        // citesAsEvidence is_a cites (CiTO declares
        // citesAsEvidence rdfs:subPropertyOf cites).
        assert!(
            vocab.is_a(CITES_AS_EVIDENCE_IRI, CITES_IRI),
            "citesAsEvidence must subsume under cites"
        );
        assert!(
            vocab.subsumes(CITES_AS_EVIDENCE_IRI, CITES_IRI),
            "subsumes must agree with is_a"
        );
        // Strict, not reflexive.
        assert!(!vocab.subsumes(CITES_IRI, CITES_IRI));
    }

    // ── from_owl_ontology parity with from_codegen on bundled CiTO ───

    /// The non-codegen [`LoadedOwlVocabulary::from_owl_ontology`] path
    /// (the one wasm uses — `read_owl` straight into the owned indices)
    /// yields the *same* corpus as the codegen
    /// `owl_to_builder → CodegenData → from_codegen` path: the two
    /// functors agree object-for-object and edge-for-edge. This is the
    /// load-bearing equivalence for #257 — a source-loaded vocabulary is
    /// indistinguishable from a build-emitted one.
    ///
    /// The two paths consume **independent** parses of the same bytes.
    /// `read_owl`'s `deduplicate_classes` / `deduplicate_properties`
    /// preserve first-occurrence document order, so two separate parses of
    /// the same source list entities in the same order (#264). Feeding the
    /// two paths from two distinct parses therefore proves both the functor
    /// equivalence *and* that `read_owl` is order-stable across parses — a
    /// stronger guarantee than sharing one parse would give.
    #[test]
    fn from_owl_ontology_equals_from_codegen_on_cito() {
        // from_codegen path, sourced from its own parse.
        let ont_codegen = read_owl(CITO_2_8_1_OWL).expect("bundled CiTO must parse");
        let builder = owl_to_builder(&ont_codegen);
        let data = codegen_data_from_builder(&builder);
        let via_codegen = LoadedOwlVocabulary::from_codegen(&data);

        // from_owl_ontology path, sourced from a *separate* parse of the
        // same bytes — deterministic ordering makes the two agree.
        let ont_owl = read_owl(CITO_2_8_1_OWL).expect("bundled CiTO must parse");
        let via_owl = LoadedOwlVocabulary::from_owl_ontology(&ont_owl);

        assert_eq!(
            via_owl.entity_count(),
            via_codegen.entity_count(),
            "entity counts must match across the two load paths"
        );
        assert_eq!(
            via_owl.subsumption_edge_count(),
            via_codegen.subsumption_edge_count(),
            "subsumption-edge counts must match across the two load paths"
        );
        // Same entity set (IRI + kind + label + definition), in the same
        // load order: the two paths produce identical owned values.
        assert_eq!(
            via_owl, via_codegen,
            "from_owl_ontology must reproduce the from_codegen corpus exactly"
        );
        // And the citation-typing taxonomy survives the direct path.
        assert!(via_owl.entity_count() > 30, "real CiTO is rich");
        assert!(
            via_owl.is_a(CITES_AS_EVIDENCE_IRI, CITES_IRI),
            "citesAsEvidence is_a cites must hold on the direct path"
        );
    }

    #[test]
    fn classes_and_properties_partition_entities() {
        let vocab = cito_vocabulary();
        assert_eq!(
            vocab.classes().len() + vocab.properties().len(),
            vocab.entity_count(),
            "every entity is either a Class or an ObjectProperty"
        );
        // cites is an object property; it has a label and definition.
        assert!(vocab.label_of(CITES_IRI).is_some_and(|l| !l.is_empty()));
        assert!(
            vocab
                .find("http://purl.org/spar/cito/this-iri-does-not-exist")
                .is_none()
        );
    }

    // ── Category laws on the materialised CiTO vocabulary ────────────

    /// The `ACTIVE` singleton is process-global; the two tests that
    /// install it must not race. A mutex serialises them, and we only
    /// install once (OnceLock).
    static INSTALL_LOCK: Mutex<()> = Mutex::new(());

    fn ensure_cito_installed() {
        let _guard = INSTALL_LOCK.lock().unwrap();
        if OwlVocabularyCategory::active().is_none() {
            // First installer wins; ignore the Err if another test in the
            // same process already installed an identical vocabulary.
            let _ = OwlVocabularyCategory::install(cito_vocabulary());
        }
    }

    #[test]
    fn category_laws_hold_on_cito() {
        ensure_cito_installed();
        assert_category_laws::<OwlVocabularyCategory>();
    }

    #[test]
    fn category_morphisms_include_identities_and_subsumptions() {
        ensure_cito_installed();
        let ms = OwlVocabularyCategory::morphisms();
        let id_count = ms
            .iter()
            .filter(|m| m.kind == OwlSubsumptionKind::Identity)
            .count();
        let v = OwlVocabularyCategory::active().unwrap();
        assert_eq!(id_count, v.entity_count(), "one identity per entity");
        assert!(
            ms.iter().any(|m| m.kind == OwlSubsumptionKind::Subsumption),
            "at least one subsumption morphism"
        );

        // citesAsEvidence → cites is a (transitive-closure) morphism.
        let cae = OwlEntity(v.find(CITES_AS_EVIDENCE_IRI).unwrap() as u32);
        let cites = OwlEntity(v.find(CITES_IRI).unwrap() as u32);
        assert!(
            ms.contains(&OwlSubsumption::subsumption(cae, cites)),
            "citesAsEvidence → cites must be a declared morphism"
        );
        // And OwlEntity::iri resolves back to the IRI.
        assert_eq!(cae.iri(), CITES_AS_EVIDENCE_IRI);
    }

    // ── proptest over synthesised small vocabularies ─────────────────

    /// A synthesised vocabulary: a few class IRIs, a few property IRIs,
    /// and within-kind child→parent edges. Reuses the non-self-loop edge
    /// construction from the codegen-side proptests.
    #[derive(Debug, Clone)]
    struct SynthVocab {
        classes: Vec<String>,
        properties: Vec<String>,
        class_edges: Vec<(usize, usize)>,
        prop_edges: Vec<(usize, usize)>,
    }

    /// Acyclic child→parent edges: the parent index is always strictly
    /// greater than the child (`child < parent < n`), so the synthesised
    /// hierarchy is a DAG — matching real `rdfs:subClassOf` /
    /// `rdfs:subPropertyOf` graphs, which are acyclic (a cycle would
    /// assert mutual equivalence, a degenerate case). De-duplicated so
    /// edge counts are exact. Yields no edges when `n < 2`.
    fn arb_edges(n: usize) -> BoxedStrategy<Vec<(usize, usize)>> {
        if n < 2 {
            return Just(Vec::new()).boxed();
        }
        proptest::collection::vec((0..n, 1..n), 0..6)
            .prop_map(move |raw| {
                let mut edges: Vec<(usize, usize)> = raw
                    .into_iter()
                    // child < parent: pick parent in (child, n), wrapping
                    // a too-small raw offset up past `child` so the edge
                    // always points to a strictly larger index.
                    .filter_map(|(child, raw_parent)| {
                        let parent = child + 1 + (raw_parent % (n - 1));
                        if parent < n {
                            Some((child, parent))
                        } else {
                            None
                        }
                    })
                    .collect();
                edges.sort_unstable();
                edges.dedup();
                edges
            })
            .boxed()
    }

    fn arb_synth() -> impl Strategy<Value = SynthVocab> {
        (1usize..=5, 1usize..=5).prop_flat_map(|(n_cls, n_prop)| {
            let classes: Vec<String> = (0..n_cls)
                .map(|i| format!("http://ex.org/v#C{i}"))
                .collect();
            let properties: Vec<String> = (0..n_prop)
                .map(|i| format!("http://ex.org/v#p{i}"))
                .collect();
            (
                Just(classes),
                Just(properties),
                arb_edges(n_cls),
                arb_edges(n_prop),
            )
                .prop_map(|(classes, properties, class_edges, prop_edges)| {
                    SynthVocab {
                        classes,
                        properties,
                        class_edges,
                        prop_edges,
                    }
                })
        })
    }

    /// Build a builder directly from the synthesised vocabulary (one
    /// entity per class/property, kinds tagged, within-kind taxonomy
    /// edges) — no XML round-trip needed; the codegen-side proptests
    /// already cover `owl_to_builder` from a synthesised `OwlOntology`.
    fn synth_builder(s: &SynthVocab) -> OntologyBuilder {
        use pr4xis::codegen::EntityDef;
        let mut b = OntologyBuilder::new();
        for iri in &s.classes {
            b.add_entity(EntityDef::new(iri, iri).pos(KIND_CLASS));
        }
        for iri in &s.properties {
            b.add_entity(EntityDef::new(iri, iri).pos(KIND_OBJECT_PROPERTY));
        }
        for (c, p) in &s.class_edges {
            b.add_taxonomy(&s.classes[*c], &s.classes[*p]);
        }
        for (c, p) in &s.prop_edges {
            b.add_taxonomy(&s.properties[*c], &s.properties[*p]);
        }
        b
    }

    /// Build the parallel [`OwlOntology`] for the same synthesised
    /// vocabulary — the input the non-codegen
    /// [`LoadedOwlVocabulary::from_owl_ontology`] path consumes. No
    /// labels / comments (the IRI-local-name fallback applies), within-
    /// kind subsumption edges split between `taxonomy` (classes) and
    /// `property_taxonomy` (properties), mirroring what `read_owl` returns.
    fn synth_owl_ontology(
        s: &SynthVocab,
    ) -> crate::social::software::markup::xml::owl::ontology::OwlOntology {
        use crate::social::software::markup::xml::owl::ontology::{
            OwlClass, OwlObjectProperty, OwlOntology,
        };
        OwlOntology {
            iri: "http://ex.org/v#".to_string(),
            classes: s
                .classes
                .iter()
                .map(|iri| OwlClass {
                    iri: iri.clone(),
                    ..Default::default()
                })
                .collect(),
            properties: s
                .properties
                .iter()
                .map(|iri| OwlObjectProperty {
                    iri: iri.clone(),
                    ..Default::default()
                })
                .collect(),
            taxonomy: s
                .class_edges
                .iter()
                .map(|(c, p)| (s.classes[*c].clone(), s.classes[*p].clone()))
                .collect(),
            property_taxonomy: s
                .prop_edges
                .iter()
                .map(|(c, p)| (s.properties[*c].clone(), s.properties[*p].clone()))
                .collect(),
            ..Default::default()
        }
    }

    proptest! {
        /// from_codegen preserves the entity count and every direct edge.
        #[test]
        fn prop_from_codegen_preserves_entities_and_edges(s in arb_synth()) {
            let builder = synth_builder(&s);
            let data = codegen_data_from_builder(&builder);
            let vocab = LoadedOwlVocabulary::from_codegen(&data);

            prop_assert_eq!(vocab.entity_count(), s.classes.len() + s.properties.len());
            prop_assert_eq!(
                vocab.subsumption_edge_count(),
                s.class_edges.len() + s.prop_edges.len()
            );
            // Every direct class edge survives as an is_a (direct ⇒ subsumes).
            for (c, p) in &s.class_edges {
                prop_assert!(vocab.is_a(&s.classes[*c], &s.classes[*p]));
            }
            for (c, p) in &s.prop_edges {
                prop_assert!(vocab.is_a(&s.properties[*c], &s.properties[*p]));
            }
        }

        /// compose of two subsumption edges yields the transitive edge,
        /// and identity laws hold — verified on a standalone vocabulary
        /// value (no global install needed for the structural checks
        /// below, which operate on the materialised vocabulary directly).
        #[test]
        fn prop_subsumes_is_transitive(s in arb_synth()) {
            let builder = synth_builder(&s);
            let data = codegen_data_from_builder(&builder);
            let vocab = LoadedOwlVocabulary::from_codegen(&data);

            // Transitivity of subsumes: if a is_a b and b is_a c then a is_a c.
            let iris: Vec<String> = s.classes.iter().chain(s.properties.iter()).cloned().collect();
            for a in &iris {
                for b in &iris {
                    for c in &iris {
                        if vocab.is_a(a, b) && vocab.is_a(b, c) {
                            prop_assert!(
                                vocab.is_a(a, c),
                                "transitivity: {} is_a {} is_a {} but not {} is_a {}",
                                a, b, c, a, c
                            );
                        }
                    }
                }
            }
        }

        /// The two load paths agree on every synthesised vocabulary: the
        /// non-codegen `from_owl_ontology` (wasm's path) materialises the
        /// exact same corpus as `from_codegen` (the build path). This is
        /// the #257 equivalence at full structural generality.
        ///
        /// Both paths start from the same synthesised [`OwlOntology`]: the
        /// `from_codegen` reference routes through the production
        /// `owl_to_builder` (so labels take the same `rdfs:label`-then-
        /// local-name fallback `from_owl_ontology` applies), not the
        /// bespoke `synth_builder` helper (which labels each entity with
        /// its full IRI for the structural from_codegen proptests above).
        #[test]
        fn prop_from_owl_ontology_equals_from_codegen(s in arb_synth()) {
            let ont = synth_owl_ontology(&s);
            let via_codegen = {
                let builder = owl_to_builder(&ont);
                let data = codegen_data_from_builder(&builder);
                LoadedOwlVocabulary::from_codegen(&data)
            };
            let via_owl = LoadedOwlVocabulary::from_owl_ontology(&ont);
            prop_assert_eq!(via_owl, via_codegen);
        }
    }
}
