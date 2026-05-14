//! Mechanobiology — how mechanical forces become biological signals.
//!
//! Models mechanical forces (membrane tension, shear stress, compressive
//! and tensile stress, substrate stiffness), mechanotransduction machinery
//! (mechanosensitive channels and their conformational states), frequency-
//! response properties (filtering, activation threshold, inactivation
//! kinetics, recovery time), cellular responses (calcium transients,
//! cytoskeletal remodelling, focal adhesions, mechanoadaptation), and the
//! causal cascade mechanical load → membrane deformation → channel gating
//! → ion influx → intracellular signalling, plus the repetitive-stimulus
//! frequency-dependence branch and the sustained-force adaptation branch.
//! Per `feedback_one_ontology_per_module` the original split between
//! `MechanobiologyEntity` and `MechanobiologyCausalEvent` has been merged:
//! events are first-class concepts subsumed by the `MechanobiologyEvent`
//! umbrella.
//!
//! # Literature
//!
//! - **Mofrad & Kamm (eds.) (2010)** *Cellular Mechanotransduction:
//!   Diverse Perspectives from Molecules to Tissues*, Cambridge University
//!   Press — canonical reference for membrane tension, shear stress, focal
//!   adhesions, cytoskeletal remodelling, and the molecular substrates of
//!   mechanotransduction.
//! - **Iskratsch, Wolfenson & Sheetz (2014)** "Appreciating force and shape —
//!   the rise of mechanotransduction in cell biology", *Nature Reviews
//!   Molecular Cell Biology* 15(12):825–833 — review of force/shape sensing,
//!   mechanoadaptation, and substrate-stiffness response.
//! - **Vogel & Sheetz (2006)** "Local force and geometry sensing regulate
//!   cell functions", *Nature Reviews Molecular Cell Biology* 7(4):265–275 —
//!   activation thresholds, mechanosensor conformational states, calcium-
//!   transient downstream responses.
//! - **Coste et al. (2010)** "Piezo1 and Piezo2 are essential components of
//!   distinct mechanically activated cation channels", *Science*
//!   330(6000):55–60 — Piezo discovery (2021 Nobel Prize), open/closed/
//!   inactivated states.
//! - **Lewis, Cui & Grandl (2017)** "Transduction of repetitive mechanical
//!   stimuli by Piezo1 and Piezo2 ion channels", *Cell Reports*
//!   19(12):2572–2585 (PMID:28636944) — frequency filtering and inactivation
//!   kinetics of Piezo channels.
//! - **Lin, Buyan & Corry (2022)** "Computational studies of Piezo1 yield
//!   insights into key lipid-protein interactions, channel activation
//!   stresses and channel deactivation states", *J. Gen. Physiol.* (2023
//!   PMID:37459546) — Piezo1 membrane-stretch activation threshold.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Mechanobiology",
    source: "Mofrad & Kamm eds. (2010) Cellular Mechanotransduction (CUP); Iskratsch, Wolfenson & Sheetz (2014) Nat. Rev. Mol. Cell Biol. 15(12):825-833; Vogel & Sheetz (2006) Nat. Rev. Mol. Cell Biol. 7(4):265-275; Coste et al. (2010) Science 330(6000):55-60; Lewis, Cui & Grandl (2017) Cell Reports 19(12):2572-2585 (PMID:28636944); Lin, Buyan & Corry (2022/2023) J. Gen. Physiol. (PMID:37459546)",

    concepts: [
        // === Mechanical forces (Mofrad & Kamm 2010 Ch. 2-4) ===
        MembraneTension,
        ShearStress,
        CompressiveStress,
        TensileStress,
        SubstrateStiffness,

        // === Mechanotransduction machinery (Coste 2010; Mofrad & Kamm 2010) ===
        MechanosensitiveChannel,
        ChannelConformation,
        OpenState,
        ClosedState,
        InactivatedState,

        // === Frequency response (Lewis et al. 2017) ===
        FrequencyFiltering,
        ActivationThreshold,
        InactivationKinetics,
        RecoveryTime,

        // === Cellular responses (Vogel & Sheetz 2006; Iskratsch 2014) ===
        CalciumTransient,
        CytoskeletalRemodeling,
        FocalAdhesion,
        Mechanoadaptation,

        // === Abstract umbrellas ===
        MechanicalForce,
        ChannelState,
        FrequencyProperty,
        CellularResponse,
        MechanobiologyEvent,

        // === Causal events (Iskratsch 2014; Lewis et al. 2017) ===
        MechanicalLoad,
        MembraneDeformation,
        ChannelGating,
        IonInflux,
        IntracellularSignaling,
        RepetitiveStimulus,
        ChannelInactivation,
        FrequencyDependentResponse,
        SustainedForce,
        ThresholdShift,
    ],

    labels: {
        MembraneTension: ("en", "Membrane tension",
            "Mofrad & Kamm (2010) Ch. 2: in-plane tension across the plasma membrane; the canonical activating stimulus for Piezo-family channels."),
        ShearStress: ("en", "Shear stress",
            "Mofrad & Kamm (2010) Ch. 3: tangential force per unit area applied to a surface (e.g. fluid flow over endothelium)."),
        CompressiveStress: ("en", "Compressive stress",
            "Mofrad & Kamm (2010) Ch. 4: force per unit area that pushes material together; experienced by cartilage and bone."),
        TensileStress: ("en", "Tensile stress",
            "Mofrad & Kamm (2010) Ch. 4: force per unit area that pulls material apart; experienced by tendon and ligament."),
        SubstrateStiffness: ("en", "Substrate stiffness",
            "Iskratsch et al. (2014): elastic modulus of the extracellular substrate sensed by cells through focal adhesions and the cytoskeleton."),

        MechanosensitiveChannel: ("en", "Mechanosensitive channel",
            "Coste et al. (2010): transmembrane channel that opens in response to membrane tension (Piezo1/2 are the canonical eukaryotic exemplars)."),
        ChannelConformation: ("en", "Channel conformation",
            "Coste et al. (2010): the structural state of a mechanosensitive channel (open, closed, or inactivated)."),
        OpenState: ("en", "Open state",
            "Vogel & Sheetz (2006); Coste et al. (2010): conformational state in which the channel permits ion flux."),
        ClosedState: ("en", "Closed state",
            "Vogel & Sheetz (2006); Coste et al. (2010): conformational state in which the channel is impermeant and primed to reopen."),
        InactivatedState: ("en", "Inactivated state",
            "Lewis et al. (2017): non-conducting state distinct from closed; the channel cannot reopen until recovery from inactivation."),

        FrequencyFiltering: ("en", "Frequency filtering",
            "Lewis et al. (2017): inactivation kinetics define a high-frequency cutoff for repetitive mechanical stimuli; mechanosensitive channels act as low-pass filters."),
        ActivationThreshold: ("en", "Activation threshold",
            "Lin, Buyan & Corry (2022/2023) PMID:37459546: minimum membrane tension required to open the channel (~3 mN/m for Piezo1)."),
        InactivationKinetics: ("en", "Inactivation kinetics",
            "Lewis et al. (2017): time constant governing the transition from open to inactivated state (~15-30 ms for Piezo1)."),
        RecoveryTime: ("en", "Recovery time",
            "Lewis et al. (2017): time required for the channel to return from inactivated to closed state; sets the maximum stimulus frequency."),

        CalciumTransient: ("en", "Calcium transient",
            "Vogel & Sheetz (2006): brief intracellular Ca2+ elevation produced by mechanosensitive-channel ion influx."),
        CytoskeletalRemodeling: ("en", "Cytoskeletal remodeling",
            "Iskratsch et al. (2014): reorganisation of actin / microtubule / intermediate-filament networks in response to mechanical cues."),
        FocalAdhesion: ("en", "Focal adhesion",
            "Iskratsch et al. (2014): multiprotein complex anchoring the cytoskeleton to the extracellular matrix; the cell's primary force-sensing organelle at the substrate interface."),
        Mechanoadaptation: ("en", "Mechanoadaptation",
            "Iskratsch et al. (2014): long-term shift in cellular force-sensing thresholds in response to sustained mechanical loading."),

        MechanicalForce: ("en", "Mechanical force",
            "Mofrad & Kamm (2010): umbrella for the categories of mechanical stimulus that cells experience (tension, shear, compression, substrate stiffness)."),
        ChannelState: ("en", "Channel state",
            "Coste et al. (2010): umbrella for the conformational states of a mechanosensitive channel."),
        FrequencyProperty: ("en", "Frequency property",
            "Lewis et al. (2017): umbrella for frequency-related channel properties (filtering, threshold, inactivation, recovery)."),
        CellularResponse: ("en", "Cellular response",
            "Iskratsch et al. (2014): umbrella for downstream cellular responses to mechanical stimulus."),
        MechanobiologyEvent: ("en", "Mechanobiology event",
            "Iskratsch et al. (2014): umbrella for time-extended mechanotransduction events (load, deformation, gating, influx, signalling, adaptation)."),

        MechanicalLoad: ("en", "Mechanical load",
            "Iskratsch et al. (2014): externally applied mechanical force on tissue or cell."),
        MembraneDeformation: ("en", "Membrane deformation",
            "Iskratsch et al. (2014): change in plasma-membrane geometry produced by mechanical load."),
        ChannelGating: ("en", "Channel gating",
            "Coste et al. (2010): transition of the mechanosensitive channel from closed to open conformation."),
        IonInflux: ("en", "Ion influx",
            "Coste et al. (2010): inward flow of cations through the open channel."),
        IntracellularSignaling: ("en", "Intracellular signaling",
            "Iskratsch et al. (2014): activation of downstream signalling cascades by Ca2+ and other second messengers."),
        RepetitiveStimulus: ("en", "Repetitive stimulus",
            "Lewis et al. (2017): repeated mechanical perturbations applied to the same channel population."),
        ChannelInactivation: ("en", "Channel inactivation",
            "Lewis et al. (2017): transition of the channel into the non-conducting inactivated state."),
        FrequencyDependentResponse: ("en", "Frequency-dependent response",
            "Lewis et al. (2017): the channel's response amplitude varies with stimulus frequency owing to its inactivation kinetics."),
        SustainedForce: ("en", "Sustained force",
            "Iskratsch et al. (2014): mechanical force applied chronically over hours or longer."),
        ThresholdShift: ("en", "Threshold shift",
            "Iskratsch et al. (2014): change in the activation threshold of mechanosensitive machinery in response to sustained force (mechanoadaptation)."),
    },

    is_a: [
        // Forces
        (MembraneTension, MechanicalForce),
        (ShearStress, MechanicalForce),
        (CompressiveStress, MechanicalForce),
        (TensileStress, MechanicalForce),
        (SubstrateStiffness, MechanicalForce),

        // Channel states
        (OpenState, ChannelState),
        (ClosedState, ChannelState),
        (InactivatedState, ChannelState),
        (MechanosensitiveChannel, ChannelState),
        (ChannelConformation, ChannelState),

        // Frequency properties
        (FrequencyFiltering, FrequencyProperty),
        (ActivationThreshold, FrequencyProperty),
        (InactivationKinetics, FrequencyProperty),
        (RecoveryTime, FrequencyProperty),

        // Cellular responses
        (CalciumTransient, CellularResponse),
        (CytoskeletalRemodeling, CellularResponse),
        (FocalAdhesion, CellularResponse),
        (Mechanoadaptation, CellularResponse),

        // Events under MechanobiologyEvent
        (MechanicalLoad, MechanobiologyEvent),
        (MembraneDeformation, MechanobiologyEvent),
        (ChannelGating, MechanobiologyEvent),
        (IonInflux, MechanobiologyEvent),
        (IntracellularSignaling, MechanobiologyEvent),
        (RepetitiveStimulus, MechanobiologyEvent),
        (ChannelInactivation, MechanobiologyEvent),
        (FrequencyDependentResponse, MechanobiologyEvent),
        (SustainedForce, MechanobiologyEvent),
        (ThresholdShift, MechanobiologyEvent),
    ],

    causes: [
        // Iskratsch et al. (2014) — canonical mechanotransduction chain.
        (MechanicalLoad, MembraneDeformation),
        (MembraneDeformation, ChannelGating),
        (ChannelGating, IonInflux),
        (IonInflux, IntracellularSignaling),
        // Lewis et al. (2017) — frequency-filtering branch.
        (RepetitiveStimulus, ChannelInactivation),
        (ChannelInactivation, FrequencyDependentResponse),
        // Iskratsch et al. (2014) — sustained-force / mechanoadaptation branch.
        (SustainedForce, MembraneDeformation),
        (SustainedForce, ThresholdShift),
    ],

    opposes: [
        // OpenState vs ClosedState — mutually exclusive conducting / non-
        // conducting conformations (Coste et al. 2010).
        (OpenState, ClosedState),
        (ClosedState, OpenState),
        // ActivationThreshold vs Mechanoadaptation — sustained force shifts
        // the threshold (Iskratsch et al. 2014); the static "threshold"
        // concept and the dynamic adaptation are dual.
        (ActivationThreshold, Mechanoadaptation),
        (Mechanoadaptation, ActivationThreshold),
    ],
}

// Backward-compatibility re-exports for partner functors / sibling crates
// that reference the legacy `*Entity` / `*CategoryRelationKind` names.
pub use MechanobiologyConcept as MechanobiologyEntity;
pub use MechanobiologyRelationKind as MechanobiologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: activation threshold in mN/m for mechanosensitive opening.
///
/// Lin, Buyan & Corry (2022/2023) PMID:37459546 — Piezo1 requires ~1–5 mN/m
/// membrane tension (midpoint ~3 mN/m) for half-maximal activation.
#[derive(Debug, Clone)]
pub struct ActivationThresholdValue;

impl Quality for ActivationThresholdValue {
    type Individual = MechanobiologyConcept;
    type Value = f64;

    fn get(&self, individual: &MechanobiologyConcept) -> Option<f64> {
        use MechanobiologyConcept::*;
        match individual {
            MembraneTension => Some(3.0),
            MechanosensitiveChannel => Some(3.0),
            ActivationThreshold => Some(3.0),
            _ => None,
        }
    }
}

/// Quality: is this entity frequency-dependent?
///
/// Lewis et al. (2017) — Piezo channels' inactivation kinetics determine
/// which frequencies the channel can follow.
#[derive(Debug, Clone)]
pub struct IsFrequencyDependent;

impl Quality for IsFrequencyDependent {
    type Individual = MechanobiologyConcept;
    type Value = bool;

    fn get(&self, individual: &MechanobiologyConcept) -> Option<bool> {
        use MechanobiologyConcept::*;
        match individual {
            MechanosensitiveChannel => Some(true),
            FrequencyFiltering => Some(true),
            InactivationKinetics => Some(true),
            RecoveryTime => Some(true),
            OpenState => Some(false),
            ClosedState => Some(false),
            _ => None,
        }
    }
}

/// Quality: inactivation time in milliseconds.
///
/// Lewis et al. (2017) — Piezo1 inactivation time constant ~15–30 ms
/// (midpoint ~20 ms), bounding the maximum stimulus frequency.
#[derive(Debug, Clone)]
pub struct InactivationTimeMs;

impl Quality for InactivationTimeMs {
    type Individual = MechanobiologyConcept;
    type Value = f64;

    fn get(&self, individual: &MechanobiologyConcept) -> Option<f64> {
        use MechanobiologyConcept::*;
        match individual {
            MechanosensitiveChannel => Some(20.0),
            InactivationKinetics => Some(20.0),
            _ => None,
        }
    }
}

/// Quality: does this entity require membrane tension to engage?
///
/// Coste et al. (2010); Lin, Buyan & Corry (2022/2023) — Piezo opening
/// requires membrane tension; downstream cellular structures (focal
/// adhesions, cytoskeleton) do not.
#[derive(Debug, Clone)]
pub struct RequiresMembraneTension;

impl Quality for RequiresMembraneTension {
    type Individual = MechanobiologyConcept;
    type Value = bool;

    fn get(&self, individual: &MechanobiologyConcept) -> Option<bool> {
        use MechanobiologyConcept::*;
        match individual {
            OpenState => Some(true),
            MechanosensitiveChannel => Some(true),
            ChannelConformation => Some(true),
            CytoskeletalRemodeling => Some(false),
            FocalAdhesion => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for MechanobiologyOntology {
    type Cat = MechanobiologyCategory;
    type Qual = ActivationThresholdValue;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MechanicalLoadCausesSignaling));
        axioms.push(Box::new(RepetitiveStimulusCausesFrequencyResponse));
        axioms.push(Box::new(MechanosensitiveChannelIsFrequencyDependent));
        axioms.push(Box::new(ChannelGatingRequiresTension));
        axioms.push(Box::new(SustainedForceCausesAdaptation));
        axioms
    }
}

/// Helper: does a `Causation` edge exist from `cause` to `effect`?
fn causes(cause: MechanobiologyConcept, effect: MechanobiologyConcept) -> bool {
    MechanobiologyCategory::morphisms().iter().any(|m| {
        m.kind() == MechanobiologyRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

/// Axiom: MechanicalLoad transitively causes IntracellularSignaling.
///
/// Iskratsch et al. (2014) — the canonical four-step chain
/// load → deformation → gating → influx → signalling.
pub struct MechanicalLoadCausesSignaling;

impl Axiom for MechanicalLoadCausesSignaling {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                MechanobiologyConcept::MechanicalLoad,
                MechanobiologyConcept::MembraneDeformation,
            ),
            (
                MechanobiologyConcept::MembraneDeformation,
                MechanobiologyConcept::ChannelGating,
            ),
            (
                MechanobiologyConcept::ChannelGating,
                MechanobiologyConcept::IonInflux,
            ),
            (
                MechanobiologyConcept::IonInflux,
                MechanobiologyConcept::IntracellularSignaling,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanicalLoadCausesSignaling",
        "Mechanical load transitively causes intracellular signalling via the deformation -> gating -> influx -> signalling chain",
        "Iskratsch, Wolfenson & Sheetz (2014) Nat. Rev. Mol. Cell Biol. 15(12):825-833"
    );
}

pr4xis::register_axiom!(
    MechanicalLoadCausesSignaling,
    "Iskratsch, Wolfenson & Sheetz (2014) Nat. Rev. Mol. Cell Biol. 15(12):825-833"
);

/// Axiom: RepetitiveStimulus transitively causes FrequencyDependentResponse.
///
/// Lewis et al. (2017) — Piezo channels filter repetitive stimuli through
/// their inactivation kinetics.
pub struct RepetitiveStimulusCausesFrequencyResponse;

impl Axiom for RepetitiveStimulusCausesFrequencyResponse {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                MechanobiologyConcept::RepetitiveStimulus,
                MechanobiologyConcept::ChannelInactivation,
            ),
            (
                MechanobiologyConcept::ChannelInactivation,
                MechanobiologyConcept::FrequencyDependentResponse,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RepetitiveStimulusCausesFrequencyResponse",
        "Repetitive mechanical stimulus drives channel inactivation, producing frequency-dependent response amplitudes",
        "Lewis, Cui & Grandl (2017) Cell Reports 19(12):2572-2585 (PMID:28636944)"
    );
}

pr4xis::register_axiom!(
    RepetitiveStimulusCausesFrequencyResponse,
    "Lewis, Cui & Grandl (2017) Cell Reports 19(12):2572-2585 (PMID:28636944)"
);

/// Axiom: MechanosensitiveChannel is frequency-dependent.
///
/// Lewis et al. (2017) — Piezo channels behave as low-pass filters whose
/// cutoff is determined by their inactivation kinetics.
pub struct MechanosensitiveChannelIsFrequencyDependent;

impl Axiom for MechanosensitiveChannelIsFrequencyDependent {
    fn verify(&self) -> Verdict {
        if IsFrequencyDependent.get(&MechanobiologyConcept::MechanosensitiveChannel) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanosensitiveChannelIsFrequencyDependent",
        "Mechanosensitive channels are frequency-dependent owing to their inactivation kinetics",
        "Lewis, Cui & Grandl (2017) Cell Reports 19(12):2572-2585 (PMID:28636944)"
    );
}

pr4xis::register_axiom!(
    MechanosensitiveChannelIsFrequencyDependent,
    "Lewis, Cui & Grandl (2017) Cell Reports 19(12):2572-2585 (PMID:28636944)"
);

/// Axiom: ChannelGating (OpenState) requires membrane tension.
///
/// Coste et al. (2010); Lin, Buyan & Corry (2022/2023) — opening a
/// mechanosensitive channel requires membrane tension above the activation
/// threshold.
pub struct ChannelGatingRequiresTension;

impl Axiom for ChannelGatingRequiresTension {
    fn verify(&self) -> Verdict {
        if RequiresMembraneTension.get(&MechanobiologyConcept::OpenState) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ChannelGatingRequiresTension",
        "Channel gating (entering the open state) requires membrane tension above the activation threshold",
        "Coste et al. (2010) Science 330(6000):55-60; Lin, Buyan & Corry (2022/2023) J. Gen. Physiol. (PMID:37459546)"
    );
}

pr4xis::register_axiom!(
    ChannelGatingRequiresTension,
    "Coste et al. (2010) Science 330(6000):55-60; Lin, Buyan & Corry (2022/2023) J. Gen. Physiol. (PMID:37459546)"
);

/// Axiom: SustainedForce causes ThresholdShift (mechanoadaptation).
///
/// Iskratsch et al. (2014) — chronic mechanical loading shifts the
/// activation threshold of mechanosensitive machinery.
pub struct SustainedForceCausesAdaptation;

impl Axiom for SustainedForceCausesAdaptation {
    fn verify(&self) -> Verdict {
        if causes(
            MechanobiologyConcept::SustainedForce,
            MechanobiologyConcept::ThresholdShift,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SustainedForceCausesAdaptation",
        "Sustained mechanical force shifts the activation threshold (mechanoadaptation)",
        "Iskratsch, Wolfenson & Sheetz (2014) Nat. Rev. Mol. Cell Biol. 15(12):825-833"
    );
}

pr4xis::register_axiom!(
    SustainedForceCausesAdaptation,
    "Iskratsch, Wolfenson & Sheetz (2014) Nat. Rev. Mol. Cell Biol. 15(12):825-833"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<MechanobiologyCategory>();
    }

    #[test]
    fn ontology_validates() {
        MechanobiologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 5 forces + 5 mechano-machinery + 4 frequency props + 4 cellular responses
        // + 5 abstract umbrellas (MechanicalForce, ChannelState, FrequencyProperty,
        //   CellularResponse, MechanobiologyEvent)
        // + 10 events = 33.
        assert_eq!(MechanobiologyConcept::variants().len(), 33);
    }

    // -- Domain axiom tests --

    #[test]
    fn mechanical_load_causes_signaling_axiom() {
        assert!(MechanicalLoadCausesSignaling.verify().is_ok());
    }

    #[test]
    fn repetitive_stimulus_causes_frequency_response_axiom() {
        assert!(RepetitiveStimulusCausesFrequencyResponse.verify().is_ok());
    }

    #[test]
    fn mechanosensitive_channel_is_frequency_dependent_axiom() {
        assert!(MechanosensitiveChannelIsFrequencyDependent.verify().is_ok());
    }

    #[test]
    fn channel_gating_requires_tension_axiom() {
        assert!(ChannelGatingRequiresTension.verify().is_ok());
    }

    #[test]
    fn sustained_force_causes_adaptation_axiom() {
        assert!(SustainedForceCausesAdaptation.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    #[test]
    fn forces_subsume_under_mechanical_force() {
        use MechanobiologyConcept::*;
        let subs: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for f in [
            MembraneTension,
            ShearStress,
            CompressiveStress,
            TensileStress,
            SubstrateStiffness,
        ] {
            assert!(
                subs.contains(&(f, MechanicalForce)),
                "{:?} should subsume under MechanicalForce",
                f
            );
        }
    }

    #[test]
    fn channel_states_subsume_under_channel_state() {
        use MechanobiologyConcept::*;
        let subs: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for s in [
            OpenState,
            ClosedState,
            InactivatedState,
            MechanosensitiveChannel,
            ChannelConformation,
        ] {
            assert!(
                subs.contains(&(s, ChannelState)),
                "{:?} should subsume under ChannelState",
                s
            );
        }
    }

    #[test]
    fn events_subsume_under_mechanobiology_event() {
        use MechanobiologyConcept::*;
        let subs: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
            MechanicalLoad,
            MembraneDeformation,
            ChannelGating,
            IonInflux,
            IntracellularSignaling,
            RepetitiveStimulus,
            ChannelInactivation,
            FrequencyDependentResponse,
            SustainedForce,
            ThresholdShift,
        ] {
            assert!(
                subs.contains(&(ev, MechanobiologyEvent)),
                "{:?} should subsume under MechanobiologyEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[test]
    fn mechanical_load_directly_causes_membrane_deformation() {
        assert!(causes(
            MechanobiologyConcept::MechanicalLoad,
            MechanobiologyConcept::MembraneDeformation
        ));
    }

    #[test]
    fn channel_gating_causes_ion_influx() {
        assert!(causes(
            MechanobiologyConcept::ChannelGating,
            MechanobiologyConcept::IonInflux
        ));
    }

    #[test]
    fn sustained_force_causes_threshold_shift() {
        assert!(causes(
            MechanobiologyConcept::SustainedForce,
            MechanobiologyConcept::ThresholdShift
        ));
    }

    #[test]
    fn intracellular_signaling_does_not_cause_mechanical_load() {
        assert!(!causes(
            MechanobiologyConcept::IntracellularSignaling,
            MechanobiologyConcept::MechanicalLoad
        ));
    }

    // -- Opposition-kind tests --

    #[test]
    fn open_and_closed_state_oppose() {
        let opps: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            MechanobiologyConcept::OpenState,
            MechanobiologyConcept::ClosedState
        )));
        assert!(opps.contains(&(
            MechanobiologyConcept::ClosedState,
            MechanobiologyConcept::OpenState
        )));
    }

    #[test]
    fn threshold_and_mechanoadaptation_oppose() {
        let opps: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            MechanobiologyConcept::ActivationThreshold,
            MechanobiologyConcept::Mechanoadaptation
        )));
    }

    #[test]
    fn open_does_not_oppose_inactivated() {
        let opps: Vec<_> = MechanobiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MechanobiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(!opps.contains(&(
            MechanobiologyConcept::OpenState,
            MechanobiologyConcept::InactivatedState
        )));
    }

    // -- Quality tests --

    #[test]
    fn membrane_tension_threshold() {
        assert_eq!(
            ActivationThresholdValue.get(&MechanobiologyConcept::MembraneTension),
            Some(3.0)
        );
    }

    #[test]
    fn channel_threshold() {
        assert_eq!(
            ActivationThresholdValue.get(&MechanobiologyConcept::MechanosensitiveChannel),
            Some(3.0)
        );
    }

    #[test]
    fn channel_is_frequency_dependent() {
        assert_eq!(
            IsFrequencyDependent.get(&MechanobiologyConcept::MechanosensitiveChannel),
            Some(true)
        );
    }

    #[test]
    fn open_state_not_frequency_dependent() {
        assert_eq!(
            IsFrequencyDependent.get(&MechanobiologyConcept::OpenState),
            Some(false)
        );
    }

    #[test]
    fn inactivation_time() {
        assert_eq!(
            InactivationTimeMs.get(&MechanobiologyConcept::MechanosensitiveChannel),
            Some(20.0)
        );
    }

    #[test]
    fn open_state_requires_tension() {
        assert_eq!(
            RequiresMembraneTension.get(&MechanobiologyConcept::OpenState),
            Some(true)
        );
    }

    #[test]
    fn cytoskeletal_remodeling_no_tension() {
        assert_eq!(
            RequiresMembraneTension.get(&MechanobiologyConcept::CytoskeletalRemodeling),
            Some(false)
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = MechanobiologyConcept> {
        proptest::sample::select(MechanobiologyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MechanobiologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MechanobiologyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = MechanobiologyConcept::variants();
            for m in MechanobiologyCategory::morphisms() {
                if m.kind() == MechanobiologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = MechanobiologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == MechanobiologyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(
                    opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} -> {:?} but not back",
                    a,
                    b
                );
            }
        }

        /// ActivationThresholdValue is positive whenever defined.
        #[test]
        fn prop_threshold_always_positive(c in arb_concept()) {
            if let Some(t) = ActivationThresholdValue.get(&c) {
                prop_assert!(t > 0.0, "activation threshold must be positive for {:?}", c);
            }
        }

        /// InactivationTimeMs is positive whenever defined.
        #[test]
        fn prop_inactivation_time_positive(c in arb_concept()) {
            if let Some(t) = InactivationTimeMs.get(&c) {
                prop_assert!(t > 0.0, "inactivation time must be positive for {:?}", c);
            }
        }
    }
}
