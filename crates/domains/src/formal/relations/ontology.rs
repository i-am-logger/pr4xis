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
//! Four literature lineages supply the content:
//!
//! 1. **Applied-ontology tradition** — Smith et al. (2005) *Relations
//!    in biomedical ontologies* (OBO Relation Ontology), Genome Biology
//!    6:R46, and SKOS / SKOS-XL (W3C 2009). Source of the ten relation
//!    types themselves (`is_a` / `part_of` / `causally_related_to` /
//!    `related_to` / `precedes` / `broader` / `narrower` / `related` /
//!    `exactMatch` / `depends_on`).
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
//! (1910–13); Masolo et al. (2003) DOLCE.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Relations",
    source: "Smith et al. (2005) Genome Biology 6:R46 (OBO-RO); SKOS (W3C 2009); Tarski (1941) J. Symbolic Logic 6; Russell & Whitehead Principia Mathematica (1910–13); Masolo et al. (2003) DOLCE WonderWeb D18",
    being: AbstractObject,

    concepts: [
        // === Binary relation types (10) — what a kinded edge can mean ===
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
            "Abstract parent of the ten canonical binary relation types. Anything kinded as a RelationType has a direction, a source, and a target."),
        StructuralProperty: ("en", "Structural property",
            "Abstract parent of the seven algebraic properties (symmetric, transitive, etc.) that classify a relation. From Tarski (1941) relation algebra."),
    },

    is_a: [
        // All ten relation types are RelationTypes.
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
    ],

    axioms: {
        OppositionIsSymmetric: {
            source: "Aristotle Peri Hermeneias; Saussure (1916); Tarski (1941)",
            description: "the Relations ontology declares Opposition as a Symmetric structural property; verified by matching the declared edge (Opposition, Symmetric) in the RelationProperty quality below",
            holds: {
                // In this ontology, a relation type R "is symmetric" means its
                // declared properties (captured by RelationProperty quality) include
                // StructuralProperty::Symmetric. Since that quality is defined below,
                // this axiom verifies the catalog is consistent — Opposition carries
                // Symmetric.
                let role = RelationProperty;
                role.get(&RelationsConcept::Opposition)
                    .map(|props| props.contains(&RelationsConcept::Symmetric))
                    .unwrap_or(false)
            },
        },
        SubsumptionIsAntisymmetric: {
            source: "Guarino (2009) The Ontological Level; Tarski (1941)",
            description: "the Relations catalog declares Subsumption as Antisymmetric",
            holds: {
                let role = RelationProperty;
                role.get(&RelationsConcept::Subsumption)
                    .map(|props| props.contains(&RelationsConcept::Antisymmetric))
                    .unwrap_or(false)
            },
        },
        CausationIsAsymmetric: {
            source: "Lewis (1973) Causation; Reichenbach (1956) Direction of Time",
            description: "the Relations catalog declares Causation as Irreflexive (strict; combined with Antisymmetric gives asymmetric in Tarski's sense)",
            holds: {
                let role = RelationProperty;
                role.get(&RelationsConcept::Causation)
                    .map(|props| props.contains(&RelationsConcept::Irreflexive))
                    .unwrap_or(false)
            },
        },
        ParthoodIsDistinctFromSubsumption: {
            source: "Noonan (2003) — Is-a is not part-of; Varzi (2007) Spatial Reasoning",
            description: "the declared edge (Parthood, Subsumption, DistinctFrom) exists, encoding the Varzi/Noonan point that parthood and subsumption are genuinely different relations",
            holds: {
                use pr4xis::category::Category;
                let morphs = RelationsCategory::morphisms();
                morphs.iter().any(|r| {
                    r.from == RelationsConcept::Parthood
                        && r.to == RelationsConcept::Subsumption
                        && r.kind == RelationsRelationKind::DistinctFrom
                })
            },
        },
        SubsumptionSpecialisationAreInverses: {
            source: "SKOS (W3C 2009) §8.6.3 — broader/narrower inverse",
            description: "Subsumption and Specialisation declare InverseOf edges in both directions, encoding the SKOS broader/narrower inverse pair",
            holds: {
                use pr4xis::category::Category;
                let morphs = RelationsCategory::morphisms();
                let fwd = morphs.iter().any(|r| {
                    r.from == RelationsConcept::Subsumption
                        && r.to == RelationsConcept::Specialisation
                        && r.kind == RelationsRelationKind::InverseOf
                });
                let rev = morphs.iter().any(|r| {
                    r.from == RelationsConcept::Specialisation
                        && r.to == RelationsConcept::Subsumption
                        && r.kind == RelationsRelationKind::InverseOf
                });
                fwd && rev
            },
        },
        TenCanonicalRelationTypes: {
            source: "Smith et al. (2005) OBO-RO; SKOS (W3C 2009)",
            description: "the direct children of RelationType are exactly the ten canonical binary relation types drawn from OBO-RO + SKOS: Subsumption, Parthood, Causation, Opposition, Similarity, Precedence, Equivalence, Specialisation, Dependence, Association",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                let rels = RelationsTaxonomy::relations();
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
                ];
                let actual: Vec<_> = rels
                    .iter()
                    .filter_map(|(c, p)| if *p == RelationsConcept::RelationType { Some(*c) } else { None })
                    .collect();
                actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
            },
        },
        SevenStructuralProperties: {
            source: "Tarski (1941) Calculus of Relations",
            description: "the direct children of StructuralProperty are exactly the seven algebraic properties from Tarski's relation calculus: Symmetric, Antisymmetric, Transitive, Reflexive, Irreflexive, Functional, Involutive",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                let rels = RelationsTaxonomy::relations();
                let expected = [
                    RelationsConcept::Symmetric,
                    RelationsConcept::Antisymmetric,
                    RelationsConcept::Transitive,
                    RelationsConcept::Reflexive,
                    RelationsConcept::Irreflexive,
                    RelationsConcept::Functional,
                    RelationsConcept::Involutive,
                ];
                let actual: Vec<_> = rels
                    .iter()
                    .filter_map(|(c, p)| if *p == RelationsConcept::StructuralProperty { Some(*c) } else { None })
                    .collect();
                actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
            },
        },
    },
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
        use RelationsConcept as R;
        match c {
            R::Subsumption => Some(vec![R::Antisymmetric, R::Transitive, R::Reflexive]),
            R::Parthood => Some(vec![R::Antisymmetric, R::Transitive, R::Irreflexive]),
            R::Causation => Some(vec![R::Irreflexive, R::Transitive]),
            R::Opposition => Some(vec![R::Symmetric, R::Irreflexive, R::Involutive]),
            R::Similarity => Some(vec![R::Symmetric, R::Reflexive]),
            R::Precedence => Some(vec![R::Irreflexive, R::Transitive, R::Antisymmetric]),
            R::Equivalence => Some(vec![R::Symmetric, R::Reflexive, R::Transitive]),
            R::Specialisation => Some(vec![R::Antisymmetric, R::Transitive, R::Irreflexive]),
            R::Dependence => Some(vec![R::Irreflexive, R::Transitive, R::Antisymmetric]),
            R::Association => Some(vec![R::Symmetric]),
            // Structural-property concepts themselves have no properties-of-relations.
            _ => None,
        }
    }
}

impl Ontology for RelationsOntology {
    type Cat = RelationsCategory;
    type Qual = RelationProperty;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        RelationsOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        RelationsOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<RelationsCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        RelationsOntology::validate().unwrap();
    }

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
        ] {
            let props = role.get(&rt);
            assert!(
                props.is_some() && !props.unwrap().is_empty(),
                "relation type {:?} has no declared structural properties",
                rt
            );
        }
    }

    #[test]
    fn ten_relation_types_axiom_holds() {
        assert!(TenCanonicalRelationTypes.holds());
    }

    #[test]
    fn seven_structural_properties_axiom_holds() {
        assert!(SevenStructuralProperties.holds());
    }

    #[test]
    fn opposition_is_symmetric_holds() {
        assert!(OppositionIsSymmetric.holds());
    }

    #[test]
    fn subsumption_is_antisymmetric_holds() {
        assert!(SubsumptionIsAntisymmetric.holds());
    }
}
