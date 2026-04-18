//! Syntrometry ontology — Heim's syntrometric primitives.
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
//! is claimed to instantiate Heim's syntrometric structure — not to assert
//! it. The companion module `lineage_functor` maps Syntrometry → the pr4xis
//! substrate and verifies the functor laws; five further cross-functors
//! align Heim's vocabulary with pr4xis's meta, composition, staging, and
//! cognitive ontologies.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Syntrometry",
    source: "Heim (~1980 *Syntrometrische Maximentelezentrik*); modernized 2025 paper on heim-theory.com",
    being: AbstractObject,

    concepts: [
        // === Distinction primitives (§1) ===
        Predicate,
        PredicateMatrix,
        Dialectic,
        Coordination,
        Aspect,

        // === Syntrometric structures (§2) ===
        Syntrix,
        SyntrixLevel,
        Syncolator,
        Composer,

        // === CEM mereology primitive ===
        Part,

        // === Teleological / hierarchical (§§ Telezentrik + Metroplextheorie) ===
        // The "metaphysical" concepts that are actually architecture.
        Telecenter,
        Maxim,
        TranscendenceLevel,
        Metroplex,

        // === Permutation operators (§1) ===
        // Heim's two operators on predicate sequences and orientations.
        // C (capital) permutes predicate SEQUENCES; c (lowercase) permutes
        // orientations within a single predicate.
        SequencePermutation,
        OrientationPermutation,

        // === Multi-aspect structure (§2) ===
        AspectivalSystem,

        // === Self-observation (reflexivity ρ) ===
        Reflexivity,
    ],

    labels: {
        Predicate: ("en", "Predicate", "The basic distinction — the atomic unit of syntrometric logic. A predicate separates what-is from what-is-not (Spencer-Brown Laws of Form)."),
        PredicateMatrix: ("en", "PredicateMatrix (Heim's *Predikatrix*)", "A structured set of predicates — a predicate-system. Every PredicateMatrix carries an ordering (Coordination) and an opposition structure (Dialectic)."),
        Dialectic: ("en", "Dialectic (Heim's *Dialektik*)", "The binary-opposition structure on a PredicateMatrix. Pairs of predicates that are mutually exclusive or complementary."),
        Coordination: ("en", "Coordination (Heim's *Koordination*)", "The ordering / coordination mapping between predicates within a PredicateMatrix. Determines predicate sequences and orientations."),
        Aspect: ("en", "Aspect (Heim's *Aspekt*)", "Subjective aspect S = [D × K × P]. The product of Dialectic × Coordination × PredicateMatrix. An observer-relative view of the underlying distinction-system."),

        Syntrix: ("en", "Syntrix", "The Category of Leveled Structures C_SL (§2.2). A hierarchical organisation of Aspects into levels — the syntrometric analogue of a category."),
        SyntrixLevel: ("en", "Syntrix level", "A single level within a Syntrix — corresponds to an object / grade of abstraction. The Syntrix itself is the tower of levels."),
        Syncolator: ("en", "Syncolator (Heim's *Synkolator*)", "An endofunctor on the Syntrix: F: C_SL → C_SL. Maps each level to another level of the same Syntrix, preserving composition."),
        Composer: ("en", "Composer (Heim's *Korporator*)", "A structure-mapping functor between Syntrices: K: Syntrix_1 → Syntrix_2. The general case of cross-syntrix composition — Syncolator is the endomorphism specialisation."),

        Part: ("en", "Part", "The mereological part-of primitive Part(A, B). Classical Extensional Mereology (CEM) as used by Heim; must satisfy Weak Supplementation."),

        SequencePermutation: ("en", "Sequence permutation C", "Heim §1: the operator C that permutes predicate sequences within a PredicateMatrix. Categorically an automorphism on the sequence-ordering — composition under associativity of the underlying Coordination."),
        OrientationPermutation: ("en", "Orientation permutation c", "Heim §1: the operator c that permutes the orientation (direction) of an individual predicate. Paired with C to give the full Heim permutation algebra C/c."),
        AspectivalSystem: ("en", "AspectivalSystem (Heim's *Aspektivsystem*)", "Heim §2: a structured system of Aspects with aspect-relative relations between them. *Aspektrelativität* (aspect-relative truth) is the property an AspectivalSystem exhibits — different Aspects see different facets of the same underlying distinction-system. Maps via Kripke semantics to pr4xis's 'multiple ontologies viewing the same domain via functors' pattern."),
        Reflexivity: ("en", "Reflexivity ρ", "Heim: the self-observation natural transformation — a Syntrix observing itself. Categorically a natural transformation ρ : Id ⇒ Syncolator (or Syncolator ⇒ Id), encoding von Foerster's eigenform operationally."),
        Telecenter: ("en", "Telecenter", "A goal-attractor — an organising target that convergent syntrometric activity tends toward. Categorically a colimit / fixed-point; cybernetically an Ashby 'good regulator' attractor; in pr4xis maps to CommunicativeGoal / Eigenform (X = F(X)). Source: Heim Telezentrik."),
        Maxim: ("en", "Maxim (Heim's *Maxime*)", "An extremal of expedient ideas — the selection operator choosing among candidate Aspects which ones actualise toward a Telecenter. Cybernetically Conant-Ashby 'every regulator is a model of its system'; in pr4xis maps to BDI Intention / C1 Attention / Optimization. Source: Heim *Maximentelezentrik*."),
        TranscendenceLevel: ("en", "TranscendenceLevel (Heim's *Transzendenzstufe*)", "A transcendence-level — a qualitative leap between grades of abstraction within a Metroplex hierarchy. In pr4xis maps to Staging levels (Futamura projections) / Metroplex grades / C1 vs C2 consciousness split. Source: Heim §§ Metroplextheorie."),
        Metroplex: ("en", "Metroplex", "The hierarchical container organising Syntrices into TranscendenceLevels (Heim's *Transzendenzstufen*). In pr4xis maps to system-of-systems composition. Source: Heim §§ Metroplextheorie."),
    },

    is_a: [
        // True subsumption only: every Syncolator IS a Composer, because
        // an endofunctor is a functor specialised to Source = Target
        // (Mac Lane Ch. II §1). Compositional / part-of relationships go
        // into has_a: below.
        (Syncolator, Composer),
    ],

    has_a: [
        // An Aspect is constituted from D × K × P (Heim §1). Each of the
        // three is a proper part of every Aspect instance.
        (Aspect, Dialectic),
        (Aspect, Coordination),
        (Aspect, PredicateMatrix),

        // A PredicateMatrix is a structured collection OF Predicates.
        (PredicateMatrix, Predicate),

        // A Syntrix contains Levels.
        (Syntrix, SyntrixLevel),

        // A SyntrixLevel is a PredicateMatrix-at-a-given-grade; mereologically
        // it contains the same predicates its parent PredicateMatrix would.
        (SyntrixLevel, Predicate),

        // === Teleological + hierarchical structure ===
        // A Metroplex contains Syntrices organised by TranscendenceLevels.
        (Metroplex, Syntrix),
        (Metroplex, TranscendenceLevel),
        // A Telecenter carries Maxims (the selection operators that
        // actualise which Aspects converge toward the Telecenter).
        (Telecenter, Maxim),

        // AspectivalSystem contains Aspects by definition — a system-of-aspects.
        (AspectivalSystem, Aspect),
    ],

    edges: [
        // === Predicate composition ===
        (Coordination, PredicateMatrix, Orders),
        (Dialectic, PredicateMatrix, StructuresOppositionIn),

        // === Aspect construction ===
        (PredicateMatrix, Aspect, Contributes),
        (Dialectic, Aspect, Contributes),
        (Coordination, Aspect, Contributes),

        // === Syntrix hierarchy ===
        (SyntrixLevel, Syntrix, LevelOf),
        (Aspect, Syntrix, InhabitsLevelOf),

        // === Syntrometric operators ===
        (Syncolator, Syntrix, EndomorphismOn),
        (Composer, Syntrix, MapsBetween),

        // === Permutation operators (C/c) ===
        (SequencePermutation, Coordination, Permutes),
        (OrientationPermutation, Predicate, Permutes),

        // === Multi-aspect structure ===
        // AspectivalSystem organises Aspects at the categorical level.
        (AspectivalSystem, Syntrix, OrganisesOver),

        // === Self-observation (Reflexivity ρ) ===
        // Reflexivity is a natural transformation on Syncolator — the
        // Syntrix observes itself via the endofunctor.
        (Reflexivity, Syncolator, NaturallyTransforms),

        // === Teleological / hierarchical ===
        // A Maxim selects among candidate Aspects for a Telecenter.
        (Maxim, Aspect, Selects),
        // Maxims are oriented toward a Telecenter — the target of selection.
        (Maxim, Telecenter, ConvergesToward),
        // TranscendenceLevel is a structural index into a Metroplex — each
        // level corresponds to a grade of Syntrix within the hierarchy.
        (TranscendenceLevel, Syntrix, Grades),
        // A Telecenter is a fixed-point of a Syncolator (Eigenform
        // X = F(X)) — this is the categorical content that justifies
        // the pr4xis mapping to Eigenform / colimit. Reads as
        // "Telecenter is a FixedPointOf Syncolator".
        (Telecenter, Syncolator, FixedPointOf),
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
            S::Predicate | S::PredicateMatrix | S::Dialectic | S::Coordination | S::Aspect => {
                "distinction-primitive"
            }
            S::Syntrix | S::SyntrixLevel | S::Syncolator | S::Composer => "syntrometric-structure",
            S::Part => "mereology",
            S::Telecenter | S::Maxim | S::TranscendenceLevel | S::Metroplex => {
                "teleological-hierarchical"
            }
            S::SequencePermutation | S::OrientationPermutation => "permutation-operator",
            S::AspectivalSystem => "multi-aspect",
            S::Reflexivity => "self-observation",
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

/// Axiom: an Aspect is the product [Dialectic × Coordination × PredicateMatrix]
/// (§1 of the modernized paper). The three must all appear as direct
/// mereological parts of Aspect.
pub struct AspectIsTripleProduct;

impl Axiom for AspectIsTripleProduct {
    fn description(&self) -> &str {
        "Aspect mereologically contains {Dialectic, Coordination, PredicateMatrix} (Heim §1)"
    }
    fn holds(&self) -> bool {
        let parts = direct_parts_of(SyntrometryConcept::Aspect);
        let expected = [
            SyntrometryConcept::Dialectic,
            SyntrometryConcept::Coordination,
            SyntrometryConcept::PredicateMatrix,
        ];
        expected.iter().all(|e| parts.contains(e))
    }
}
pr4xis::register_axiom!(
    AspectIsTripleProduct,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

/// Axiom: Syncolator is-a Composer — every endofunctor is a functor
/// specialised to the same source and target. (Mac Lane Ch. II §1.)
pub struct SyncolatorIsComposer;

impl Axiom for SyncolatorIsComposer {
    fn description(&self) -> &str {
        "Syncolator is-a Composer (endofunctor specialises functor)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        SyntrometryTaxonomy::relations().iter().any(|(c, p)| {
            *c == SyntrometryConcept::Syncolator && *p == SyntrometryConcept::Composer
        })
    }
}
pr4xis::register_axiom!(
    SyncolatorIsComposer,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

/// Axiom: the Syntrix hierarchy has the expected level/aspekt edges into
/// Syntrix — without them the leveled-category structure collapses.
pub struct SyntrixIsLeveled;

impl Axiom for SyntrixIsLeveled {
    fn description(&self) -> &str {
        "Syntrix carries LevelOf and InhabitsLevelOf edges from SyntrixLevel and Aspect"
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
            && has(S::Aspect, S::Syntrix, K::InhabitsLevelOf)
    }
}
pr4xis::register_axiom!(
    SyntrixIsLeveled,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

/// A Metroplex mereologically contains Syntrices organised
/// by TranscendenceLevels. Both parts must be present.
pub struct MetroplexContainsSyntrixAndLevels;

impl Axiom for MetroplexContainsSyntrixAndLevels {
    fn description(&self) -> &str {
        "Metroplex contains {Syntrix, TranscendenceLevel} (Heim Metroplextheorie)"
    }
    fn holds(&self) -> bool {
        let parts = direct_parts_of(SyntrometryConcept::Metroplex);
        parts.contains(&SyntrometryConcept::Syntrix)
            && parts.contains(&SyntrometryConcept::TranscendenceLevel)
    }
}
pr4xis::register_axiom!(
    MetroplexContainsSyntrixAndLevels,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

/// Every Maxim ConvergesToward a Telecenter. The pair
/// (Maxim, Telecenter) must exist with the ConvergesToward kind —
/// otherwise the teleological selection claim of Maximentelezentrik
/// is unencoded.
pub struct MaximConvergesTowardTelecenter;

impl Axiom for MaximConvergesTowardTelecenter {
    fn description(&self) -> &str {
        "Maxim carries a ConvergesToward edge into Telecenter (Heim Telezentrik)"
    }
    fn holds(&self) -> bool {
        use SyntrometryConcept as S;
        use SyntrometryRelationKind as K;
        SyntrometryCategory::morphisms()
            .iter()
            .any(|r| r.from == S::Maxim && r.to == S::Telecenter && r.kind == K::ConvergesToward)
    }
}
pr4xis::register_axiom!(
    MaximConvergesTowardTelecenter,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

/// A Telecenter is a fixed-point of a Syncolator — the categorical
/// content of the eigenform / goal-attractor mapping. Encoded as
/// `(Telecenter, Syncolator, FixedPointOf)` so `(from, to, kind)`
/// reads "Telecenter is a FixedPointOf Syncolator", preserving the
/// intended subject/object roles.
pub struct TelecenterIsSyncolatorFixedPoint;

impl Axiom for TelecenterIsSyncolatorFixedPoint {
    fn description(&self) -> &str {
        "Telecenter is a FixedPointOf Syncolator (eigenform X=F(X))"
    }
    fn holds(&self) -> bool {
        use SyntrometryConcept as S;
        use SyntrometryRelationKind as K;
        SyntrometryCategory::morphisms()
            .iter()
            .any(|r| r.from == S::Telecenter && r.to == S::Syncolator && r.kind == K::FixedPointOf)
    }
}
pr4xis::register_axiom!(
    TelecenterIsSyncolatorFixedPoint,
    "> \"A Modernized Syntrometric Logic: Foundations and Applications\" (2025)"
);

impl Ontology for SyntrometryOntology {
    type Cat = SyntrometryCategory;
    type Qual = SyntrometryCategoryOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        SyntrometryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AspectIsTripleProduct),
            Box::new(SyncolatorIsComposer),
            Box::new(SyntrixIsLeveled),
            Box::new(MetroplexContainsSyntrixAndLevels),
            Box::new(MaximConvergesTowardTelecenter),
            Box::new(TelecenterIsSyncolatorFixedPoint),
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
            AspectIsTripleProduct.holds(),
            "{}",
            AspectIsTripleProduct.description()
        );
    }

    #[test]
    fn synkolator_is_korporator_axiom_holds() {
        assert!(
            SyncolatorIsComposer.holds(),
            "{}",
            SyncolatorIsComposer.description()
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

    #[test]
    fn metroplex_contains_syntrix_and_levels_holds() {
        assert!(
            MetroplexContainsSyntrixAndLevels.holds(),
            "{}",
            MetroplexContainsSyntrixAndLevels.description()
        );
    }

    #[test]
    fn maxime_converges_toward_telecenter_holds() {
        assert!(
            MaximConvergesTowardTelecenter.holds(),
            "{}",
            MaximConvergesTowardTelecenter.description()
        );
    }

    #[test]
    fn telecenter_is_synkolator_fixed_point_holds() {
        assert!(
            TelecenterIsSyncolatorFixedPoint.holds(),
            "{}",
            TelecenterIsSyncolatorFixedPoint.description()
        );
    }
}
