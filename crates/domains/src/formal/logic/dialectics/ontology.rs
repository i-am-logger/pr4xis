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

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Dialectics",
    source: "Aristotle (~350 BCE); Hegel (1807, 1812); Marx (1867); Adorno (1966); Priest (1987)",

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

/// The intellectual tradition a dialectics concept originates from.
///
/// A closed set of the five scholarly lineages this ontology draws on
/// (see the module-level `source:`): Aristotle (~350 BCE) for the Square
/// of Opposition and dialectical argument; Hegel (1807, 1812) for the
/// triad and its mechanisms; Marx (1867) for internal / material
/// contradiction; Adorno (1966) for negative dialectics; Priest (1987)
/// for dialetheism and paraconsistent logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialecticsTradition {
    /// Aristotle (~350 BCE): Square of Opposition, dialectical argument from endoxa.
    Aristotle,
    /// Hegel (1807, 1812): the dialectical triad and its mechanisms.
    Hegel,
    /// Marx (1867): internal / material contradiction.
    Marx,
    /// Adorno (1966): negative dialectics, non-identity.
    Adorno,
    /// Priest (1987): dialetheism, paraconsistent logic.
    Priest,
}

/// Quality: the [`DialecticsTradition`] each concept comes from.
#[derive(Debug, Clone)]
pub struct DialecticsTraditionOf;

impl Quality for DialecticsTraditionOf {
    type Individual = DialecticsConcept;
    type Value = DialecticsTradition;

    fn get(&self, c: &DialecticsConcept) -> Option<DialecticsTradition> {
        use DialecticsConcept as D;
        Some(match c {
            D::SquareOfOpposition
            | D::Contrary
            | D::Contradictory
            | D::Subaltern
            | D::Subcontrary
            | D::DialecticalArgument
            | D::Endoxa => DialecticsTradition::Aristotle,
            D::DialecticalMoment
            | D::Thesis
            | D::Antithesis
            | D::Synthesis
            | D::DeterminateNegation
            | D::Sublation
            | D::Contradiction => DialecticsTradition::Hegel,
            D::InternalContradiction => DialecticsTradition::Marx,
            D::NegativeDialectics | D::NonIdentity => DialecticsTradition::Adorno,
            D::TrueContradiction | D::Paraconsistent => DialecticsTradition::Priest,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Direct subsumption children of `parent`. Filters
/// `DialecticsCategory::morphisms()` by the `Subsumption` kind, per the
/// kinded-morphism canonical pattern (per_def `TaxonomyDef` is gone).
fn direct_children_of(parent: DialecticsConcept) -> Vec<DialecticsConcept> {
    use pr4xis::category::{Arrow, Category};
    DialecticsCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == DialecticsRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

/// Whether an `Opposition`-kinded edge exists between `a` and `b` in either
/// direction. Filters `DialecticsCategory::morphisms()` by the `Opposition`
/// kind, per the kinded-morphism canonical pattern (per_def `OppositionDef`
/// is gone).
fn opposed_either_direction(a: DialecticsConcept, b: DialecticsConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    DialecticsCategory::morphisms().iter().any(|m| {
        m.kind() == DialecticsRelationKind::Opposition
            && ((m.source() == a && m.target() == b) || (m.source() == b && m.target() == a))
    })
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: `DialecticalMoment` has exactly three direct children —
/// `Thesis`, `Antithesis`, `Synthesis` — the Hegelian triad.
pub struct HegelianTriad;

impl Axiom for HegelianTriad {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let actual = direct_children_of(DialecticsConcept::DialecticalMoment);
        let expected = [
            DialecticsConcept::Thesis,
            DialecticsConcept::Antithesis,
            DialecticsConcept::Synthesis,
        ];
        if actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "HegelianTriad",
        "the direct children of DialecticalMoment are exactly {Thesis, Antithesis, Synthesis} (Hegel 1807)",
        "Hegel (1807) Phenomenology of Spirit"
    );
}
pr4xis::register_axiom!(HegelianTriad, "Hegel (1807) Phenomenology of Spirit");

/// Axiom: Aristotle's Square of Opposition has exactly four direct
/// children — contraries, contradictories, subalterns, subcontraries.
pub struct AristotelianSquareHasFourVertices;

impl Axiom for AristotelianSquareHasFourVertices {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let actual = direct_children_of(DialecticsConcept::SquareOfOpposition);
        let expected = [
            DialecticsConcept::Contrary,
            DialecticsConcept::Contradictory,
            DialecticsConcept::Subaltern,
            DialecticsConcept::Subcontrary,
        ];
        if actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AristotelianSquareHasFourVertices",
        "the direct children of SquareOfOpposition are exactly {Contrary, Contradictory, Subaltern, Subcontrary} (Aristotle / Apuleius)",
        "Aristotle (~350 BCE) Peri Hermeneias; Apuleius; Blanché (1966) hexagonal extension."
    );
}
pr4xis::register_axiom!(
    AristotelianSquareHasFourVertices,
    "Aristotle (~350 BCE) Peri Hermeneias; Apuleius; Blanché (1966) hexagonal extension."
);

/// Axiom: every Synthesis has an upstream Sublation producing it —
/// the edge `(Sublation, Synthesis, Produces)` must exist. Without this,
/// Synthesis would be unexplained.
pub struct SynthesisHasSublation;

impl Axiom for SynthesisHasSublation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DialecticsConcept as D;
        use DialecticsRelationKind as K;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if DialecticsCategory::morphisms()
            .iter()
            .any(|r| r.from == D::Sublation && r.to == D::Synthesis && r.kind == K::Produces)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SynthesisHasSublation",
        "Sublation produces Synthesis (Hegel, Aufhebung is the mechanism)",
        "Hegel (1812-16) Science of Logic"
    );
}
pr4xis::register_axiom!(SynthesisHasSublation, "Hegel (1812-16) Science of Logic");

/// Axiom: Thesis and Antithesis oppose each other at the opposition-reasoning
/// level. This is the dialectical reading of the generic `opposes` relation.
pub struct ThesisAntithesisOppose;

impl Axiom for ThesisAntithesisOppose {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if opposed_either_direction(DialecticsConcept::Thesis, DialecticsConcept::Antithesis) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ThesisAntithesisOppose",
        "Thesis opposes Antithesis (the canonical dialectical opposition)",
        "Hegel (1807) Phenomenology of Spirit"
    );
}
pr4xis::register_axiom!(
    ThesisAntithesisOppose,
    "Hegel (1807) Phenomenology of Spirit"
);

/// Axiom: Adorno's rejection of Synthesis is encoded — NegativeDialectics
/// opposes Synthesis, not merely sits next to it.
pub struct AdornoRefusesSynthesis;

impl Axiom for AdornoRefusesSynthesis {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if opposed_either_direction(
            DialecticsConcept::NegativeDialectics,
            DialecticsConcept::Synthesis,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AdornoRefusesSynthesis",
        "NegativeDialectics opposes Synthesis (Adorno 1966 refuses Hegelian reconciliation)",
        "Adorno (1966) Negative Dialectics"
    );
}
pr4xis::register_axiom!(AdornoRefusesSynthesis, "Adorno (1966) Negative Dialectics");

/// Axiom: Priest's dialetheism requires paraconsistent logic — the
/// edge `(TrueContradiction, Paraconsistent, Requires)` must exist.
pub struct DialetheismNeedsParaconsistency;

impl Axiom for DialetheismNeedsParaconsistency {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DialecticsConcept as D;
        use DialecticsRelationKind as K;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if DialecticsCategory::morphisms().iter().any(|r| {
            r.from == D::TrueContradiction && r.to == D::Paraconsistent && r.kind == K::Requires
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "DialetheismNeedsParaconsistency",
        "TrueContradiction requires Paraconsistent logic (Priest 1987)",
        "Priest (1987) In Contradiction"
    );
}
pr4xis::register_axiom!(
    DialetheismNeedsParaconsistency,
    "Priest (1987) In Contradiction"
);

impl Ontology for DialecticsOntology {
    type Cat = DialecticsCategory;
    type Qual = DialecticsTraditionOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = DialecticsOntology::generated_structural_axioms();
        axioms.push(Box::new(HegelianTriad));
        axioms.push(Box::new(AristotelianSquareHasFourVertices));
        axioms.push(Box::new(SynthesisHasSublation));
        axioms.push(Box::new(ThesisAntithesisOppose));
        axioms.push(Box::new(AdornoRefusesSynthesis));
        axioms.push(Box::new(DialetheismNeedsParaconsistency));
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
        assert_category_laws::<DialecticsCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DialecticsOntology::validate().unwrap();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hegelian_triad_holds() {
        assert!(
            HegelianTriad.verify().is_ok(),
            "{}",
            HegelianTriad.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn aristotelian_square_has_four_vertices_holds() {
        assert!(
            AristotelianSquareHasFourVertices.verify().is_ok(),
            "{}",
            AristotelianSquareHasFourVertices.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn synthesis_has_sublation_holds() {
        assert!(
            SynthesisHasSublation.verify().is_ok(),
            "{}",
            SynthesisHasSublation.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn thesis_antithesis_oppose_holds() {
        assert!(
            ThesisAntithesisOppose.verify().is_ok(),
            "{}",
            ThesisAntithesisOppose.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn adorno_refuses_synthesis_holds() {
        assert!(
            AdornoRefusesSynthesis.verify().is_ok(),
            "{}",
            AdornoRefusesSynthesis.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dialetheism_needs_paraconsistency_holds() {
        assert!(
            DialetheismNeedsParaconsistency.verify().is_ok(),
            "{}",
            DialetheismNeedsParaconsistency.description().as_str()
        );
    }
}
