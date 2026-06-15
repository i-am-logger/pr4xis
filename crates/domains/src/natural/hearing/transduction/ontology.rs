//! Mechanotransduction — conversion of mechanical vibration into neural
//! signals in cochlear hair cells.
//!
//! # Literature
//!
//! - **Hudspeth (1989)** "How the ear's works work", *Nature*
//!   341(6241):397-404.
//! - **Hudspeth (2014)** "Integrating the active process of hair cells
//!   with cochlear function", *Nat. Rev. Neurosci.* 15(9):600-614.
//! - **Fettiplace & Kim (2014)** "The Physiology of Mechanoelectrical
//!   Transduction Channels in Hearing", *Physiol. Rev.* 94(3):951-986.
//! - **Pan et al. (2013)** "TMC1 and TMC2 are components of the
//!   mechanotransduction channel in hair cells of the mammalian inner
//!   ear", *Neuron* 79(3):504-515.
//! - **Zheng et al. (2000)** "Prestin is the motor protein of cochlear
//!   outer hair cells", *Nature* 405(6783):149-155.
//! - **Kubisch et al. (1999)** "KCNQ4, a novel potassium channel
//!   expressed in sensory outer hair cells", *Cell* 96(3):437-446.
//! - **von Bekesy (1952)** "DC Resting Potentials Inside the Cochlear
//!   Partition", *JASA* 24(1):72-76.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Transduction",
    source: "Hudspeth (1989) Nature 341(6241):397; Hudspeth (2014) Nat. Rev. Neurosci. 15(9):600; Fettiplace & Kim (2014) Physiol. Rev. 94(3):951; Pan et al. (2013) Neuron 79(3):504; Zheng et al. (2000) Nature 405(6783):149; Kubisch et al. (1999) Cell 96(3):437; von Bekesy (1952) JASA 24(1):72",

    concepts: [
        Stereocilium, StereociliaBundle, TipLink, Kinocilium, CuticularPlate,
        Cadherin23, Protocadherin15,
        METChannel, TMC1, TMC2, TMIE, LHFPL5,
        KCNQ4, CaV1_3, BKChannel,
        Prestin,
        Potassium, Calcium, Glutamate,
        EndocochlearPotential, ReceptorPotential,
        StereociliaDeflection, TipLinkTension, METChannelOpening,
        PotassiumInflux, CalciumInflux, Depolarization,
        GlutamateRelease, ActionPotential,
        Electromotility, CochlearAmplification,
        // Umbrellas
        METComponent, IonChannel, TipLinkProtein, CellularSignal,
        // Events
        BasilarMembraneMotion, StereociliaBundleDeflection, TipLinkStretch,
        METChannelGating, PotassiumEntry, CellDepolarization, CalciumEntry,
        VesicleRelease, NerveActivation,
        PrestiConformationChange, CellLengthChange, BasilarMembraneAmplification,
        SlowAdaptation, FastAdaptation,
        TransductionEvent,
    ],

    labels: {
        Stereocilium: ("en", "Stereocilium",
            "Hudspeth (2014) Nat. Rev. Neurosci. 15(9):600 — actin-cored hair-cell process."),
        StereociliaBundle: ("en", "Stereocilia bundle",
            "Hudspeth (2014): graded-height bundle of stereocilia."),
        TipLink: ("en", "Tip link",
            "Pickles et al. (1984): 207 nm extracellular filament linking adjacent stereocilia."),
        Kinocilium: ("en", "Kinocilium",
            "Hudspeth (2014): true cilium present during development."),
        CuticularPlate: ("en", "Cuticular plate",
            "Hudspeth (2014): apical actin meshwork anchoring stereocilia."),
        Cadherin23: ("en", "Cadherin-23",
            "Siemens et al. (2004) Nature 428(6986):950 — upper tip-link protein."),
        Protocadherin15: ("en", "Protocadherin-15",
            "Ahmed et al. (2006) J. Neurosci. 26(26):7022 — lower tip-link protein."),
        METChannel: ("en", "MET channel",
            "Fettiplace & Kim (2014) Physiol. Rev. 94(3):951 — mechanoelectrical-transduction channel."),
        TMC1: ("en", "TMC1",
            "Pan et al. (2013) Neuron 79(3):504 — pore-forming MET subunit."),
        TMC2: ("en", "TMC2",
            "Pan et al. (2013): second pore-forming MET subunit."),
        TMIE: ("en", "TMIE",
            "Zhao et al. (2014) Neuron 84(5):954 — auxiliary MET subunit."),
        LHFPL5: ("en", "LHFPL5",
            "Xiong et al. (2012) Cell 151(6):1283 — auxiliary MET subunit (TMHS)."),
        KCNQ4: ("en", "KCNQ4",
            "Kubisch et al. (1999) Cell 96(3):437 — K+ channel in OHCs."),
        CaV1_3: ("en", "CaV1.3",
            "Platzer et al. (2000) Cell 102(1):89 — L-type Ca²⁺ channel in IHCs."),
        BKChannel: ("en", "BK channel",
            "Pyott et al. (2007): large-conductance Ca²⁺-activated K⁺ channel."),
        Prestin: ("en", "Prestin",
            "Zheng et al. (2000) Nature 405(6783):149 — SLC26A5 OHC motor protein."),
        Potassium: ("en", "Potassium",
            "Hudspeth (2014): K⁺ ion — primary MET-channel carrier."),
        Calcium: ("en", "Calcium",
            "Hudspeth (2014): Ca²⁺ — secondary MET-channel carrier; vesicle-release trigger."),
        Glutamate: ("en", "Glutamate",
            "Glowatzki & Fuchs (2002) Nat. Neurosci. 5(2):147 — IHC afferent neurotransmitter."),
        EndocochlearPotential: ("en", "Endocochlear potential",
            "von Bekesy (1952) JASA 24(1):72 — +80 mV in scala media."),
        ReceptorPotential: ("en", "Receptor potential",
            "Hudspeth (2014): hair-cell membrane-potential change."),
        StereociliaDeflection: ("en", "Stereocilia deflection",
            "Hudspeth (2014): bundle pivoting from BM motion."),
        TipLinkTension: ("en", "Tip-link tension",
            "Hudspeth (2014): mechanical loading of tip links."),
        METChannelOpening: ("en", "MET channel opening",
            "Fettiplace & Kim (2014): tip-link-tension-gated channel opening."),
        PotassiumInflux: ("en", "Potassium influx",
            "Hudspeth (2014): K⁺ entry through open MET channels."),
        CalciumInflux: ("en", "Calcium influx",
            "Hudspeth (2014): Ca²⁺ entry through MET (or voltage-gated) channels."),
        Depolarization: ("en", "Depolarization",
            "Hudspeth (2014): hair-cell membrane potential rise."),
        GlutamateRelease: ("en", "Glutamate release",
            "Glowatzki & Fuchs (2002): vesicle-fusion-driven glutamate output at the IHC ribbon."),
        ActionPotential: ("en", "Action potential",
            "Hudspeth (2014): all-or-none spiral-ganglion afferent spike."),
        Electromotility: ("en", "Electromotility",
            "Zheng et al. (2000): prestin-driven OHC length change in response to voltage."),
        CochlearAmplification: ("en", "Cochlear amplification",
            "Hudspeth (2014): OHC-electromotility-driven BM-motion amplification."),
        METComponent: ("en", "MET component",
            "Pan et al. (2013): umbrella for MET-channel subunits."),
        IonChannel: ("en", "Ion channel",
            "Hudspeth (2014): umbrella for hair-cell ion channels."),
        TipLinkProtein: ("en", "Tip-link protein",
            "Siemens et al. (2004): umbrella for tip-link extracellular proteins."),
        CellularSignal: ("en", "Cellular signal",
            "Hudspeth (2014): umbrella for hair-cell intracellular signals."),
        BasilarMembraneMotion: ("en", "Basilar-membrane motion",
            "von Bekesy (1960): event — BM travelling-wave displacement."),
        StereociliaBundleDeflection: ("en", "Stereocilia bundle deflection",
            "Hudspeth (2014): event — bundle pivot at the apical pole."),
        TipLinkStretch: ("en", "Tip-link stretch",
            "Hudspeth (2014): event — extension of tip links by bundle deflection."),
        METChannelGating: ("en", "MET channel gating",
            "Fettiplace & Kim (2014): event — channel opening/closing."),
        PotassiumEntry: ("en", "Potassium entry",
            "Hudspeth (2014): event — K⁺ flow through open MET channels."),
        CellDepolarization: ("en", "Cell depolarization",
            "Hudspeth (2014): event — hair-cell voltage shift toward zero."),
        CalciumEntry: ("en", "Calcium entry",
            "Hudspeth (2014): event — Ca²⁺ entry near the ribbon synapse."),
        VesicleRelease: ("en", "Vesicle release",
            "Glowatzki & Fuchs (2002): event — synaptic vesicle fusion."),
        NerveActivation: ("en", "Nerve activation",
            "Hudspeth (2014): terminal event — spiral-ganglion-afferent firing."),
        PrestiConformationChange: ("en", "Prestin conformation change",
            "Zheng et al. (2000): event — prestin voltage-driven shape change."),
        CellLengthChange: ("en", "Cell length change",
            "Zheng et al. (2000): event — OHC contraction/elongation."),
        BasilarMembraneAmplification: ("en", "BM amplification",
            "Hudspeth (2014): event — feedback BM-motion boost from OHC electromotility."),
        SlowAdaptation: ("en", "Slow adaptation",
            "Eatock (2000): event — Ca²⁺-dependent slow MET adaptation."),
        FastAdaptation: ("en", "Fast adaptation",
            "Eatock (2000): event — Ca²⁺-dependent fast MET adaptation."),
        TransductionEvent: ("en", "Transduction event",
            "Hudspeth (2014): umbrella concept for hair-cell perdurants."),
    },

    is_a: [
        (TMC1, METComponent), (TMC2, METComponent),
        (TMIE, METComponent), (LHFPL5, METComponent),
        (METChannel, IonChannel),
        (Cadherin23, TipLinkProtein), (Protocadherin15, TipLinkProtein),
        (KCNQ4, IonChannel), (CaV1_3, IonChannel), (BKChannel, IonChannel),
        (PotassiumInflux, CellularSignal), (CalciumInflux, CellularSignal),
        (Depolarization, CellularSignal), (GlutamateRelease, CellularSignal),
        (ActionPotential, CellularSignal), (ReceptorPotential, CellularSignal),
        (BasilarMembraneMotion, TransductionEvent),
        (StereociliaBundleDeflection, TransductionEvent),
        (TipLinkStretch, TransductionEvent),
        (METChannelGating, TransductionEvent),
        (PotassiumEntry, TransductionEvent), (CellDepolarization, TransductionEvent),
        (CalciumEntry, TransductionEvent), (VesicleRelease, TransductionEvent),
        (NerveActivation, TransductionEvent),
        (PrestiConformationChange, TransductionEvent),
        (CellLengthChange, TransductionEvent),
        (BasilarMembraneAmplification, TransductionEvent),
        (SlowAdaptation, TransductionEvent), (FastAdaptation, TransductionEvent),
    ],

    has_a: [
        (StereociliaBundle, Stereocilium), (StereociliaBundle, TipLink),
        (StereociliaBundle, Kinocilium),
        (TipLink, Cadherin23), (TipLink, Protocadherin15),
        (METChannel, TMC1), (METChannel, TMC2),
        (METChannel, TMIE), (METChannel, LHFPL5),
    ],

    causes: [
        (BasilarMembraneMotion, StereociliaBundleDeflection),
        (StereociliaBundleDeflection, TipLinkStretch),
        (TipLinkStretch, METChannelGating),
        (METChannelGating, PotassiumEntry),
        (PotassiumEntry, CellDepolarization),
        (CellDepolarization, CalciumEntry),
        (CalciumEntry, VesicleRelease),
        (VesicleRelease, NerveActivation),
        (CellDepolarization, PrestiConformationChange),
        (PrestiConformationChange, CellLengthChange),
        (CellLengthChange, BasilarMembraneAmplification),
        (CalciumEntry, SlowAdaptation),
        (CalciumEntry, FastAdaptation),
    ],

    opposes: [
        (PotassiumInflux, CalciumInflux), (CalciumInflux, PotassiumInflux),
        (Electromotility, ActionPotential), (ActionPotential, Electromotility),
    ],
}

#[derive(Debug, Clone)]
pub struct RestingPotential;
impl Quality for RestingPotential {
    type Individual = TransductionConcept;
    type Value = f64;
    fn get(&self, individual: &TransductionConcept) -> Option<f64> {
        use TransductionConcept::*;
        match individual {
            EndocochlearPotential => Some(80.0),
            Potassium => Some(-90.0),
            Calcium => Some(131.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TipLinkLength;
impl Quality for TipLinkLength {
    type Individual = TransductionConcept;
    type Value = f64;
    fn get(&self, individual: &TransductionConcept) -> Option<f64> {
        use TransductionConcept::*;
        match individual {
            Cadherin23 => Some(170.0),
            Protocadherin15 => Some(37.0),
            TipLink => Some(207.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelConductance;
impl Quality for ChannelConductance {
    type Individual = TransductionConcept;
    type Value = f64;
    fn get(&self, individual: &TransductionConcept) -> Option<f64> {
        use TransductionConcept::*;
        match individual {
            METChannel => Some(150.0),
            TMC1 => Some(150.0),
            KCNQ4 => Some(10.0),
            BKChannel => Some(250.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IsOHCSpecific;
impl Quality for IsOHCSpecific {
    type Individual = TransductionConcept;
    type Value = bool;
    fn get(&self, individual: &TransductionConcept) -> Option<bool> {
        use TransductionConcept::*;
        Some(matches!(
            individual,
            Prestin | Electromotility | CochlearAmplification
        ))
    }
}

fn is_a(child: TransductionConcept, parent: TransductionConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    TransductionCategory::morphisms().iter().any(|m| {
        m.kind() == TransductionRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn parts_of(whole: TransductionConcept) -> Vec<TransductionConcept> {
    use pr4xis::category::{Arrow, Category};
    TransductionCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == TransductionRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn effects_of(cause: TransductionConcept) -> Vec<TransductionConcept> {
    use pr4xis::category::{Arrow, Category};
    TransductionCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == TransductionRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct BundleContainsTipLinkProteins;
impl Axiom for BundleContainsTipLinkProteins {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use TransductionConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(StereociliaBundle);
        if parts.contains(&Cadherin23) && parts.contains(&Protocadherin15) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BundleContainsTipLinkProteins",
        "stereocilia bundle transitively contains cadherin-23 and protocadherin-15",
        "Siemens et al. (2004) Nature 428(6986):950; Ahmed et al. (2006) J. Neurosci. 26(26):7022"
    );
}
pr4xis::register_axiom!(
    BundleContainsTipLinkProteins,
    "Siemens et al. (2004) Nature 428(6986):950; Ahmed et al. (2006) J. Neurosci. 26(26):7022"
);

pub struct TMCsAreMETComponents;
impl Axiom for TMCsAreMETComponents {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use TransductionConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if is_a(TMC1, METComponent) && is_a(TMC2, METComponent) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "TMCsAreMETComponents",
        "TMC1 and TMC2 are components of the MET channel",
        "Pan et al. (2013) Neuron 79(3):504"
    );
}
pr4xis::register_axiom!(TMCsAreMETComponents, "Pan et al. (2013) Neuron 79(3):504");

pub struct BMMotionCausesNerveActivation;
impl Axiom for BMMotionCausesNerveActivation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use TransductionConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(BasilarMembraneMotion).contains(&NerveActivation) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BMMotionCausesNerveActivation",
        "basilar membrane motion transitively causes nerve activation",
        "Hudspeth (2014) Nat. Rev. Neurosci. 15(9):600"
    );
}
pr4xis::register_axiom!(
    BMMotionCausesNerveActivation,
    "Hudspeth (2014) Nat. Rev. Neurosci. 15(9):600"
);

pub struct DepolarizationCausesElectromotility;
impl Axiom for DepolarizationCausesElectromotility {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use TransductionConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(CellDepolarization).contains(&PrestiConformationChange) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "DepolarizationCausesElectromotility",
        "cell depolarization causes prestin conformational change",
        "Zheng et al. (2000) Nature 405(6783):149"
    );
}
pr4xis::register_axiom!(
    DepolarizationCausesElectromotility,
    "Zheng et al. (2000) Nature 405(6783):149"
);

pub struct EndocochlearPotentialIsPositive;
impl Axiom for EndocochlearPotentialIsPositive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let v = RestingPotential
            .get(&TransductionConcept::EndocochlearPotential)
            .unwrap_or(0.0);
        if v > 0.0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "EndocochlearPotentialIsPositive",
        "endocochlear potential is positive (+80 mV)",
        "von Bekesy (1952) JASA 24(1):72"
    );
}
pr4xis::register_axiom!(
    EndocochlearPotentialIsPositive,
    "von Bekesy (1952) JASA 24(1):72"
);

pub struct PrestiIsOHCSpecific;
impl Axiom for PrestiIsOHCSpecific {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if IsOHCSpecific.get(&TransductionConcept::Prestin) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "PrestiIsOHCSpecific",
        "prestin is specific to outer hair cells",
        "Zheng et al. (2000) Nature 405(6783):149"
    );
}
pr4xis::register_axiom!(
    PrestiIsOHCSpecific,
    "Zheng et al. (2000) Nature 405(6783):149"
);

impl Ontology for TransductionOntology {
    type Cat = TransductionCategory;
    type Qual = ChannelConductance;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(BundleContainsTipLinkProteins));
        a.push(Box::new(TMCsAreMETComponents));
        a.push(Box::new(BMMotionCausesNerveActivation));
        a.push(Box::new(DepolarizationCausesElectromotility));
        a.push(Box::new(EndocochlearPotentialIsPositive));
        a.push(Box::new(PrestiIsOHCSpecific));
        a
    }
}

// Back-compat aliases.
pub use TransductionConcept as TransductionEntity;
pub use TransductionRelationKind as TransductionCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TransductionCategory>();
    }
    #[test]
    fn ontology_validates() {
        TransductionOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[test]
    fn bundle_contains_tip_link_proteins() {
        assert!(BundleContainsTipLinkProteins.verify().is_ok());
    }
    #[test]
    fn tmcs_are_met_components() {
        assert!(TMCsAreMETComponents.verify().is_ok());
    }
    #[test]
    fn bm_motion_causes_nerve_activation() {
        assert!(BMMotionCausesNerveActivation.verify().is_ok());
    }
    #[test]
    fn depolarization_causes_electromotility() {
        assert!(DepolarizationCausesElectromotility.verify().is_ok());
    }
    #[test]
    fn endocochlear_potential_positive() {
        assert!(EndocochlearPotentialIsPositive.verify().is_ok());
    }
    #[test]
    fn prestin_ohc_specific() {
        assert!(PrestiIsOHCSpecific.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in TransductionCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TransductionOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
