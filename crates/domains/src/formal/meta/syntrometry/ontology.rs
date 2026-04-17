//! Syntrometry ontology — Heim's syntrometric primitives, Phase 1.
//!
//! Encodes the core of Burkhard Heim's *Syntrometrische Maximentelezentrik*
//! (the logical foundation underneath Heim theory proper) as a pr4xis
//! ontology. Reformulated in modern category-theoretic dress following:
//!
//! > "A Modernized Syntrometric Logic: Foundations and Applications" (2025)
//! > <https://heim-theory.com/wp-content/uploads/2025/11/A-Modernized-Syntrometric-Logic-Foundations-and-Applications.pdf>
//! > — §1, §2 (especially §2.2 "The Syntrix as the Category of Leveled Structures").
//!
//! The goal is to *verify* the lineage claim — pr4xis's categorical substrate
//! is claimed to instantiate Heim's syntrometric structure — not to assert it.
//! Phase 1 encodes the distinction primitives (Predicate, Predikatrix,
//! Dialektik, Koordination, Aspekt), the syntrometric structures (Syntrix,
//! SyntrixLevel, Synkolator, Korporator), and the mereological primitive
//! (Part), plus their structural relations. A companion module
//! `lineage_functor` maps Syntrometry → a small pr4xis-substrate ontology
//! and verifies the functor laws.
//!
//! Phase 2 (deferred): telecenters, transzendenzstufen, maximes — the
//! teleological concepts that map to goal-directed planning architecture
//! (see memory: `project_heim_transport.md`).

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Syntrometry",
    source: "Heim (~1980 *Syntrometrische Maximentelezentrik*); modernized 2025 paper on heim-theory.com",
    being: AbstractObject,

    concepts: [
        // === Distinction primitives (§1) ===
        Predicate,
        Predikatrix,
        Dialektik,
        Koordination,
        Aspekt,

        // === Syntrometric structures (§2) ===
        Syntrix,
        SyntrixLevel,
        Synkolator,
        Korporator,

        // === CEM mereology primitive ===
        Part,
    ],

    labels: {
        Predicate: ("en", "Predicate", "The basic distinction — the atomic unit of syntrometric logic. A predicate separates what-is from what-is-not (Spencer-Brown Laws of Form)."),
        Predikatrix: ("en", "Predikatrix", "A structured set of predicates — a predicate-system. Every Predikatrix carries an ordering (Koordination) and an opposition structure (Dialektik)."),
        Dialektik: ("en", "Dialektik", "The binary-opposition structure on a Predikatrix. Pairs of predicates that are mutually exclusive or complementary."),
        Koordination: ("en", "Koordination", "The ordering / coordination mapping between predicates within a Predikatrix. Determines predicate sequences and orientations."),
        Aspekt: ("en", "Aspekt", "Subjective aspect S = [D × K × P]. The product of Dialektik × Koordination × Predikatrix. An observer-relative view of the underlying distinction-system."),

        Syntrix: ("en", "Syntrix", "The Category of Leveled Structures C_SL (§2.2). A hierarchical organisation of Aspekts into levels — the syntrometric analogue of a category."),
        SyntrixLevel: ("en", "Syntrix level", "A single level within a Syntrix — corresponds to an object / grade of abstraction. The Syntrix itself is the tower of levels."),
        Synkolator: ("en", "Synkolator", "An endofunctor on the Syntrix: F: C_SL → C_SL. Maps each level to another level of the same Syntrix, preserving composition."),
        Korporator: ("en", "Korporator", "A structure-mapping functor between Syntrices: K: Syntrix_1 → Syntrix_2. The general case of cross-syntrix composition — Synkolator is the endomorphism specialisation."),

        Part: ("en", "Part", "The mereological part-of primitive Part(A, B). Classical Extensional Mereology (CEM) as used by Heim; must satisfy Weak Supplementation."),
    },

    is_a: [
        // True subsumption only: every Synkolator IS a Korporator, because
        // an endofunctor is a functor specialised to Source = Target
        // (Mac Lane Ch. II §1). Compositional / part-of relationships go
        // into has_a: below.
        (Synkolator, Korporator),
    ],

    has_a: [
        // An Aspekt is constituted from D × K × P (Heim §1). Each of the
        // three is a proper part of every Aspekt instance.
        (Aspekt, Dialektik),
        (Aspekt, Koordination),
        (Aspekt, Predikatrix),

        // A Predikatrix is a structured collection OF Predicates.
        (Predikatrix, Predicate),

        // A Syntrix contains Levels.
        (Syntrix, SyntrixLevel),

        // A SyntrixLevel is a Predikatrix-at-a-given-grade; mereologically
        // it contains the same predicates its parent Predikatrix would.
        (SyntrixLevel, Predicate),
    ],

    edges: [
        // === Predicate composition ===
        (Koordination, Predikatrix, Orders),
        (Dialektik, Predikatrix, StructuresOppositionIn),

        // === Aspekt construction ===
        (Predikatrix, Aspekt, Contributes),
        (Dialektik, Aspekt, Contributes),
        (Koordination, Aspekt, Contributes),

        // === Syntrix hierarchy ===
        (SyntrixLevel, Syntrix, LevelOf),
        (Aspekt, Syntrix, InhabitsLevelOf),

        // === Syntrometric operators ===
        (Synkolator, Syntrix, EndomorphismOn),
        (Korporator, Syntrix, MapsBetween),
    ],
}

/// Quality: syntrometric category each concept belongs to.
#[derive(Debug, Clone)]
pub struct SyntrometryCategoryOf;

impl Quality for SyntrometryCategoryOf {
    type Individual = SyntrometryConcept;
    type Value = &'static str;

    fn get(&self, c: &SyntrometryConcept) -> Option<&'static str> {
        use SyntrometryConcept as S;
        Some(match c {
            S::Predicate | S::Predikatrix | S::Dialektik | S::Koordination | S::Aspekt => {
                "distinction-primitive"
            }
            S::Syntrix | S::SyntrixLevel | S::Synkolator | S::Korporator => {
                "syntrometric-structure"
            }
            S::Part => "mereology",
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Direct mereological parts of a whole (non-transitive).
fn direct_parts_of(whole: SyntrometryConcept) -> Vec<SyntrometryConcept> {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    SyntrometryMereology::relations()
        .into_iter()
        .filter_map(|(w, part)| if w == whole { Some(part) } else { None })
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms — invariants of the syntrometric substrate
// ---------------------------------------------------------------------------

/// Axiom: an Aspekt is the product [Dialektik × Koordination × Predikatrix]
/// (§1 of the modernized paper). The three must all appear as direct
/// mereological parts of Aspekt.
pub struct AspektIsTripleProduct;

impl Axiom for AspektIsTripleProduct {
    fn description(&self) -> &str {
        "Aspekt mereologically contains {Dialektik, Koordination, Predikatrix} (Heim §1)"
    }
    fn holds(&self) -> bool {
        let parts = direct_parts_of(SyntrometryConcept::Aspekt);
        let expected = [
            SyntrometryConcept::Dialektik,
            SyntrometryConcept::Koordination,
            SyntrometryConcept::Predikatrix,
        ];
        expected.iter().all(|e| parts.contains(e))
    }
}

/// Axiom: Synkolator is-a Korporator — every endofunctor is a functor
/// specialised to the same source and target. (Mac Lane Ch. II §1.)
pub struct SynkolatorIsKorporator;

impl Axiom for SynkolatorIsKorporator {
    fn description(&self) -> &str {
        "Synkolator is-a Korporator (endofunctor specialises functor)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        SyntrometryTaxonomy::relations().iter().any(|(c, p)| {
            *c == SyntrometryConcept::Synkolator && *p == SyntrometryConcept::Korporator
        })
    }
}

/// Axiom: the Syntrix hierarchy has the expected level/aspekt edges into
/// Syntrix — without them the leveled-category structure collapses.
pub struct SyntrixIsLeveled;

impl Axiom for SyntrixIsLeveled {
    fn description(&self) -> &str {
        "Syntrix carries LevelOf and InhabitsLevelOf edges from SyntrixLevel and Aspekt"
    }
    fn holds(&self) -> bool {
        use SyntrometryConcept as S;
        use SyntrometryRelationKind as K;
        let m = SyntrometryCategory::morphisms();
        let has = |from: S, to: S, kind: K| {
            m.iter()
                .any(|r| r.from == from && r.to == to && r.kind == kind)
        };
        has(S::SyntrixLevel, S::Syntrix, K::LevelOf)
            && has(S::Aspekt, S::Syntrix, K::InhabitsLevelOf)
    }
}

impl Ontology for SyntrometryOntology {
    type Cat = SyntrometryCategory;
    type Qual = SyntrometryCategoryOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        SyntrometryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AspektIsTripleProduct),
            Box::new(SynkolatorIsKorporator),
            Box::new(SyntrixIsLeveled),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<SyntrometryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SyntrometryOntology::validate().unwrap();
    }

    #[test]
    fn aspekt_triple_product_axiom_holds() {
        assert!(
            AspektIsTripleProduct.holds(),
            "{}",
            AspektIsTripleProduct.description()
        );
    }

    #[test]
    fn synkolator_is_korporator_axiom_holds() {
        assert!(
            SynkolatorIsKorporator.holds(),
            "{}",
            SynkolatorIsKorporator.description()
        );
    }

    #[test]
    fn syntrix_is_leveled_axiom_holds() {
        assert!(
            SyntrixIsLeveled.holds(),
            "{}",
            SyntrixIsLeveled.description()
        );
    }
}
