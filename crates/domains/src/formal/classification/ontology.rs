//! Classification — the theory of kinds, taxa, and ranks (issue #152).
//!
//! The *richer* vocabulary behind domain ontologies' `is_a:` clause.
//! Where ontologies want to talk about what KIND of thing something
//! is, what rank it holds, and how ranks subordinate — this supplies
//! the terms grounded in classification theory.
//!
//! Four literature lineages:
//!
//! 1. **Ontological-level classification** — Guarino (2009)
//!    "The Ontological Level: Revisiting 30 Years of Knowledge
//!    Representation", in Borgo & Lesmo (eds) *Formal Ontology in
//!    Information Systems*; Guarino & Welty (2002) "Evaluating
//!    Ontological Decisions with OntoClean". Source of `Kind` as the
//!    substantial-sortal foundation and `Category` in the rigid-class
//!    sense.
//!
//! 2. **Species problem** — Ghiselin (1974) "A Radical Solution to
//!    the Species Problem", Systematic Zoology 23; Hull (1978) "A
//!    Matter of Individuality", Philosophy of Science 45. Source of
//!    `Individual` as the species-are-individuals move that troubles
//!    classical classification.
//!
//! 3. **Taxonomic ranks** — Linnaeus (1735) *Systema Naturae*, 1st ed.,
//!    Leiden; modern systematics (ICZN / ICNafp codes). Source of the
//!    seven classical ranks: Kingdom / Phylum / Class / Order / Family
//!    / Genus / Species.
//!
//! 4. **Critique of rigid classification** — Ereshefsky (2001) *The
//!    Poverty of the Linnaean Hierarchy*, Cambridge UP. Argues
//!    ranks-as-categories fail and the Linnaean system should be
//!    abandoned; motivates keeping `Rank` as a *meta-concept* rather
//!    than a rigid hierarchy.
//!
//! 5. **Differentia** — Aristotle *Categories* (c. 350 BCE);
//!    Porphyry *Isagoge* (c. 270 CE); medieval scholastic tradition.
//!    Source of `Differentia`: the feature distinguishing a species
//!    within its genus.
//!
//! Source: Guarino (2009); Guarino & Welty (2002) Comm. ACM; Ghiselin
//! (1974); Hull (1978); Linnaeus (1735); Ereshefsky (2001); Aristotle
//! Categories; Porphyry Isagoge.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Classification",
    source: "Guarino (2009); Guarino & Welty (2002); Ghiselin (1974) Syst. Zool. 23; Hull (1978) Phil. Sci. 45; Linnaeus (1735) Systema Naturae; Ereshefsky (2001) Poverty of Linnaean Hierarchy; Aristotle Categories",
    being: AbstractObject,

    concepts: [
        // === Ontological-level classification ===
        Kind,
        Category,
        Taxon,

        // === Linnaean ranks (seven classical, in subordination order) ===
        Species,
        Genus,
        Family,
        Order,
        Class,
        Phylum,
        Kingdom,

        // === Meta-concepts ===
        Rank,
        Differentia,
        Individual,
    ],

    labels: {
        Kind: ("en", "Kind",
            "Guarino (2009) OntoClean: a substantial sortal — a rigid, identity-supplying category that natural things fall under. Distinguished from mere Category by carrying identity criteria."),
        Category: ("en", "Category",
            "Guarino & Welty (2002): a rigid classificatory unit. Broader than Kind; every Kind is a Category but not every Category is a Kind (e.g. `Red` is a Category but not a Kind — redness doesn't supply identity)."),
        Taxon: ("en", "Taxon",
            "Systematic biology: a named classificatory unit at any Rank. The species *Homo sapiens* is a taxon; the genus *Homo* is a taxon; the family Hominidae is a taxon. Contrast Kind (abstract) vs Taxon (named-and-placed)."),

        Species: ("en", "Species",
            "Linnaean Rank 1 (lowest). Ghiselin (1974) / Hull (1978): arguably an Individual rather than a Kind — species have spatiotemporal boundaries, reproduce, go extinct. The species problem."),
        Genus: ("en", "Genus",
            "Linnaean Rank 2. A Taxon subsuming one or more Species sharing a Differentia. *Homo* is the Genus subsuming *Homo sapiens*, *Homo neanderthalensis*, etc."),
        Family: ("en", "Family",
            "Linnaean Rank 3. Subsumes related Genera. *Hominidae* (great apes + humans)."),
        Order: ("en", "Order",
            "Linnaean Rank 4. Subsumes related Families. *Primates*."),
        Class: ("en", "Class",
            "Linnaean Rank 5. Subsumes related Orders. *Mammalia*. (Confusable with OOP 'class' or logical 'class' — this is the taxonomic Class.)"),
        Phylum: ("en", "Phylum",
            "Linnaean Rank 6. Subsumes related Classes. *Chordata*."),
        Kingdom: ("en", "Kingdom",
            "Linnaean Rank 7 (highest traditional). Subsumes related Phyla. *Animalia*, *Plantae*, *Fungi*, etc."),

        Rank: ("en", "Rank",
            "Meta-concept: what Species, Genus, Family, etc. ARE instances of. Ereshefsky (2001) argues ranks lack biological reality but retain pragmatic communicative value. In pr4xis we keep `Rank` as a meta-concept; whether specific ranks are realist is a downstream choice."),
        Differentia: ("en", "Differentia",
            "Aristotle *Categories* (c. 350 BCE) / Porphyry *Isagoge*: the feature that distinguishes a Species within its Genus. 'Rational' is the Differentia of *Homo sapiens* within the Genus of animal Kinds."),
        Individual: ("en", "Individual",
            "Ghiselin (1974) / Hull (1978): an entity with spatiotemporal boundaries and parts rather than members. Argued to be the correct ontological category for biological species, complicating the received classification picture."),
    },

    is_a: [
        // Kind ⊑ Category (every Kind is a Category; Kinds add identity criteria)
        (Kind, Category),

        // All seven Linnaean ranks are Taxa when they name actual groups.
        (Species, Taxon),
        (Genus, Taxon),
        (Family, Taxon),
        (Order, Taxon),
        (Class, Taxon),
        (Phylum, Taxon),
        (Kingdom, Taxon),
    ],

    edges: [
        // Linnaean subordination chain (Species at bottom, Kingdom at top).
        (Species, Genus, SubordinateTo),
        (Genus, Family, SubordinateTo),
        (Family, Order, SubordinateTo),
        (Order, Class, SubordinateTo),
        (Class, Phylum, SubordinateTo),
        (Phylum, Kingdom, SubordinateTo),

        // Ranks are instances of Rank (meta-relation)
        (Species, Rank, InstanceOf),
        (Genus, Rank, InstanceOf),
        (Family, Rank, InstanceOf),
        (Order, Rank, InstanceOf),
        (Class, Rank, InstanceOf),
        (Phylum, Rank, InstanceOf),
        (Kingdom, Rank, InstanceOf),

        // Differentia distinguishes Species within Genus
        (Differentia, Species, Distinguishes),
        (Species, Genus, WithinBy),

        // Individual is the Ghiselin-Hull alternative view of Species
        (Species, Individual, MayBe),

        // Taxon is a named Category
        (Taxon, Category, Is),
    ],

    axioms: {
        SevenLinnaeanRanks: {
            source: "Linnaeus (1735) Systema Naturae — seven classical ranks",
            description: "the direct children of Taxon are exactly the seven Linnaean ranks: Species, Genus, Family, Order, Class, Phylum, Kingdom",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                let rels = ClassificationTaxonomy::relations();
                let expected = [
                    ClassificationConcept::Species,
                    ClassificationConcept::Genus,
                    ClassificationConcept::Family,
                    ClassificationConcept::Order,
                    ClassificationConcept::Class,
                    ClassificationConcept::Phylum,
                    ClassificationConcept::Kingdom,
                ];
                let actual: Vec<_> = rels
                    .iter()
                    .filter_map(|(c, p)| if *p == ClassificationConcept::Taxon { Some(*c) } else { None })
                    .collect();
                actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
            },
        },
        KindIsCategory: {
            source: "Guarino (2009) OntoClean — Kind specialises Category by carrying identity",
            description: "Kind is declared as a specialisation of Category (via is_a), encoding the OntoClean distinction that every Kind is a Category but Kinds additionally supply identity criteria",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                ClassificationTaxonomy::relations().iter().any(|(c, p)| {
                    *c == ClassificationConcept::Kind && *p == ClassificationConcept::Category
                })
            },
        },
        LinnaeanSubordinationChain: {
            source: "Linnaeus (1735); modern systematics (ICZN)",
            description: "the Linnaean ranks form an ascending chain: Species → Genus → Family → Order → Class → Phylum → Kingdom, connected by SubordinateTo edges",
            holds: {
                use pr4xis::category::Category;
                let morphs = ClassificationCategory::morphisms();
                let has = |from: ClassificationConcept, to: ClassificationConcept| {
                    morphs.iter().any(|r| {
                        r.from == from
                            && r.to == to
                            && r.kind == ClassificationRelationKind::SubordinateTo
                    })
                };
                has(ClassificationConcept::Species, ClassificationConcept::Genus)
                    && has(ClassificationConcept::Genus, ClassificationConcept::Family)
                    && has(ClassificationConcept::Family, ClassificationConcept::Order)
                    && has(ClassificationConcept::Order, ClassificationConcept::Class)
                    && has(ClassificationConcept::Class, ClassificationConcept::Phylum)
                    && has(ClassificationConcept::Phylum, ClassificationConcept::Kingdom)
            },
        },
        DifferentiaDistinguishesSpecies: {
            source: "Aristotle Categories; Porphyry Isagoge",
            description: "the edge (Differentia, Species, Distinguishes) exists, encoding the Porphyrian tree: within a Genus, a Differentia picks out a Species",
            holds: {
                use pr4xis::category::Category;
                ClassificationCategory::morphisms().iter().any(|r| {
                    r.from == ClassificationConcept::Differentia
                        && r.to == ClassificationConcept::Species
                        && r.kind == ClassificationRelationKind::Distinguishes
                })
            },
        },
        SpeciesMayBeIndividual: {
            source: "Ghiselin (1974) Syst. Zool. 23; Hull (1978) Phil. Sci. 45",
            description: "the edge (Species, Individual, MayBe) exists, acknowledging the Ghiselin-Hull thesis that species are spatiotemporal individuals rather than abstract Kinds — a live debate in philosophy of biology",
            holds: {
                use pr4xis::category::Category;
                ClassificationCategory::morphisms().iter().any(|r| {
                    r.from == ClassificationConcept::Species
                        && r.to == ClassificationConcept::Individual
                        && r.kind == ClassificationRelationKind::MayBe
                })
            },
        },
    },
}

// -----------------------------------------------------------------------------
// ClassificationLineage — tag each concept with its literature origin.
// -----------------------------------------------------------------------------

/// Quality: which literature tradition introduces each concept?
#[derive(Debug, Clone)]
pub struct ClassificationLineage;

impl Quality for ClassificationLineage {
    type Individual = ClassificationConcept;
    type Value = &'static str;

    fn get(&self, c: &ClassificationConcept) -> Option<&'static str> {
        use ClassificationConcept as C;
        Some(match c {
            C::Kind | C::Category | C::Taxon => "guarino",
            C::Species | C::Genus | C::Family | C::Order | C::Class | C::Phylum | C::Kingdom => {
                "linnaeus"
            }
            C::Rank => "ereshefsky",
            C::Differentia => "aristotle-porphyry",
            C::Individual => "ghiselin-hull",
        })
    }
}

impl Ontology for ClassificationOntology {
    type Cat = ClassificationCategory;
    type Qual = ClassificationLineage;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        ClassificationOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        ClassificationOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ClassificationCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ClassificationOntology::validate().unwrap();
    }

    #[test]
    fn seven_linnaean_ranks_holds() {
        assert!(SevenLinnaeanRanks.holds());
    }

    #[test]
    fn kind_is_category_holds() {
        assert!(KindIsCategory.holds());
    }

    #[test]
    fn linnaean_subordination_chain_holds() {
        assert!(LinnaeanSubordinationChain.holds());
    }

    #[test]
    fn differentia_distinguishes_species_holds() {
        assert!(DifferentiaDistinguishesSpecies.holds());
    }

    #[test]
    fn species_may_be_individual_holds() {
        assert!(SpeciesMayBeIndividual.holds());
    }
}
