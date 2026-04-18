//! Dialectics — reasoning through opposition, from Aristotle to Priest.
//!
//! Dialectics is the structured theory of how opposing terms interact:
//! how a position generates its negation, how the tension between them
//! produces higher-order resolution (or, in modern variants, explicitly
//! refuses resolution). Three literatures supply the concepts here:
//!
//! 1. **Classical** — Aristotle, *Peri Hermeneias* (~350 BCE), *Topics*;
//!    Apuleius and medieval logicians on the Square of Opposition;
//!    Blanché (1966) hexagonal extension.
//! 2. **German idealist & Marxist** — Hegel, *Phenomenology of Spirit*
//!    (1807) and *Science of Logic* (1812–16), for Thesis / Antithesis /
//!    Synthesis, Determinate Negation, and Sublation (*Aufhebung*); Marx,
//!    *Capital* (1867), for internal / material contradiction; Adorno,
//!    *Negative Dialectics* (1966), for non-identity and the refusal of
//!    Hegelian reconciliation.
//! 3. **Modern formal** — Priest, *In Contradiction* (1987), for
//!    dialetheism and paraconsistent logic — the formal treatment of
//!    true contradiction.
//!
//! These traditions are compatible at the structural level we encode
//! here: each names primitives of the opposition-resolution pattern,
//! which is what pr4xis needs for `Syntrometry::Dialectic` (Heim's *Dialektik*) and for
//! dialectical reasoning in downstream ontologies.

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Dialectics",
    source: "Aristotle (~350 BCE); Hegel (1807, 1812); Marx (1867); Adorno (1966); Priest (1987)",
    being: AbstractObject,

    concepts: [
        // === Aristotelian Square of Opposition ===
        SquareOfOpposition,
        Contrary,
        Contradictory,
        Subaltern,
        Subcontrary,

        // === Hegelian triad (core) ===
        DialecticalMoment,
        Thesis,
        Antithesis,
        Synthesis,

        // === Hegelian mechanisms ===
        DeterminateNegation,
        Sublation,
        Contradiction,

        // === Marxist specialisation ===
        InternalContradiction,

        // === Adorno ===
        NegativeDialectics,
        NonIdentity,

        // === Priest (modern formal) ===
        TrueContradiction,
        Paraconsistent,

        // === Aristotle on dialectical argument ===
        DialecticalArgument,
        Endoxa,
    ],

    labels: {
        SquareOfOpposition: ("en", "Square of Opposition", "Aristotle / Apuleius: the four-vertex diagram relating A/E/I/O propositions by contrariety, contradiction, subalternation, and subcontrariety."),
        Contrary: ("en", "Contrary", "Aristotle: two propositions that cannot both be true but can both be false."),
        Contradictory: ("en", "Contradictory", "Aristotle: two propositions that cannot both be true AND cannot both be false — the strongest opposition."),
        Subaltern: ("en", "Subaltern", "Aristotle: the weaker / particular proposition entailed by a stronger universal."),
        Subcontrary: ("en", "Subcontrary", "Aristotle: two propositions that cannot both be false but can both be true."),

        DialecticalMoment: ("en", "Dialectical moment", "Hegel: a structural position within a dialectical movement — Thesis, Antithesis, or Synthesis."),
        Thesis: ("en", "Thesis", "Hegel: the initial affirmation; the starting position before negation."),
        Antithesis: ("en", "Antithesis", "Hegel: the determinate negation of the Thesis — not abstract nothingness but a specific opposing position."),
        Synthesis: ("en", "Synthesis", "Hegel: the higher unity that preserves, negates, and elevates both Thesis and Antithesis — the outcome of Sublation."),

        DeterminateNegation: ("en", "Determinate negation", "Hegel, Science of Logic §§80–82: negation that is specific to what it negates, so that the negation carries the content of the original. Distinct from abstract / empty negation."),
        Sublation: ("en", "Sublation (Aufhebung)", "Hegel: the triple move of simultaneously negating, preserving, and elevating — the mechanism that produces Synthesis from Thesis + Antithesis."),
        Contradiction: ("en", "Contradiction", "Hegel: the internal tension between Thesis and Antithesis; the engine of dialectical development. For Hegel and Marx, productive rather than pathological."),

        InternalContradiction: ("en", "Internal contradiction", "Marx, Capital: a contradiction immanent to a system (e.g. capital's self-undermining tendency), as opposed to external conflict. Drives historical change."),

        NegativeDialectics: ("en", "Negative dialectics", "Adorno (1966): dialectical thinking that refuses the Hegelian Synthesis — non-reconciliation, non-identity-thinking."),
        NonIdentity: ("en", "Non-identity", "Adorno: the residue that resists being subsumed under a concept; what Synthesis fails to capture."),

        TrueContradiction: ("en", "True contradiction", "Priest, In Contradiction (1987): a statement that is both true and false. Dialetheism claims some contradictions are of this kind."),
        Paraconsistent: ("en", "Paraconsistent logic", "A logic that does not explode on contradiction — where P ∧ ¬P does not entail arbitrary Q. The formal substrate for dialetheism."),

        DialecticalArgument: ("en", "Dialectical argument", "Aristotle, Topics: reasoning from endoxa (reputable opinions) to examine a claim — distinct from demonstrative (apodictic) reasoning."),
        Endoxa: ("en", "Endoxa", "Aristotle: widely-held or expert-held opinions that serve as starting points for dialectical argument."),
    },

    is_a: [
        // True subsumption: the Hegelian moments are all DialecticalMoments.
        (Thesis, DialecticalMoment),
        (Antithesis, DialecticalMoment),
        (Synthesis, DialecticalMoment),

        // Marxist internal contradiction is-a contradiction.
        (InternalContradiction, Contradiction),

        // Hegel's determinate negation is the specific kind of negation
        // dialectics uses. (Leaving generic Negation unencoded here —
        // distinction.rs covers pre-dialectical distinction.)

        // Non-identity is the distinguishing concept of Adorno's negative
        // dialectics.
        (NonIdentity, NegativeDialectics),

        // Aristotelian opposition relations are specific Square-of-Opposition
        // cases; keeping them as direct Square children rather than chaining
        // through Contradiction (which means something different in Hegel).
        (Contrary, SquareOfOpposition),
        (Contradictory, SquareOfOpposition),
        (Subaltern, SquareOfOpposition),
        (Subcontrary, SquareOfOpposition),
    ],

    edges: [
        // === Hegelian triad mechanics ===
        // The central dynamic: Thesis → Antithesis via Determinate Negation;
        // both sublated into Synthesis.
        (Thesis, Antithesis, NegatedBy),
        (Antithesis, Thesis, Negates),
        (DeterminateNegation, Antithesis, Produces),
        (Contradiction, Antithesis, Generates),
        (Sublation, Synthesis, Produces),
        (Thesis, Synthesis, SublatedInto),
        (Antithesis, Synthesis, SublatedInto),

        // === Negative dialectics (Adorno) ===
        // The residue Synthesis fails to capture.
        (Synthesis, NonIdentity, LeavesResidue),
        (NonIdentity, NegativeDialectics, Characterises),

        // === Dialetheism (Priest) ===
        // A true contradiction demands a paraconsistent logic to reason in.
        (TrueContradiction, Paraconsistent, Requires),
        (Contradiction, TrueContradiction, SpecialisesTo),

        // === Aristotelian argument structure ===
        (DialecticalArgument, Endoxa, StartsFrom),
    ],

    opposes: [
        // Hegelian Thesis vs Antithesis — the canonical dialectical opposition.
        (Thesis, Antithesis),
        // Adorno refuses Hegelian Synthesis.
        (NegativeDialectics, Synthesis),
        // Dialetheism refuses classical consistency as definitional.
        (Paraconsistent, Contradiction),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Which tradition a concept comes from.
#[derive(Debug, Clone)]
pub struct DialecticsTradition;

impl Quality for DialecticsTradition {
    type Individual = DialecticsConcept;
    type Value = &'static str;

    fn get(&self, c: &DialecticsConcept) -> Option<&'static str> {
        use DialecticsConcept as D;
        Some(match c {
            D::SquareOfOpposition
            | D::Contrary
            | D::Contradictory
            | D::Subaltern
            | D::Subcontrary
            | D::DialecticalArgument
            | D::Endoxa => "aristotle",
            D::DialecticalMoment
            | D::Thesis
            | D::Antithesis
            | D::Synthesis
            | D::DeterminateNegation
            | D::Sublation
            | D::Contradiction => "hegel",
            D::InternalContradiction => "marx",
            D::NegativeDialectics | D::NonIdentity => "adorno",
            D::TrueContradiction | D::Paraconsistent => "priest",
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: DialecticsConcept) -> Vec<DialecticsConcept> {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    DialecticsTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: `DialecticalMoment` has exactly three direct children —
/// `Thesis`, `Antithesis`, `Synthesis` — the Hegelian triad.
pub struct HegelianTriad;

impl Axiom for HegelianTriad {
    fn description(&self) -> &str {
        "the direct children of DialecticalMoment are exactly {Thesis, Antithesis, Synthesis} (Hegel 1807)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DialecticsConcept::DialecticalMoment);
        let expected = [
            DialecticsConcept::Thesis,
            DialecticsConcept::Antithesis,
            DialecticsConcept::Synthesis,
        ];
        actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
    }
}
pr4xis::register_axiom!(HegelianTriad);

/// Axiom: Aristotle's Square of Opposition has exactly four direct
/// children — contraries, contradictories, subalterns, subcontraries.
pub struct AristotelianSquareHasFourVertices;

impl Axiom for AristotelianSquareHasFourVertices {
    fn description(&self) -> &str {
        "the direct children of SquareOfOpposition are exactly {Contrary, Contradictory, Subaltern, Subcontrary} (Aristotle / Apuleius)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DialecticsConcept::SquareOfOpposition);
        let expected = [
            DialecticsConcept::Contrary,
            DialecticsConcept::Contradictory,
            DialecticsConcept::Subaltern,
            DialecticsConcept::Subcontrary,
        ];
        actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
    }
}
pr4xis::register_axiom!(AristotelianSquareHasFourVertices);

/// Axiom: every Synthesis has an upstream Sublation producing it —
/// the edge `(Sublation, Synthesis, Produces)` must exist. Without this,
/// Synthesis would be unexplained.
pub struct SynthesisHasSublation;

impl Axiom for SynthesisHasSublation {
    fn description(&self) -> &str {
        "Sublation produces Synthesis (Hegel, Aufhebung is the mechanism)"
    }
    fn holds(&self) -> bool {
        use DialecticsConcept as D;
        use DialecticsRelationKind as K;
        DialecticsCategory::morphisms()
            .iter()
            .any(|r| r.from == D::Sublation && r.to == D::Synthesis && r.kind == K::Produces)
    }
}
pr4xis::register_axiom!(SynthesisHasSublation);

/// Axiom: Thesis and Antithesis oppose each other at the opposition-reasoning
/// level. This is the dialectical reading of the generic `opposes` relation.
pub struct ThesisAntithesisOppose;

impl Axiom for ThesisAntithesisOppose {
    fn description(&self) -> &str {
        "Thesis opposes Antithesis (the canonical dialectical opposition)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        DialecticsOpposition::pairs().iter().any(|(a, b)| {
            (*a == DialecticsConcept::Thesis && *b == DialecticsConcept::Antithesis)
                || (*a == DialecticsConcept::Antithesis && *b == DialecticsConcept::Thesis)
        })
    }
}
pr4xis::register_axiom!(ThesisAntithesisOppose);

/// Axiom: Adorno's rejection of Synthesis is encoded — NegativeDialectics
/// opposes Synthesis, not merely sits next to it.
pub struct AdornoRefusesSynthesis;

impl Axiom for AdornoRefusesSynthesis {
    fn description(&self) -> &str {
        "NegativeDialectics opposes Synthesis (Adorno 1966 refuses Hegelian reconciliation)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        DialecticsOpposition::pairs().iter().any(|(a, b)| {
            (*a == DialecticsConcept::NegativeDialectics && *b == DialecticsConcept::Synthesis)
                || (*a == DialecticsConcept::Synthesis
                    && *b == DialecticsConcept::NegativeDialectics)
        })
    }
}
pr4xis::register_axiom!(AdornoRefusesSynthesis);

/// Axiom: Priest's dialetheism requires paraconsistent logic — the
/// edge `(TrueContradiction, Paraconsistent, Requires)` must exist.
pub struct DialetheismNeedsParaconsistency;

impl Axiom for DialetheismNeedsParaconsistency {
    fn description(&self) -> &str {
        "TrueContradiction requires Paraconsistent logic (Priest 1987)"
    }
    fn holds(&self) -> bool {
        use DialecticsConcept as D;
        use DialecticsRelationKind as K;
        DialecticsCategory::morphisms().iter().any(|r| {
            r.from == D::TrueContradiction && r.to == D::Paraconsistent && r.kind == K::Requires
        })
    }
}
pr4xis::register_axiom!(DialetheismNeedsParaconsistency);

impl Ontology for DialecticsOntology {
    type Cat = DialecticsCategory;
    type Qual = DialecticsTradition;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        DialecticsOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(HegelianTriad),
            Box::new(AristotelianSquareHasFourVertices),
            Box::new(SynthesisHasSublation),
            Box::new(ThesisAntithesisOppose),
            Box::new(AdornoRefusesSynthesis),
            Box::new(DialetheismNeedsParaconsistency),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<DialecticsCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        DialecticsOntology::validate().unwrap();
    }

    #[test]
    fn hegelian_triad_holds() {
        assert!(HegelianTriad.holds(), "{}", HegelianTriad.description());
    }

    #[test]
    fn aristotelian_square_has_four_vertices_holds() {
        assert!(
            AristotelianSquareHasFourVertices.holds(),
            "{}",
            AristotelianSquareHasFourVertices.description()
        );
    }

    #[test]
    fn synthesis_has_sublation_holds() {
        assert!(
            SynthesisHasSublation.holds(),
            "{}",
            SynthesisHasSublation.description()
        );
    }

    #[test]
    fn thesis_antithesis_oppose_holds() {
        assert!(
            ThesisAntithesisOppose.holds(),
            "{}",
            ThesisAntithesisOppose.description()
        );
    }

    #[test]
    fn adorno_refuses_synthesis_holds() {
        assert!(
            AdornoRefusesSynthesis.holds(),
            "{}",
            AdornoRefusesSynthesis.description()
        );
    }

    #[test]
    fn dialetheism_needs_paraconsistency_holds() {
        assert!(
            DialetheismNeedsParaconsistency.holds(),
            "{}",
            DialetheismNeedsParaconsistency.description()
        );
    }
}
