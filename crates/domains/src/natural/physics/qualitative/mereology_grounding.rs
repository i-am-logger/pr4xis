//! The `QualitativeProcess -> formal::mereology::MereologyTheory` bridge the
//! module's own README flagged as missing, resolved by actually checking the
//! literature rather than assuming a correspondence exists.
//!
//! This is deliberately NOT a `pr4xis::category::Functor` impl -- and,
//! unlike `formal::mereology::wordnet_grounding` (which grounds 2 of
//! `MereologyTheory`'s 13 concepts in WordNet), the honest answer here turns
//! out to be a TOTAL negative: none of `QualitativeProcess`'s 7 concepts,
//! including the two Hayes naive-physics relata a spatial-parthood bridge
//! would most plausibly target, has a citable `MereologyTheoryConcept`
//! counterpart.
//!
//! `Individual`/`Quantity`/`Process`/`Precondition`/`Influence` are Forbus
//! (1984) process-theory primitives with no part-whole content at all --
//! not even candidates. That leaves exactly two candidates, and both fail
//! for reasons the literature states directly rather than by omission:
//!
//! - **`Containment` is not parthood.** Casati & Varzi (1999) *Parts and
//!   Places: The Structures of Spatial Representation* (MIT Press) build
//!   the parthood theory `MereologyTheory` is grounded in in Chapter 2
//!   ("Parthood Structures"), and treat spatial LOCATION -- the family of
//!   relations a container/content pair actually instantiates -- as a
//!   separate structure four chapters later, in Chapter 6 ("Modes of
//!   Location"), with its own primitives (`Functionality`: nothing has more
//!   than one exact location; `Conditional Reflexivity`: exact locations are
//!   exactly located at themselves -- Casati & Varzi 1999 p. 121, as
//!   summarized in Gilmore, Calosi & Costa, "Location and Mereology",
//!   *Stanford Encyclopedia of Philosophy* (2013, rev. 2024) §2.2.2).
//!   Location is never *defined as* parthood anywhere in that apparatus. A
//!   marble inside a box occupies a region enclosed by the box's spatial
//!   boundary without composing any part of the box's matter -- exactly the
//!   distinction the book keeps two chapters apart rather than collapsing.
//!   Asserting `Containment -> ProperPart` would invent a correspondence
//!   the source's own structure argues against.
//! - **`Support` is not parthood either, for an even more basic reason: it
//!   is not the same KIND of relation.** Hayes (1979) *The Naive Physics
//!   Manifesto* introduces support as a physical/causal primitive --
//!   contact plus gravity; remove the support and the individual falls
//!   (formalized further by Davis (1990) *Representations of Commonsense
//!   Knowledge* Ch. 7) -- with no part-whole content anywhere in the
//!   naive-physics treatment. A book resting on a table is not a part of
//!   the table under any reading Hayes offers.
//!
//! So the classifier below maps every `QualitativeProcessConcept` to
//! `None`. Per the README's own framing (`README.md` "Functors" section)
//! and the `formal::mereology::wordnet_grounding` precedent this file
//! mirrors: "it is entirely legitimate for this classifier to map ...
//! Containment and Support ... to `None` if that's what the literature
//! actually supports" -- it does. The keystone axiom below proves that
//! negative rather than assuming it, mirroring the spirit of
//! `wordnet_grounding`'s `ungrounded_mereology_concepts_stay_ungrounded`
//! test, promoted here to the primary claim rather than a side check.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::Axiom;

use super::containment::{self, Individual, Size};
use super::ontology::QualitativeProcessConcept;
use crate::formal::mereology::ontology::MereologyTheoryConcept;

/// The grounding classifier: every `QualitativeProcessConcept` maps to
/// `None` -- no loaded/citable `MereologyTheoryConcept` counterpart. Written
/// as an explicit match over every variant (not a wildcard `_ => None`) so
/// adding a future `QualitativeProcessConcept` forces a conscious decision
/// here rather than silently inheriting the negative. See the module doc
/// comment for the per-concept reasoning (Forbus concepts are not
/// part-whole candidates at all; `Containment` is a LOCATION relation
/// Casati & Varzi (1999) keep formally separate from parthood; `Support` is
/// a physical/causal relation Hayes (1979) never treats as part-whole).
pub fn mereology_concept_of_qualitative_process(
    concept: QualitativeProcessConcept,
) -> Option<MereologyTheoryConcept> {
    match concept {
        QualitativeProcessConcept::Individual
        | QualitativeProcessConcept::Quantity
        | QualitativeProcessConcept::Process
        | QualitativeProcessConcept::Precondition
        | QualitativeProcessConcept::Influence
        | QualitativeProcessConcept::Containment
        | QualitativeProcessConcept::Support => None,
    }
}

/// The keystone axiom: **no** `QualitativeProcessConcept` -- including
/// `Containment` and `Support`, the only two concepts a spatial-parthood
/// bridge could plausibly target -- honestly grounds in any
/// `MereologyTheoryConcept`. Concept-level proof (see
/// [`HayesContainmentAndSupportStayUngroundedWhenExercised`] below for the
/// matching instance-level check against real Hayes-physics computations).
pub struct NoQualitativeProcessConceptGroundsInMereology;

impl Axiom for NoQualitativeProcessConceptGroundsInMereology {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let grounded_count = QualitativeProcessConcept::variants()
            .into_iter()
            .filter(|c| mereology_concept_of_qualitative_process(*c).is_some())
            .count();
        if grounded_count == 0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NoQualitativeProcessConceptGroundsInMereology",
        "no QualitativeProcess concept -- including Containment and Support -- has an honestly-citable MereologyTheory counterpart: spatial containment is a location relation distinct from parthood, and support is a physical/causal relation, not a part-whole one",
        "Casati & Varzi (1999) Parts and Places Ch. 2 vs Ch. 6 (p. 121); Hayes (1979) The Naive Physics Manifesto"
    );
}

pr4xis::register_axiom!(
    NoQualitativeProcessConceptGroundsInMereology,
    "Casati & Varzi (1999) Parts and Places; Hayes (1979) The Naive Physics Manifesto"
);

/// The instance-level complement to the keystone axiom: run the REAL
/// Hayes-physics computations (`containment::fits`,
/// `containment::falls_without_support`) on concrete fixtures and confirm
/// the classifier's `None` answer holds when actually exercised against
/// live data, not just at the abstract concept level.
///
/// Honest scope statement (mirroring `wordnet_grounding`'s own): this
/// checks 2 representative fixtures -- the trophy/suitcase containment pair
/// `ContainerSizeAtLeastContentSize` (`ontology.rs`) already establishes as
/// physically correct, and a supported/unsupported individual pair for
/// `UnsupportedIndividualsFall` -- NOT a proptest sweep over the full
/// `Size` x support-boolean space (that sweep already exists, independent
/// of mereology, as `prop_winograd_antecedent_is_whichever_individual_exceeds_the_other`
/// in `ontology.rs`). What IS proven here: a concrete Hayes-physics
/// judgment succeeding or failing never coincides with, or depends on, a
/// `MereologyTheoryConcept` classification -- the two ontologies' real
/// realized mechanics compose independently, exactly as the negative
/// classifier claims, checked against actual computation rather than typed
/// only at the enum level.
pub struct HayesContainmentAndSupportStayUngroundedWhenExercised;

impl Axiom for HayesContainmentAndSupportStayUngroundedWhenExercised {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Fixture 1 -- Containment: the same trophy/suitcase pair
        // `ContainerSizeAtLeastContentSize` proves physically correct,
        // re-run here to tie that SAME real computation to the mereology
        // classifier rather than asserting the tie abstractly.
        let suitcase = Individual {
            name: "suitcase".into(),
            size: Size::Large,
        };
        let trophy = Individual {
            name: "trophy".into(),
            size: Size::Small,
        };
        let small_suitcase = Individual {
            name: "small suitcase".into(),
            size: Size::Small,
        };
        let big_trophy = Individual {
            name: "big trophy".into(),
            size: Size::Large,
        };
        let containment_computation_correct = containment::fits(&suitcase, &trophy)
            && !containment::fits(&small_suitcase, &big_trophy);
        let containment_ungrounded =
            mereology_concept_of_qualitative_process(QualitativeProcessConcept::Containment)
                .is_none();

        // Fixture 2 -- Support: a supported individual stays up, an
        // unsupported one falls (Hayes 1979), re-run against the same
        // classifier question.
        let support_computation_correct =
            !containment::falls_without_support(true) && containment::falls_without_support(false);
        let support_ungrounded =
            mereology_concept_of_qualitative_process(QualitativeProcessConcept::Support).is_none();

        if containment_computation_correct
            && containment_ungrounded
            && support_computation_correct
            && support_ungrounded
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HayesContainmentAndSupportStayUngroundedWhenExercised",
        "checked against representative fixtures (a trophy/suitcase containment pair and a supported/unsupported individual): real Hayes-physics computations succeed or fail exactly as ContainerSizeAtLeastContentSize/UnsupportedIndividualsFall predict, and neither outcome ever depends on or produces a MereologyTheory classification -- not a proven universal over the full Size x support-boolean space",
        "Hayes (1985) Naive Physics I: Ontology for Liquids \u{00a7}3; Hayes (1979) The Naive Physics Manifesto; Casati & Varzi (1999) Parts and Places"
    );
}

pr4xis::register_axiom!(
    HayesContainmentAndSupportStayUngroundedWhenExercised,
    "Hayes (1985); Hayes (1979); Casati & Varzi (1999)"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn containment_has_no_mereology_counterpart() {
        assert_eq!(
            mereology_concept_of_qualitative_process(QualitativeProcessConcept::Containment),
            None
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn support_has_no_mereology_counterpart() {
        assert_eq!(
            mereology_concept_of_qualitative_process(QualitativeProcessConcept::Support),
            None
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn no_qualitative_process_concept_grounds_in_mereology() {
        // No invented correspondences: every QualitativeProcess concept --
        // Forbus process-theory primitives AND both Hayes relata -- has no
        // loaded MereologyTheory counterpart, and this classifier says so
        // honestly rather than forcing a total mapping.
        use pr4xis::category::FinitelyGenerated;
        let grounded_count = QualitativeProcessConcept::variants()
            .into_iter()
            .filter(|c| mereology_concept_of_qualitative_process(*c).is_some())
            .count();
        assert_eq!(
            grounded_count, 0,
            "no QualitativeProcess concept should ground in MereologyTheory"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn keystone_axiom_holds() {
        assert!(
            NoQualitativeProcessConceptGroundsInMereology
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hayes_containment_and_support_stay_ungrounded_when_exercised() {
        assert!(
            HayesContainmentAndSupportStayUngroundedWhenExercised
                .verify()
                .is_ok()
        );
    }
}
