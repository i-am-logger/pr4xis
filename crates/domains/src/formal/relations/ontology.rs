//! Relations — the canonical vocabulary of binary relation types
//! that any ontology can use to label its edges.
//!
//! This is pr4xis's answer to "what KIND of relation is this edge?".
//! Every directed edge in every pr4xis ontology carries a kind tag;
//! this ontology enumerates the kinds that are first-class-known
//! across the workspace and says — for each — which structural
//! properties (symmetric, antisymmetric, transitive, …) they satisfy
//! by definition.
//!
//! Six literature lineages supply the content:
//!
//! 1. **Applied-ontology tradition** — Smith et al. (2005) *Relations
//!    in biomedical ontologies* (OBO Relation Ontology), Genome Biology
//!    6:R46, and SKOS / SKOS-XL (W3C 2009). Source of eleven of the
//!    thirteen relation types (`is_a` / `part_of` / `causally_related_to` /
//!    `related_to` / `precedes` / `broader` / `narrower` / `related` /
//!    `exactMatch` / `depends_on` / `member_of`); the twelfth (`cites`)
//!    comes from lineage 5 and the thirteenth (`replaces`) from lineage 6.
//!
//! 2. **Formal relation algebra** — Tarski (1941) *On the calculus of
//!    relations* (J. Symbolic Logic 6), for the algebraic names of the
//!    structural properties (`symmetric`, `transitive`, `reflexive`,
//!    `irreflexive`, `antisymmetric`, `functional`) and their
//!    interactions.
//!
//! 3. **Logical foundations** — Russell & Whitehead *Principia
//!    Mathematica* (1910–13) Vol. I §§30–35, for binary relations as
//!    logical primitives and the laws they obey.
//!
//! 4. **Upper ontology alignment** — Masolo et al. (2003) *DOLCE*
//!    (WonderWeb D18), for how binary relations sit in a foundational
//!    ontology alongside Endurants, Perdurants, Qualities.
//!
//! 5. **Citation typing** — Peroni & Shotton (2012) *FaBiO and CiTO*
//!    (J. Web Semantics 17), for `cito:cites` — the directed
//!    cross-reference one document (here, a statute provision) asserts
//!    to another.
//!
//! 6. **Resource-versioning vocabulary** — DCMI Metadata Terms (Dublin
//!    Core, 2020), for `dcterms:replaces`/`dcterms:isReplacedBy` — the
//!    directed supersession relation between a replaced resource and its
//!    replacement.
//!
//! ## Why this is a full ontology and not a Rust enum
//!
//! The structural-axiom modules in `pr4xis::ontology::reasoning::*` used
//! to privilege four relation types with hardcoded traits (TaxonomyDef /
//! MereologyDef / CausalDef / OppositionDef). That was a category error:
//! the axioms (symmetric, irreflexive, no-cycles, etc.) are *properties
//! of relations*, not type-level distinctions. This ontology fixes that
//! — relation types are first-class entities with their own literature
//! citations and their own structural-property qualities.
//!
//! The pr4xis-core structural axioms (`NoCyclesOnKind`, `SymmetricOnKind`,
//! etc. in `pr4xis::ontology::reasoning::structural`) consume these
//! relation concepts by name — when the `ontology!` macro emits a
//! `Subsumption`-kinded edge, the kind name matches the Relations
//! concept name by convention.
//!
//! Source: Smith et al. (2005) Genome Biology 6:R46; SKOS (W3C 2009);
//! Tarski (1941) Calculus of Relations; Russell & Whitehead Principia
//! (1910–13); Masolo et al. (2003) DOLCE; Peroni & Shotton (2012) CiTO.

#[allow(unused_imports)]
use alloc::{
    boxed::Box, collections::BTreeSet, format, string::String, string::ToString, vec, vec::Vec,
};

use pr4xis::ontology::{Axiom, Ontology, Quality};
use pr4xis_runtime::ontology::{ConceptRef, relations_kind};

pr4xis::ontology! {
    name: "Relations",
    source: "Smith et al. (2005) Genome Biology 6:R46 (OBO-RO); SKOS (W3C 2009); Tarski (1941) J. Symbolic Logic 6; Russell & Whitehead Principia Mathematica (1910–13); Masolo et al. (2003) DOLCE WonderWeb D18",

    concepts: [
        // === Binary relation types (13) — what a kinded edge can mean ===
        Subsumption,
        Parthood,
        Causation,
        Opposition,
        Similarity,
        Precedence,
        Equivalence,
        Specialisation,
        Dependence,
        Association,
        MemberOf,
        Cites,
        Supersession,

        // === Structural properties (7) — what a relation type satisfies ===
        // These are Qualities-of-Relations, not relations themselves.
        Symmetric,
        Antisymmetric,
        Transitive,
        Reflexive,
        Irreflexive,
        Functional,
        Involutive,

        // === Abstract parent categories ===
        RelationType,
        StructuralProperty,
    ],

    labels: {
        // --- Relation types ---
        Subsumption: ("en", "Subsumption (is-a)",
            "Smith et al. OBO-RO `is_a`; SKOS `broader`. The relation between a specific kind and its general kind. Antisymmetric, transitive, reflexive."),
        Parthood: ("en", "Parthood (part-of)",
            "Smith et al. OBO-RO `part_of`; Casati & Varzi (1999). The mereological part-of relation. Antisymmetric, transitive."),
        Causation: ("en", "Causation (causes)",
            "Smith et al. OBO-RO `causally_related_to`; Lewis (1973). The relation between a cause and its effect. Asymmetric, irreflexive."),
        Opposition: ("en", "Opposition (antonym-of / opposed-to)",
            "SKOS `related` with semantic polarity; Saussure (1916); Cruse (1986). The relation between mutually-exclusive or polar terms. Symmetric, irreflexive."),
        Similarity: ("en", "Similarity (resembles)",
            "Tversky (1977) features of similarity. Non-transitive: A resembles B and B resembles C does not imply A resembles C. Symmetric in classical views, asymmetric in Tversky's."),
        Precedence: ("en", "Precedence (precedes)",
            "Allen (1983) interval algebra; OBO-RO `precedes`. Temporal or logical before-ness. Asymmetric, irreflexive, transitive."),
        Equivalence: ("en", "Equivalence (same-as)",
            "SKOS `exactMatch`. The relation that holds between an entity and itself, and (symmetrically/transitively) between indistinguishable entities. Forms a groupoid. Symmetric, reflexive, transitive."),
        Specialisation: ("en", "Specialisation (narrower)",
            "SKOS `narrower`. The inverse of Subsumption. A specialisation is-a specific kind of its parent. Antisymmetric, transitive, irreflexive (strict)."),
        Dependence: ("en", "Dependence (depends-on)",
            "Simons (1987) Parts: A Study in Ontology; OBO-RO `depends_on`. Ontological dependence — A cannot exist without B. Asymmetric, irreflexive."),
        Association: ("en", "Association (related-to)",
            "SKOS `related`. Uncommitted fallback when no stronger relation applies. Symmetric by default but carries no other structural claim."),
        MemberOf: ("en", "Member-of",
            "Smith et al. OBO-RO `member_of`; SKOS `skos:member` (inverse direction). The relation between an individual and the classification it belongs to (e.g. a verb and its VerbNet syntactic-semantic class) — distinct from Subsumption (a class-of-classes relation: a subclass IS a more specific class) and from Parthood (a whole-part relation: a member is not a physical part of its class). Irreflexive, not symmetric, not transitive at this kind alone (composing with the classification's own Subsumption-kinded subclass hierarchy is what licenses \"member of an ancestor class\", not iterating MemberOf itself)."),
        Cites: ("en", "Cites (cross-reference)",
            "Peroni & Shotton (2012) CiTO `cito:cites` — the citing entity references the cited entity. The relation a bare document cross-reference (a USLM `<ref href=\"…\">` from one statute provision to another) asserts: a pointer, NOT incorporation-by-reference (which carries binding force — that is a stronger, phrase-licensed relation, distinct from a plain hyperlink). Irreflexive (a provision does not cite itself), not symmetric (A cites B ⇏ B cites A), not transitive (A cites B, B cites C ⇏ A cites C) — a citation is a single directed edge, resolved (same-document, cross-document, or dangling) by a dedicated grounding lens, never by iterating the kind."),
        Supersession: ("en", "Supersession (replaces / supplants)",
            "DCMI Metadata Terms (Dublin Core, 2020) `dcterms:replaces` (\"a related resource that is supplanted, displaced, or superseded by the described resource\") and its declared inverse `dcterms:isReplacedBy` (\"a related resource that supplants, displaces, or supersedes the described resource\") — a genuine, citable, directional RDF relation, the same standard-vocabulary-as-relation-kind-authority pattern Parthood (OBO-RO) and Equivalence (SKOS) already use. Directional (A superseding B does not make B supersede A) and irreflexive (a resource does not supersede itself); NOT transitive at this kind alone (A supersedes B, B supersedes C does not by itself license A supersedes C — the same non-iteration discipline `Cites`/`MemberOf` already document)."),

        // --- Structural properties ---
        Symmetric: ("en", "Symmetric",
            "Tarski (1941): R is symmetric iff (A R B) ⇒ (B R A) for all A, B. Opposition, Similarity, Equivalence, Association satisfy this."),
        Antisymmetric: ("en", "Antisymmetric",
            "Tarski (1941): R is antisymmetric iff (A R B) ∧ (B R A) ⇒ A = B. Subsumption, Parthood, Specialisation satisfy this."),
        Transitive: ("en", "Transitive",
            "Tarski (1941): R is transitive iff (A R B) ∧ (B R C) ⇒ (A R C). Subsumption, Parthood, Precedence, Equivalence, Specialisation satisfy this."),
        Reflexive: ("en", "Reflexive",
            "Tarski (1941): R is reflexive iff (A R A) for all A. Subsumption (trivially: A is-a A), Equivalence satisfy this."),
        Irreflexive: ("en", "Irreflexive",
            "Tarski (1941): R is irreflexive iff ¬(A R A) for any A. Opposition, Causation (strict), Precedence, Dependence satisfy this."),
        Functional: ("en", "Functional",
            "Tarski (1941): R is functional iff each A has at most one B with (A R B). A relation that acts like a function."),
        Involutive: ("en", "Involutive",
            "Tarski (1941): R is involutive iff (A R B) ∧ (B R C) ⇒ A = C — applying R twice returns the origin. Opposition with negation is involutive (opposite-of-opposite is self)."),

        // --- Abstract parents ---
        RelationType: ("en", "Relation type",
            "Abstract parent of the eleven canonical binary relation types. Anything kinded as a RelationType has a direction, a source, and a target."),
        StructuralProperty: ("en", "Structural property",
            "Abstract parent of the seven algebraic properties (symmetric, transitive, etc.) that classify a relation. From Tarski (1941) relation algebra."),
    },

    is_a: [
        // All thirteen relation types are RelationTypes.
        (Subsumption, RelationType),
        (Parthood, RelationType),
        (Causation, RelationType),
        (Opposition, RelationType),
        (Similarity, RelationType),
        (Precedence, RelationType),
        (Equivalence, RelationType),
        (Specialisation, RelationType),
        (Dependence, RelationType),
        (Association, RelationType),
        (MemberOf, RelationType),
        (Cites, RelationType),
        (Supersession, RelationType),

        // All seven structural properties are StructuralProperties.
        (Symmetric, StructuralProperty),
        (Antisymmetric, StructuralProperty),
        (Transitive, StructuralProperty),
        (Reflexive, StructuralProperty),
        (Irreflexive, StructuralProperty),
        (Functional, StructuralProperty),
        (Involutive, StructuralProperty),
    ],

    edges: [
        // Subsumption ↔ Specialisation are inverses (SKOS `broader` / `narrower`).
        (Subsumption, Specialisation, InverseOf),
        (Specialisation, Subsumption, InverseOf),

        // Equivalence refines mutual Subsumption (A is-a B AND B is-a A ⇒ A = B by antisymmetry,
        // i.e. Equivalence collapses those cases).
        (Equivalence, Subsumption, RefinesWith),

        // Opposition excludes Equivalence (Aristotelian: A opposes B ⇒ A ≢ B).
        (Opposition, Equivalence, ExcludesWith),

        // Parthood is distinct from Subsumption (Noonan-Varzi: part-of is not is-a).
        (Parthood, Subsumption, DistinctFrom),

        // Dependence subsumes more-specific Causation cases (every cause is depended-on
        // for the effect to occur, but not every dependence is a causal relation).
        (Causation, Dependence, SpecialisationOf),

        // HasProperty — each relation type's definitional structural properties as
        // LOADED typed edges (audit 2026-06-12 D-7), not a hand-authored Rust
        // `match` in `RelationProperty::get`. Canonical catalog: Tarski (1941) +
        // OBO-RO + SKOS. Both endpoints are concepts in this ontology, so the
        // relation→property assignment is an ordinary kinded morphism, queryable
        // like every other edge.
        (Subsumption, Antisymmetric, HasProperty),
        (Subsumption, Transitive, HasProperty),
        (Subsumption, Reflexive, HasProperty),
        (Parthood, Antisymmetric, HasProperty),
        (Parthood, Transitive, HasProperty),
        (Parthood, Irreflexive, HasProperty),
        (Causation, Irreflexive, HasProperty),
        (Causation, Transitive, HasProperty),
        (Opposition, Symmetric, HasProperty),
        (Opposition, Irreflexive, HasProperty),
        (Opposition, Involutive, HasProperty),
        (Similarity, Symmetric, HasProperty),
        (Similarity, Reflexive, HasProperty),
        (Precedence, Irreflexive, HasProperty),
        (Precedence, Transitive, HasProperty),
        (Precedence, Antisymmetric, HasProperty),
        (Equivalence, Symmetric, HasProperty),
        (Equivalence, Reflexive, HasProperty),
        (Equivalence, Transitive, HasProperty),
        (Specialisation, Antisymmetric, HasProperty),
        (Specialisation, Transitive, HasProperty),
        (Specialisation, Irreflexive, HasProperty),
        (Dependence, Irreflexive, HasProperty),
        (Dependence, Transitive, HasProperty),
        (Dependence, Antisymmetric, HasProperty),
        (Association, Symmetric, HasProperty),
        (MemberOf, Irreflexive, HasProperty),
        (Cites, Irreflexive, HasProperty),
        (Supersession, Irreflexive, HasProperty),
    ],

}

// -----------------------------------------------------------------------------
// RelationProperty — the Quality that says which structural properties each
// relation type satisfies by definition. Canonical catalog drawn from Tarski
// (1941) + OBO-RO + SKOS specifications.
// -----------------------------------------------------------------------------

/// For each canonical relation type, the set of structural properties it
/// satisfies. `get(relation_type)` returns the list of property concepts
/// that apply. Used by structural-axiom code to know (e.g.) "Opposition
/// is Symmetric, Irreflexive, Involutive".
#[derive(Debug, Clone)]
pub struct RelationProperty;

impl Quality for RelationProperty {
    type Individual = RelationsConcept;
    type Value = Vec<RelationsConcept>;

    fn get(&self, c: &RelationsConcept) -> Option<Vec<RelationsConcept>> {
        use pr4xis::category::{Arrow, Category};
        // DERIVED by querying the loaded `HasProperty` morphisms out of `c` (audit
        // 2026-06-12 D-7) — the relation→structural-property assignment is loaded
        // typed-edge DATA, not a hand-authored Rust match. A relation type with no
        // `HasProperty` edge (e.g. a structural-property concept itself) returns
        // `None`, preserving the prior contract.
        let props: Vec<RelationsConcept> = RelationsCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == RelationsRelationKind::HasProperty && &m.source() == c)
            .map(|m| m.target())
            .collect();
        if props.is_empty() { None } else { Some(props) }
    }
}

// -----------------------------------------------------------------------------
// Domain axioms — separate `impl Axiom` blocks (new `verify` / `axiom_meta!`
// shape per #160 / #167). Each axiom filters
// `RelationsCategory::morphisms()` by relation kind, per the kinded-morphism
// canonical pattern (per_def traits are gone).
// -----------------------------------------------------------------------------

fn relation_has_property(r: RelationsConcept, p: RelationsConcept) -> bool {
    RelationProperty
        .get(&r)
        .map(|props| props.contains(&p))
        .unwrap_or(false)
}

/// The relation kinds the ontology declares `Reflexive` — DERIVED from the typed
/// `(R, Reflexive, HasProperty)` morphisms via [`RelationProperty`] (the same
/// loaded-edge query the structural axioms use), returned as the [`ConceptRef`]s
/// keyed in the `Relations` vocabulary the runtime closure keys on. No committed
/// cache, no hardcoded list: a kind is reflexive iff the ontology SAYS so —
/// Subsumption / Equivalence / Similarity, but NOT the `Irreflexive` Parthood.
/// Read by `ComposedReasoner` so a relational self-query `reaches(c, a, kind)`
/// with `c == a` holds only for a reflexive kind (OWL-RL `prp-rfp`:
/// `Reflexive(p) → p(x, x)`; the strict closure handles the rest).
pub fn reflexive_relation_kinds() -> BTreeSet<ConceptRef> {
    use pr4xis::category::{Concept, FinitelyGenerated};
    RelationsConcept::variants()
        .into_iter()
        .filter(|c| relation_has_property(*c, RelationsConcept::Reflexive))
        .map(|c| relations_kind(c.name()))
        .collect()
}

/// The relation kinds carrying the `Antisymmetric` structural property (Tarski
/// 1941; loaded from this ontology's `HasProperty` edges, never hardcoded).
/// A reasoner uses this to license a *provable* negation: for an antisymmetric
/// `R`, `A R B ∧ A ≠ B ⇒ ¬(B R A)` — so "is a law a statute" is a real No,
/// not an abstention, once "a statute is a law" holds.
pub fn antisymmetric_relation_kinds() -> BTreeSet<ConceptRef> {
    use pr4xis::category::{Concept, FinitelyGenerated};
    RelationsConcept::variants()
        .into_iter()
        .filter(|c| relation_has_property(*c, RelationsConcept::Antisymmetric))
        .map(|c| relations_kind(c.name()))
        .collect()
}

/// The `Opposition` relation kind (SKOS `related` with polarity; Saussure 1916;
/// Cruse 1986) — symmetric and irreflexive, so a *direct* opposition edge is the
/// only thing that licenses a provable negation on the disjointness axis. A
/// reasoner checks a single edge (Opposition is non-transitive), never a closure.
pub fn opposition_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::Opposition.name())
}

/// The `Parthood` relation kind (OBO-RO `part_of`; Casati & Varzi 1999) —
/// antisymmetric and transitive (see [`ParthoodIsTransitive`]), so unlike
/// [`opposition_relation_kind`] a reasoner over this kind answers through a
/// multi-hop reachability engine, not a single-edge check.
pub fn parthood_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::Parthood.name())
}

/// The `Similarity` relation kind (Tversky 1977 features of similarity) —
/// symmetric, non-transitive, componential rather than hierarchical.
pub fn similarity_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::Similarity.name())
}

/// The `Equivalence` relation kind (SKOS `exactMatch`) — symmetric,
/// reflexive, transitive; forms a groupoid.
pub fn equivalence_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::Equivalence.name())
}

/// The `Member-of` relation kind (Smith et al. OBO-RO `member_of`; SKOS
/// `skos:member` inverse) — irreflexive, not symmetric, not transitive at
/// this kind alone (see [`RelationsConcept::MemberOf`]'s own label for the
/// full citation). Also the reasoner-side vocabulary for Searle (1995),
/// *The Construction of Social Reality*, Free Press's institutional
/// "X counts as Y in context C" formula and Jones & Sergot (1996), "A
/// Formal Characterisation of Institutionalised Power", *Logic Journal of
/// the IGPL* 4(3):427-443's conditional counts-as operator — an
/// individual's institutional classification is exactly the
/// individual-to-classification relation `MemberOf` already grounds, not a
/// distinct kind.
pub fn member_of_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::MemberOf.name())
}

/// The `Supersession` relation kind (DCMI Metadata Terms `dcterms:replaces`/
/// `dcterms:isReplacedBy` — see [`RelationsConcept::Supersession`]'s own
/// label for the full citation) — directional and irreflexive; a reasoner
/// checks a single edge (non-transitive at this kind alone), never a
/// closure, the same discipline [`opposition_relation_kind`] documents for
/// its own non-transitive kind.
pub fn supersession_relation_kind() -> ConceptRef {
    use pr4xis::category::Concept;
    relations_kind(RelationsConcept::Supersession.name())
}

/// The transitivity license for a relation kind — the *rule* that authorizes a
/// multi-hop answer over that kind, read from THIS ontology as data: the ontolex
/// label of the `Transitive` structural property, and the citation grounding the
/// kind's transitivity. `Some` iff the kind declares `Transitive(R)` via its
/// `(R, Transitive, HasProperty)` edge (so a single Similarity/Association hop —
/// non-transitive — yields `None`, and a chain over it invokes no transitivity).
///
/// The citation is read off the kind's dedicated transitivity axiom
/// ([`SubsumptionIsTransitive`] → Tarski 1941; [`ParthoodIsTransitive`] → Casati
/// & Varzi 1999) — a first-class cited surface, never a literal in the caller. A
/// transitive kind with no dedicated citation axiom yet returns the property name
/// with an empty citation, so a caller can still surface "R is transitive" and
/// treat the citation as a documented follow-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitivityLicense {
    /// The `Transitive` structural property's ontolex label ("Transitive").
    pub property: String,
    /// The citation grounding this kind's transitivity, from its axiom — empty
    /// when no dedicated transitivity axiom cites this kind yet.
    pub citation: String,
}

/// The transitivity license for the relation kind named `kind_name` (matched
/// against the Relations concept names, e.g. `"Subsumption"` / `"Parthood"`).
/// See [`TransitivityLicense`].
pub fn transitivity_license(kind_name: &str) -> Option<TransitivityLicense> {
    use pr4xis::category::{Concept, FinitelyGenerated};
    let concept = RelationsConcept::variants()
        .into_iter()
        .find(|c| c.name() == kind_name)?;
    if !relation_has_property(concept, RelationsConcept::Transitive) {
        return None;
    }
    let property = RelationsConcept::Transitive
        .lexical()
        .map(|l| l.label.as_str().to_string())
        .unwrap_or_else(|| RelationsConcept::Transitive.name().to_string());
    let citation = match concept {
        RelationsConcept::Subsumption => SubsumptionIsTransitive.citation().as_str().to_string(),
        RelationsConcept::Parthood => ParthoodIsTransitive.citation().as_str().to_string(),
        // Transitive per the catalog, but no dedicated citation axiom wired yet:
        // surface the named property; the citation is a documented follow-up.
        _ => String::new(),
    };
    Some(TransitivityLicense { property, citation })
}

fn kinded_edge_exists(
    from: RelationsConcept,
    to: RelationsConcept,
    kind: RelationsRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    RelationsCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

fn direct_children_of(parent: RelationsConcept) -> Vec<RelationsConcept> {
    use pr4xis::category::{Arrow, Category};
    RelationsCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == RelationsRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

/// Aristotle / Saussure / Tarski — Opposition is symmetric.
pub struct OppositionIsSymmetric;

impl Axiom for OppositionIsSymmetric {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::Opposition, RelationsConcept::Symmetric) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OppositionIsSymmetric",
        "RelationProperty catalog declares Opposition as Symmetric (Tarski 1941): (A R B) \u{21d2} (B R A)",
        "Aristotle Peri Hermeneias; Saussure (1916); Tarski (1941) J. Symbolic Logic 6"
    );
}

/// Guarino / Tarski — Subsumption is antisymmetric.
pub struct SubsumptionIsAntisymmetric;

impl Axiom for SubsumptionIsAntisymmetric {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(
            RelationsConcept::Subsumption,
            RelationsConcept::Antisymmetric,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SubsumptionIsAntisymmetric",
        "RelationProperty catalog declares Subsumption as Antisymmetric (Tarski 1941): (A R B) \u{2227} (B R A) \u{21d2} A = B",
        "Guarino (2009) The Ontological Level; Tarski (1941) J. Symbolic Logic 6"
    );
}

/// Guarino / Tarski — Subsumption is transitive. The structural property that
/// LICENSES a multi-hop is-a chain: `A ⊑ B ∧ B ⊑ C ⇒ A ⊑ C`. Read by the chat
/// engine (via [`transitivity_license`]) to surface the *rule* that authorized a
/// transitive answer, not just the witness chain.
pub struct SubsumptionIsTransitive;

impl Axiom for SubsumptionIsTransitive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::Subsumption, RelationsConcept::Transitive) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SubsumptionIsTransitive",
        "RelationProperty catalog declares Subsumption as Transitive (Tarski 1941): (A R B) \u{2227} (B R C) \u{21d2} (A R C) — the property that licenses a multi-hop is-a chain",
        "Tarski (1941) Calculus of Relations, J. Symbolic Logic 6"
    );
}

/// Casati & Varzi / OBO-RO — Parthood is transitive. A part of a part is a part
/// of the whole: `A part-of B ∧ B part-of C ⇒ A part-of C`. The licensing rule
/// for a multi-hop mereological chain (`clause → section → title`).
pub struct ParthoodIsTransitive;

impl Axiom for ParthoodIsTransitive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::Parthood, RelationsConcept::Transitive) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParthoodIsTransitive",
        "RelationProperty catalog declares Parthood as Transitive (Casati & Varzi 1999; OBO-RO): a part of a part is a part of the whole",
        "Casati & Varzi (1999) Parts and Places; Smith et al. (2005) Genome Biology 6:R46 OBO-RO"
    );
}

/// Lewis / Reichenbach — Causation is irreflexive (strict).
pub struct CausationIsAsymmetric;

impl Axiom for CausationIsAsymmetric {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::Causation, RelationsConcept::Irreflexive) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CausationIsAsymmetric",
        "RelationProperty catalog declares Causation as Irreflexive (strict; combined with Antisymmetric gives asymmetric in Tarski's sense)",
        "Lewis (1973) Causation; Reichenbach (1956) Direction of Time"
    );
}

/// OBO-RO — MemberOf is irreflexive: nothing is a member of itself.
pub struct MemberOfIsIrreflexive;

impl Axiom for MemberOfIsIrreflexive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::MemberOf, RelationsConcept::Irreflexive) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MemberOfIsIrreflexive",
        "RelationProperty catalog declares MemberOf as Irreflexive (Tarski 1941): \u{ac}(A R A) for any A — a classification is not a member of itself",
        "Smith et al. (2005) Genome Biology 6:R46 OBO-RO `member_of`; Tarski (1941) J. Symbolic Logic 6"
    );
}

/// CiTO / Tarski — Cites is irreflexive: a provision does not cite itself.
pub struct CitesIsIrreflexive;

impl Axiom for CitesIsIrreflexive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if relation_has_property(RelationsConcept::Cites, RelationsConcept::Irreflexive) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CitesIsIrreflexive",
        "RelationProperty catalog declares Cites as Irreflexive (Tarski 1941): \u{ac}(A R A) for any A — a provision does not cite itself (CiTO `cito:cites` between distinct bibliographic entities)",
        "Peroni & Shotton (2012) FaBiO and CiTO, J. Web Semantics 17 `cito:cites`; Tarski (1941) J. Symbolic Logic 6"
    );
}

/// Noonan / Varzi — Parthood is distinct from Subsumption.
pub struct ParthoodIsDistinctFromSubsumption;

impl Axiom for ParthoodIsDistinctFromSubsumption {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            RelationsConcept::Parthood,
            RelationsConcept::Subsumption,
            RelationsRelationKind::DistinctFrom,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParthoodIsDistinctFromSubsumption",
        "(Parthood, Subsumption, DistinctFrom) edge encodes the Varzi/Noonan point: parthood and subsumption are genuinely different relations",
        "Noonan (2003); Varzi (2007) Spatial Reasoning and Ontology"
    );
}

/// SKOS broader / narrower — Subsumption and Specialisation are inverses.
pub struct SubsumptionSpecialisationAreInverses;

impl Axiom for SubsumptionSpecialisationAreInverses {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let fwd = kinded_edge_exists(
            RelationsConcept::Subsumption,
            RelationsConcept::Specialisation,
            RelationsRelationKind::InverseOf,
        );
        let rev = kinded_edge_exists(
            RelationsConcept::Specialisation,
            RelationsConcept::Subsumption,
            RelationsRelationKind::InverseOf,
        );
        if fwd && rev {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SubsumptionSpecialisationAreInverses",
        "Subsumption \u{2194} Specialisation: InverseOf edges in both directions encode the SKOS broader/narrower inverse pair",
        "SKOS (W3C 2009) \u{00a7}8.6.3"
    );
}

/// OBO-RO + SKOS + CiTO + DCMI — thirteen canonical binary relation types.
pub struct ThirteenCanonicalRelationTypes;

impl Axiom for ThirteenCanonicalRelationTypes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let expected = [
            RelationsConcept::Subsumption,
            RelationsConcept::Parthood,
            RelationsConcept::Causation,
            RelationsConcept::Opposition,
            RelationsConcept::Similarity,
            RelationsConcept::Precedence,
            RelationsConcept::Equivalence,
            RelationsConcept::Specialisation,
            RelationsConcept::Dependence,
            RelationsConcept::Association,
            RelationsConcept::MemberOf,
            RelationsConcept::Cites,
            RelationsConcept::Supersession,
        ];
        let actual = direct_children_of(RelationsConcept::RelationType);
        let ok = actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ThirteenCanonicalRelationTypes",
        "direct children of RelationType are exactly the thirteen OBO-RO + SKOS + CiTO + DCMI binary relation types: Subsumption, Parthood, Causation, Opposition, Similarity, Precedence, Equivalence, Specialisation, Dependence, Association, MemberOf, Cites, Supersession",
        "Smith et al. (2005) Genome Biology 6:R46 OBO-RO; SKOS (W3C 2009); Peroni & Shotton (2012) CiTO; DCMI Metadata Terms (Dublin Core, 2020)"
    );
}

/// Tarski (1941) — seven algebraic structural properties.
pub struct SevenStructuralProperties;

impl Axiom for SevenStructuralProperties {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let expected = [
            RelationsConcept::Symmetric,
            RelationsConcept::Antisymmetric,
            RelationsConcept::Transitive,
            RelationsConcept::Reflexive,
            RelationsConcept::Irreflexive,
            RelationsConcept::Functional,
            RelationsConcept::Involutive,
        ];
        let actual = direct_children_of(RelationsConcept::StructuralProperty);
        let ok = actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SevenStructuralProperties",
        "direct children of StructuralProperty are exactly the seven Tarski (1941) algebraic properties: Symmetric, Antisymmetric, Transitive, Reflexive, Irreflexive, Functional, Involutive",
        "Tarski (1941) Calculus of Relations, J. Symbolic Logic 6"
    );
}

impl Ontology for RelationsOntology {
    type Cat = RelationsCategory;
    type Qual = RelationProperty;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(OppositionIsSymmetric));
        axioms.push(Box::new(SubsumptionIsAntisymmetric));
        axioms.push(Box::new(SubsumptionIsTransitive));
        axioms.push(Box::new(ParthoodIsTransitive));
        axioms.push(Box::new(CausationIsAsymmetric));
        axioms.push(Box::new(MemberOfIsIrreflexive));
        axioms.push(Box::new(CitesIsIrreflexive));
        axioms.push(Box::new(ParthoodIsDistinctFromSubsumption));
        axioms.push(Box::new(SubsumptionSpecialisationAreInverses));
        axioms.push(Box::new(ThirteenCanonicalRelationTypes));
        axioms.push(Box::new(SevenStructuralProperties));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<RelationsCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RelationsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_relation_type_has_properties() {
        let role = RelationProperty;
        use RelationsConcept as R;
        for rt in [
            R::Subsumption,
            R::Parthood,
            R::Causation,
            R::Opposition,
            R::Similarity,
            R::Precedence,
            R::Equivalence,
            R::Specialisation,
            R::Dependence,
            R::Association,
            R::MemberOf,
            R::Cites,
        ] {
            let props = role.get(&rt);
            assert!(
                props.is_some() && !props.unwrap().is_empty(),
                "relation type {:?} has no declared structural properties",
                rt
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn thirteen_relation_types_axiom_holds() {
        assert!(ThirteenCanonicalRelationTypes.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_structural_properties_axiom_holds() {
        assert!(SevenStructuralProperties.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn opposition_is_symmetric_holds() {
        assert!(OppositionIsSymmetric.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subsumption_is_antisymmetric_holds() {
        assert!(SubsumptionIsAntisymmetric.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subsumption_is_transitive_holds() {
        assert!(SubsumptionIsTransitive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parthood_is_transitive_holds() {
        assert!(ParthoodIsTransitive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn member_of_is_irreflexive_holds() {
        assert!(MemberOfIsIrreflexive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cites_is_irreflexive_holds() {
        assert!(CitesIsIrreflexive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn transitivity_license_reads_the_property_and_citation_as_data() {
        // The licensing rule surfaced for a multi-hop answer is READ from this
        // ontology: Subsumption's transitivity carries the `Transitive` property
        // label + the Tarski (1941) citation off its axiom; Parthood carries the
        // Casati & Varzi citation; a non-transitive kind (Opposition) has no
        // license — so the chat never fabricates a transitivity note.
        let sub = super::transitivity_license("Subsumption").expect("Subsumption is transitive");
        assert_eq!(sub.property, "Transitive");
        assert!(
            sub.citation.contains("Tarski"),
            "Subsumption transitivity cites Tarski; got {:?}",
            sub.citation
        );
        let part = super::transitivity_license("Parthood").expect("Parthood is transitive");
        assert!(
            part.citation.contains("Casati"),
            "Parthood transitivity cites Casati & Varzi; got {:?}",
            part.citation
        );
        assert!(
            super::transitivity_license("Opposition").is_none(),
            "Opposition is not transitive — no license"
        );
    }

    /// Regenerate the committed `relations_transitive_kinds.txt` that the `ontology!`
    /// macro (`pr4xis-derive`) reads at expansion — the SINGLE sanctioned proc-macro
    /// projection of this ontology's `(R, Transitive, HasProperty)` edges. `#[ignore]`d
    /// (it WRITES, asserting nothing). Run by hand when a `Transitive` edge is
    /// added/removed above:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_relations_transitive_kinds_cache`.
    /// (The runtime no longer keeps a copy — it derives the set from the loaded
    /// `morphism_kinds.prx`, regenerated by `regenerate_morphism_kinds_prx`.)
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_relations_transitive_kinds_cache() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::ontology::{materialize, transitive_kinds};

        let archive = pr4xis_runtime::emit::emit_kind_vocabulary::<RelationsCategory>();
        let relations =
            materialize(archive, OntologyName::new_static("Relations")).expect("materializes");
        let mut names: Vec<String> = transitive_kinds(&relations)
            .iter()
            .map(|c| c.name.clone())
            .collect();
        names.sort();
        let body = format!("{}\n", names.join("\n"));
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../pr4xis-derive/src/relations_transitive_kinds.txt"
            ),
            &body,
        )
        .expect("write relations_transitive_kinds.txt");
    }

    /// Drift guard (normal suite) for the committed `relations_transitive_kinds.txt`
    /// that `pr4xis-derive` reads at expansion — the SINGLE sanctioned proc-macro
    /// projection of this ontology's `(R, Transitive, HasProperty)` edges. It must
    /// equal `transitive_kinds()` read off the emitted+materialized Relations
    /// archive; a `Transitive` edge added/removed without regenerating, or a
    /// hand-edit, FAILS here. The runtime's copy was REMOVED — the kernel derives the
    /// set from the loaded `morphism_kinds.prx`, guarded by
    /// `morphism_kinds_prx_matches_the_relations_ontology` (loaded == fresh emit ⟹
    /// the runtime's transitive projection equals this same authority; the runtime's
    /// own reachability tests exercise the derivation live).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn relations_transitive_kinds_cache_matches_the_relations_ontology() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::ontology::{materialize, transitive_kinds};

        let archive = pr4xis_runtime::emit::emit_kind_vocabulary::<RelationsCategory>();
        let relations =
            materialize(archive, OntologyName::new_static("Relations")).expect("materializes");
        let declared: alloc::collections::BTreeSet<String> = transitive_kinds(&relations)
            .iter()
            .map(|c| c.name.clone())
            .collect();

        let cached: alloc::collections::BTreeSet<String> =
            include_str!("../../../../pr4xis-derive/src/relations_transitive_kinds.txt")
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect();
        assert_eq!(
            cached, declared,
            "pr4xis-derive's relations_transitive_kinds.txt is STALE — regenerate with \
             `cargo test -p pr4xis-domains -- --ignored regenerate_relations_transitive_kinds_cache`"
        );
    }

    /// Regenerate the committed `morphism_kinds.prx` — the FULL relation-kind
    /// vocabulary (every kind WITH its `HasProperty` and inter-kind edges) the
    /// runtime folds into its default morphism-kind vocab, so a serialized morphism
    /// carries the kind's ADDRESS, not its name (A3). Where the transitive cache is
    /// just the transitive NAME list, this is the whole emitted archive — each
    /// kind's structural meaning travels. `#[ignore]`d (it WRITES). Run when a
    /// relation kind or a `HasProperty`/inter-kind edge changes above:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_morphism_kinds_prx`.
    /// Then update `MORPHISM_KINDS_ROOT_HEX` in `pr4xis-runtime` to the printed root.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_morphism_kinds_prx() {
        let archive = pr4xis_runtime::emit::emit_kind_vocabulary::<RelationsCategory>();
        let bytes = pr4xis_runtime::load::emit(&archive).expect("emit morphism_kinds.prx bytes");
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../pr4xis-runtime/src/morphism_kinds.prx"
            ),
            &bytes,
        )
        .expect("write morphism_kinds.prx");
        println!(
            "MORPHISM_KINDS_ROOT_HEX = {}",
            archive.root().expect("archive roots").to_hex()
        );
    }

    /// Drift guard (normal suite) for the committed `morphism_kinds.prx` — the
    /// relation-kind vocabulary the runtime's default morphism-kind vocab loads. The
    /// committed projection must deserialize, fail-closed against the LIVE Relations
    /// root, to the SAME archive a fresh `emit_kind_vocabulary::<RelationsCategory>()`
    /// produces; a
    /// kind or a `HasProperty`/inter-kind edge changed without regenerating FAILS
    /// here (closing the rule-7 second-declaration gap, as the transitive cache does).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn morphism_kinds_prx_matches_the_relations_ontology() {
        let fresh = pr4xis_runtime::emit::emit_kind_vocabulary::<RelationsCategory>();
        let committed: &[u8] = include_bytes!("../../../../pr4xis-runtime/src/morphism_kinds.prx");
        let loaded = pr4xis_runtime::load::load(committed, fresh.root().expect("roots")).expect(
            "committed morphism_kinds.prx is STALE — regenerate with \
             `cargo test -p pr4xis-domains -- --ignored regenerate_morphism_kinds_prx`",
        );
        assert_eq!(
            loaded, fresh,
            "the committed morphism_kinds.prx must equal the emitted Relations archive"
        );
    }

    /// The reflexive relation kinds are DERIVED from this ontology's
    /// `(R, Reflexive, HasProperty)` declarations (no committed cache): Subsumption,
    /// Equivalence, Similarity are reflexive; Parthood (declared `Irreflexive`) is
    /// NOT. This is the `ComposedReasoner`'s source for the `reaches` `c == a`
    /// short-circuit, so it must track exactly what the ontology declares.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reflexive_relation_kinds_are_derived_from_the_declarations() {
        let reflexive: alloc::collections::BTreeSet<String> = super::reflexive_relation_kinds()
            .iter()
            .map(|c| c.name.clone())
            .collect();
        assert!(
            reflexive.contains("Subsumption"),
            "Subsumption is reflexive"
        );
        assert!(
            reflexive.contains("Equivalence"),
            "Equivalence is reflexive"
        );
        assert!(
            !reflexive.contains("Parthood"),
            "Parthood is Irreflexive — must not be in the reflexive set"
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn relations_prx_carries_the_loaded_transitivity_the_runtime_reads() {
        // The "code is ontological" fix (doc §11), Step 1: the transitive
        // relation kinds the runtime closure folds are LOADED from THIS ontology
        // — never a hardcoded `RelationKind::transitive()` array. Emit Relations
        // to a `.prx` [`Archive`], materialize it, and assert
        // `transitive_kinds` recovers exactly the kinds this ontology declares
        // `Transitive(R)` for via the `(R, Transitive, HasProperty)` edges above.
        // This is the runtime mirror of [`RelationProperty::get`], which makes
        // the SAME query typed over the compiled `Category`.
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::definition::EdgeTarget;
        use pr4xis_runtime::ontology::{ConceptRef, materialize, transitive_kinds};

        let archive = pr4xis_runtime::emit::emit_kind_vocabulary::<RelationsCategory>();

        // The loaded `.prx` data itself carries `(Subsumption, Transitive,
        // HasProperty)` — the assertion `RelationProperty::get` reads typed at
        // compile time, now visible as ordinary edge data on the wire.
        let subsumption = archive
            .nodes
            .iter()
            .find(|n| n.name == "Subsumption")
            .expect("Subsumption node emitted into Relations.prx");
        assert!(
            subsumption
                .edges
                .iter()
                .any(|(rel, target)| rel == "HasProperty"
                    && matches!(target, EdgeTarget::Local(p) if p == "Transitive")),
            "Relations.prx must carry the loaded (Subsumption, Transitive, HasProperty) edge; \
             got edges {:?}",
            subsumption.edges
        );

        let relations = materialize(archive, OntologyName::new_static("Relations"))
            .expect("Relations materializes");
        let kinds = transitive_kinds(&relations);

        let relations_id = OntologyName::new_static("Relations");
        let kind = |name: &str| ConceptRef::new(relations_id.clone(), name);

        // Exactly the seven kinds this ontology declares Transitive(R) for
        // (Tarski 1941 catalog; the `HasProperty` edges in the `edges:` clause).
        for transitive in [
            "Subsumption",
            "Parthood",
            "Causation",
            "Precedence",
            "Equivalence",
            "Specialisation",
            "Dependence",
        ] {
            assert!(
                kinds.contains(&kind(transitive)),
                "{transitive} declares Transitive(R) → must be a loaded transitive kind; got {kinds:?}"
            );
        }
        // The non-transitive relation types are excluded (symmetric / associative
        // only), as is the `Transitive` structural-property concept itself.
        for non_transitive in ["Opposition", "Similarity", "Association", "Transitive"] {
            assert!(
                !kinds.contains(&kind(non_transitive)),
                "{non_transitive} is not transitive → must be excluded; got {kinds:?}"
            );
        }
        assert_eq!(
            kinds.len(),
            7,
            "exactly the seven declared transitive kinds; got {kinds:?}"
        );
    }
}
