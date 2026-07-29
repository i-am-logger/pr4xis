//! Morphospace attractor-landscape ontology.
//!
//! Models the esophageal tissue morphospace as a set of attractor states
//! (healthy, inflamed, Barrett's, dysplastic, fibrotic), the disease-
//! progression causal chain that drives the tissue from healthy into
//! pathological attractors, and the repair pathways that drive it back.
//! Pure ontology — no simulation.
//!
//! Per `feedback_one_ontology_per_module` the original split between
//! `MorphospaceEntity` and `MorphospaceEvent` has been merged: events are
//! first-class concepts subsumed by the `MorphospaceEvent` umbrella.
//!
//! # Literature
//!
//! - **Fields & Levin (2022)** "Competency in Navigating Arbitrary Spaces
//!   as an Invariant for Analyzing Cognition in Diverse Embodiments",
//!   *Entropy* 24(6):819 — morphospace as the state-space the tissue
//!   navigates; attractors as goal states.
//! - **Chernet & Levin (2013)** "Endogenous Voltage Potentials and the
//!   Microenvironment", *J. Clin. Exp. Oncol.* S1:002 — Vmem ranges
//!   that characterise healthy and pathological tissue.
//! - **Levin (2015)** "The wisdom of the body: future techniques and
//!   approaches to morphogenetic fields in regenerative medicine,
//!   developmental biology and cancer", *Regenerative Medicine*
//!   10(2):105–110 — gap-junction blockade induces species-specific
//!   head anatomies, proving bistability in morphospace.
//! - **Gralnek et al. (2006)** "Esomeprazole 20mg vs. omeprazole 20mg in
//!   the treatment of erosive esophagitis", *Aliment. Pharmacol. Ther.*
//!   23(1):149–157 — PPI heals erosive esophagitis via acid removal and
//!   basal-cell turnover (GJ-independent repair pathway).

use crate::formal::math::quantity::unit::{MILLIVOLT, UNITLESS};
use crate::formal::math::quantity::value::{Quantity, QuantityRange};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Morphospace",
    source: "Fields & Levin (2022) Entropy 24(6):819; Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Levin (2015) Regen. Med. 10(2):105–110; Gralnek et al. (2006) Aliment. Pharmacol. Ther. 23(1):149–157",

    concepts: [
        // === Attractors ===
        Healthy,
        Inflamed,
        Barretts,
        Dysplastic,
        Fibrotic,

        // === Repair pathways ===
        BasalTurnover,
        BioelectricRepair,
        MechanicalStimulationPathway,
        CombinedTherapy,

        // === Bioelectric states ===
        PolarizedVmem,
        DepolarizedVmem,
        ConnectedNetwork,
        DisconnectedNetwork,

        // === Abstract umbrellas ===
        Attractor,
        RepairPathway,
        BioelectricState,
        MorphospaceEvent,

        // === Disease-progression events (merged from MorphospaceEvent) ===
        AcidDamage,
        ChronicInflammation,
        GapJunctionLoss,
        MetaplasticTransition,
        DysplasticTransition,
        FibroticRemodeling,

        // === Repair events ===
        AcidRemoval,
        BasalCellReplacement,
        VmemRepolarization,
        GapJunctionRestoration,
        PatternRecognition,
        MechanotransductionActivation,
        AutonomousRepair,
    ],

    labels: {
        Healthy: ("en", "Healthy",
            "Fields & Levin (2022): healthy attractor — polarised Vmem, connected network, no disease severity."),
        Inflamed: ("en", "Inflamed",
            "Chernet & Levin (2013): inflamed attractor — partial depolarisation, mild severity."),
        Barretts: ("en", "Barrett's",
            "Chernet & Levin (2013): Barrett's-metaplasia attractor — further depolarised, metaplastic columnar lining."),
        Dysplastic: ("en", "Dysplastic",
            "Chernet & Levin (2013): dysplastic attractor — strongly depolarised, pre-cancerous."),
        Fibrotic: ("en", "Fibrotic",
            "Levin (2014): fibrotic attractor — depolarised, scarred tissue."),

        BasalTurnover: ("en", "Basal turnover",
            "Gralnek et al. (2006): repair pathway via acid removal and basal-cell turnover — GJ-independent."),
        BioelectricRepair: ("en", "Bioelectric repair",
            "Chernet & Levin (2013): repair pathway via Vmem normalisation through gap-junction-mediated re-polarisation."),
        MechanicalStimulationPathway: ("en", "Mechanical stimulation",
            "Levin (2014): hardware-accessible repair pathway via mechanotransduction."),
        CombinedTherapy: ("en", "Combined therapy",
            "Chernet & Levin (2013): combined bioelectric + acid-removal therapy."),

        PolarizedVmem: ("en", "Polarised Vmem",
            "Chernet & Levin (2013): bioelectric state — Vmem more negative than −40 mV."),
        DepolarizedVmem: ("en", "Depolarised Vmem",
            "Chernet & Levin (2013): bioelectric state — Vmem more positive than ~−20 mV."),
        ConnectedNetwork: ("en", "Connected network",
            "Levin (2019): bioelectric state — gap-junction network intact."),
        DisconnectedNetwork: ("en", "Disconnected network",
            "Levin (2019): bioelectric state — gap junctions blocked or absent."),

        Attractor: ("en", "Attractor",
            "Fields & Levin (2022): umbrella for stable tissue configurations in morphospace."),
        RepairPathway: ("en", "Repair pathway",
            "Levin (2014): umbrella for trajectories from a pathological attractor back to the healthy attractor."),
        BioelectricState: ("en", "Bioelectric state",
            "Levin (2019): umbrella for instantaneous bioelectric properties of tissue."),
        MorphospaceEvent: ("en", "Morphospace event",
            "Fields & Levin (2022): umbrella for time-extended processes in disease progression and repair."),

        AcidDamage: ("en", "Acid damage",
            "Gralnek et al. (2006): mucosal acid injury initiates the disease cascade."),
        ChronicInflammation: ("en", "Chronic inflammation",
            "Gralnek et al. (2006): persistent inflammatory state following repeated acid injury."),
        GapJunctionLoss: ("en", "Gap-junction loss",
            "Levin (2015): connexin downregulation that disconnects the bioelectric network."),
        MetaplasticTransition: ("en", "Metaplastic transition",
            "Chernet & Levin (2013): transition from squamous to columnar epithelium (Barrett's)."),
        DysplasticTransition: ("en", "Dysplastic transition",
            "Chernet & Levin (2013): transition from Barrett's to dysplasia — increased depolarisation."),
        FibroticRemodeling: ("en", "Fibrotic remodeling",
            "Levin (2014): tissue replacement by fibrotic scar."),

        AcidRemoval: ("en", "Acid removal",
            "Gralnek et al. (2006): PPI-mediated acid removal — initiates basal-cell turnover repair."),
        BasalCellReplacement: ("en", "Basal-cell replacement",
            "Gralnek et al. (2006): basal stem-cell proliferation replaces damaged epithelium."),
        VmemRepolarization: ("en", "Vmem repolarisation",
            "Chernet & Levin (2013): Vmem restored to polarised (healthy) range."),
        GapJunctionRestoration: ("en", "Gap-junction restoration",
            "Levin (2019): re-establishment of connexin-mediated cell coupling."),
        PatternRecognition: ("en", "Pattern recognition",
            "Levin (2014): bioelectric network re-reads the target morphology."),
        MechanotransductionActivation: ("en", "Mechanotransduction activation",
            "Levin (2014): mechanical stimulation activates downstream bioelectric repair."),
        AutonomousRepair: ("en", "Autonomous repair",
            "Fields & Levin (2022): tissue navigates back to the healthy attractor without further intervention."),
    },

    is_a: [
        // Attractors.
        (Healthy, Attractor),
        (Inflamed, Attractor),
        (Barretts, Attractor),
        (Dysplastic, Attractor),
        (Fibrotic, Attractor),
        // Repair pathways.
        (BasalTurnover, RepairPathway),
        (BioelectricRepair, RepairPathway),
        (MechanicalStimulationPathway, RepairPathway),
        (CombinedTherapy, RepairPathway),
        // Bioelectric states.
        (PolarizedVmem, BioelectricState),
        (DepolarizedVmem, BioelectricState),
        (ConnectedNetwork, BioelectricState),
        (DisconnectedNetwork, BioelectricState),
        // Events.
        (AcidDamage, MorphospaceEvent),
        (ChronicInflammation, MorphospaceEvent),
        (GapJunctionLoss, MorphospaceEvent),
        (MetaplasticTransition, MorphospaceEvent),
        (DysplasticTransition, MorphospaceEvent),
        (FibroticRemodeling, MorphospaceEvent),
        (AcidRemoval, MorphospaceEvent),
        (BasalCellReplacement, MorphospaceEvent),
        (VmemRepolarization, MorphospaceEvent),
        (GapJunctionRestoration, MorphospaceEvent),
        (PatternRecognition, MorphospaceEvent),
        (MechanotransductionActivation, MorphospaceEvent),
        (AutonomousRepair, MorphospaceEvent),
    ],

    causes: [
        // Disease progression (Chernet & Levin 2013; Levin 2015).
        (AcidDamage, ChronicInflammation),
        (ChronicInflammation, GapJunctionLoss),
        (GapJunctionLoss, FibroticRemodeling),
        (ChronicInflammation, MetaplasticTransition),
        (MetaplasticTransition, DysplasticTransition),
        // Acid-removal repair pathway (Gralnek et al. 2006).
        (AcidRemoval, BasalCellReplacement),
        (BasalCellReplacement, AutonomousRepair),
        // Bioelectric repair pathway (Chernet & Levin 2013; Levin 2019).
        (VmemRepolarization, PatternRecognition),
        (GapJunctionRestoration, PatternRecognition),
        (PatternRecognition, AutonomousRepair),
        // Mechanotransduction-triggered repair (Levin 2014).
        (MechanotransductionActivation, VmemRepolarization),
        (MechanotransductionActivation, GapJunctionRestoration),
    ],

    opposes: [
        // Healthy ↔ Dysplastic: end-points of the main disease axis.
        (Healthy, Dysplastic),
        (Dysplastic, Healthy),
        // Polarised ↔ Depolarised Vmem.
        (PolarizedVmem, DepolarizedVmem),
        (DepolarizedVmem, PolarizedVmem),
        // Connected ↔ Disconnected network.
        (ConnectedNetwork, DisconnectedNetwork),
        (DisconnectedNetwork, ConnectedNetwork),
        // Acid-removal pathway (no Vmem manipulation) vs bioelectric
        // repair (direct Vmem manipulation) — two complementary mechanisms.
        (BasalTurnover, BioelectricRepair),
        (BioelectricRepair, BasalTurnover),
    ],
}

// Backward-compatibility re-exports.
pub use MorphospaceConcept as MorphospaceEntity;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: characteristic Vmem range for each attractor state, as a
/// [`QuantityRange`] in [`MILLIVOLT`] (Chernet & Levin 2013), NOT a bare
/// `(f64, f64)` pair.
#[derive(Debug, Clone)]
pub struct AttractorVmemRange;

impl Quality for AttractorVmemRange {
    type Individual = MorphospaceConcept;
    type Value = QuantityRange;

    fn get(&self, individual: &MorphospaceConcept) -> Option<QuantityRange> {
        use MorphospaceConcept::*;
        let mv = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &MILLIVOLT),
            max: Quantity::from_unit(hi, &MILLIVOLT),
        };
        match individual {
            Healthy => Some(mv(-70.0, -40.0)),
            Inflamed => Some(mv(-40.0, -28.0)),
            Barretts => Some(mv(-28.0, -18.0)),
            Dysplastic => Some(mv(-18.0, 0.0)),
            Fibrotic => Some(mv(-35.0, -20.0)),
            _ => None,
        }
    }
}

/// Quality: disease severity (0 = healthy, higher = worse).
///
/// Chernet & Levin (2013) — severity tracks depolarisation along the
/// canonical Healthy → Inflamed → Barretts → Dysplastic axis.
#[derive(Debug, Clone)]
pub struct DiseaseSeverity;

impl Quality for DiseaseSeverity {
    type Individual = MorphospaceConcept;
    type Value = Quantity;

    fn get(&self, individual: &MorphospaceConcept) -> Option<Quantity> {
        use MorphospaceConcept::*;
        match individual {
            Healthy => Some(Quantity::from_unit(0.0, &UNITLESS)),
            Inflamed => Some(Quantity::from_unit(1.0, &UNITLESS)),
            Barretts => Some(Quantity::from_unit(2.0, &UNITLESS)),
            Fibrotic => Some(Quantity::from_unit(2.0, &UNITLESS)),
            Dysplastic => Some(Quantity::from_unit(3.0, &UNITLESS)),
            _ => None,
        }
    }
}

/// Quality: does this repair pathway require gap junctions?
#[derive(Debug, Clone)]
pub struct PathwayRequiresGJ;

impl Quality for PathwayRequiresGJ {
    type Individual = MorphospaceConcept;
    type Value = bool;

    fn get(&self, individual: &MorphospaceConcept) -> Option<bool> {
        use MorphospaceConcept::*;
        match individual {
            BasalTurnover => Some(false),
            BioelectricRepair => Some(true),
            MechanicalStimulationPathway => Some(false),
            CombinedTherapy => Some(true),
            _ => None,
        }
    }
}

/// Quality: is this repair pathway hardware-accessible?
#[derive(Debug, Clone)]
pub struct PathwayIsHardwareAccessible;

impl Quality for PathwayIsHardwareAccessible {
    type Individual = MorphospaceConcept;
    type Value = bool;

    fn get(&self, individual: &MorphospaceConcept) -> Option<bool> {
        use MorphospaceConcept::*;
        match individual {
            BasalTurnover => Some(false),
            BioelectricRepair => Some(false),
            MechanicalStimulationPathway => Some(true),
            CombinedTherapy => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_a(child: MorphospaceConcept, parent: MorphospaceConcept) -> bool {
    MorphospaceCategory::morphisms().iter().any(|m| {
        m.kind() == MorphospaceRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn causes(cause: MorphospaceConcept, effect: MorphospaceConcept) -> bool {
    MorphospaceCategory::morphisms().iter().any(|m| {
        m.kind() == MorphospaceRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// All five attractor states have characteristic Vmem ranges (Chernet &
/// Levin 2013).
pub struct AllAttractorsHaveVmemRanges;

impl Axiom for AllAttractorsHaveVmemRanges {
    fn verify(&self) -> Verdict {
        use MorphospaceConcept::*;
        let vmem = AttractorVmemRange;
        let ok = [Healthy, Inflamed, Barretts, Dysplastic, Fibrotic]
            .iter()
            .all(|a| vmem.get(a).is_some());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllAttractorsHaveVmemRanges",
        "every attractor state has a characteristic Vmem range",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    AllAttractorsHaveVmemRanges,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// Healthy is the most polarised attractor (most negative Vmem min).
pub struct HealthyIsMostPolarized;

impl Axiom for HealthyIsMostPolarized {
    fn verify(&self) -> Verdict {
        use MorphospaceConcept::*;
        let vmem = AttractorVmemRange;
        let healthy = match vmem.get(&Healthy) {
            Some(r) => r,
            None => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let ok = [Inflamed, Barretts, Dysplastic, Fibrotic].iter().all(|a| {
            let r = vmem.get(a).unwrap();
            healthy.min < r.min
        });
        // `Quantity::partial_cmp` is dimension-safe: both sides are in
        // MILLIVOLT so `<` above resolves via `PartialOrd`, identical to the
        // former `f64 < f64` comparison on the raw `.min` values.
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HealthyIsMostPolarized",
        "healthy attractor has the most polarised Vmem (lowest min)",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    HealthyIsMostPolarized,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// Severity increases with depolarisation along the main disease axis.
pub struct SeverityIncreasesWithDepolarization;

impl Axiom for SeverityIncreasesWithDepolarization {
    fn verify(&self) -> Verdict {
        use MorphospaceConcept::*;
        let sev = DiseaseSeverity;
        let vmem = AttractorVmemRange;
        let pairs = [
            (Healthy, Inflamed),
            (Inflamed, Barretts),
            (Barretts, Dysplastic),
        ];
        let ok = pairs.iter().all(|(a, b)| {
            let sa = sev.get(a).unwrap();
            let sb = sev.get(b).unwrap();
            let va = vmem.get(a).unwrap();
            let vb = vmem.get(b).unwrap();
            sa < sb && va.max < vb.max
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SeverityIncreasesWithDepolarization",
        "disease severity increases monotonically with Vmem depolarisation along the main axis",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    SeverityIncreasesWithDepolarization,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// Acid damage transitively causes dysplastic transition (Chernet & Levin
/// 2013).
pub struct AcidCausesDysplasia;

impl Axiom for AcidCausesDysplasia {
    fn verify(&self) -> Verdict {
        if causes(
            MorphospaceConcept::AcidDamage,
            MorphospaceConcept::DysplasticTransition,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcidCausesDysplasia",
        "acid damage transitively causes dysplastic transition through chronic inflammation and metaplasia",
        "Chernet & Levin (2013); Gralnek et al. (2006)"
    );
}

pr4xis::register_axiom!(
    AcidCausesDysplasia,
    "Chernet & Levin (2013); Gralnek et al. (2006)"
);

/// Mechanotransduction activation transitively causes autonomous repair.
pub struct MechanotransductionCausesRepair;

impl Axiom for MechanotransductionCausesRepair {
    fn verify(&self) -> Verdict {
        if causes(
            MorphospaceConcept::MechanotransductionActivation,
            MorphospaceConcept::AutonomousRepair,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanotransductionCausesRepair",
        "mechanotransduction activation transitively causes autonomous repair via Vmem repolarisation and pattern recognition",
        "Levin (2014) Mol. Biol. Cell 25(24)"
    );
}

pr4xis::register_axiom!(
    MechanotransductionCausesRepair,
    "Levin (2014) Mol. Biol. Cell 25(24)"
);

/// Acid removal transitively causes autonomous repair (Gralnek et al. 2006).
pub struct AcidRemovalCausesRepair;

impl Axiom for AcidRemovalCausesRepair {
    fn verify(&self) -> Verdict {
        if causes(
            MorphospaceConcept::AcidRemoval,
            MorphospaceConcept::AutonomousRepair,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcidRemovalCausesRepair",
        "acid removal transitively causes autonomous repair via basal-cell replacement",
        "Gralnek et al. (2006) Aliment. Pharmacol. Ther. 23(1):149–157"
    );
}

pr4xis::register_axiom!(
    AcidRemovalCausesRepair,
    "Gralnek et al. (2006) Aliment. Pharmacol. Ther. 23(1):149–157"
);

/// Two-mechanism GJ requirement: bioelectric repair requires GJs,
/// mechanical stimulation does not.
pub struct TwoMechanismGJRequirement;

impl Axiom for TwoMechanismGJRequirement {
    fn verify(&self) -> Verdict {
        use MorphospaceConcept::*;
        let gj = PathwayRequiresGJ;
        if gj.get(&BioelectricRepair) == Some(true)
            && gj.get(&MechanicalStimulationPathway) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TwoMechanismGJRequirement",
        "bioelectric repair requires gap junctions; mechanical stimulation does not",
        "Levin (2014) Mol. Biol. Cell 25(24); Chernet & Levin (2013)"
    );
}

pr4xis::register_axiom!(
    TwoMechanismGJRequirement,
    "Levin (2014) Mol. Biol. Cell 25(24); Chernet & Levin (2013)"
);

/// Only the mechanical-stimulation pathway is hardware-accessible.
pub struct OnlyMechanicalIsHardwareAccessible;

impl Axiom for OnlyMechanicalIsHardwareAccessible {
    fn verify(&self) -> Verdict {
        let hw = PathwayIsHardwareAccessible;
        let pathways: Vec<MorphospaceConcept> = MorphospaceConcept::variants()
            .into_iter()
            .filter(|e| {
                is_a(*e, MorphospaceConcept::RepairPathway)
                    && *e != MorphospaceConcept::RepairPathway
            })
            .collect();
        let hw_accessible: Vec<&MorphospaceConcept> = pathways
            .iter()
            .filter(|e| hw.get(e) == Some(true))
            .collect();
        if hw_accessible.len() == 1
            && *hw_accessible[0] == MorphospaceConcept::MechanicalStimulationPathway
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OnlyMechanicalIsHardwareAccessible",
        "exactly one hardware-accessible repair pathway: mechanical stimulation",
        "Levin (2014) Mol. Biol. Cell 25(24)"
    );
}

pr4xis::register_axiom!(
    OnlyMechanicalIsHardwareAccessible,
    "Levin (2014) Mol. Biol. Cell 25(24)"
);

/// There are exactly 5 attractor states (Fields & Levin 2022 — morphospace
/// for esophageal tissue is partitioned into 5 stable regions).
pub struct FiveAttractorStates;

impl Axiom for FiveAttractorStates {
    fn verify(&self) -> Verdict {
        let attractors: Vec<_> = MorphospaceConcept::variants()
            .into_iter()
            .filter(|e| {
                is_a(*e, MorphospaceConcept::Attractor) && *e != MorphospaceConcept::Attractor
            })
            .collect();
        if attractors.len() == 5 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FiveAttractorStates",
        "exactly five attractor states partition the esophageal morphospace",
        "Fields & Levin (2022) Entropy 24(6):819"
    );
}

pr4xis::register_axiom!(
    FiveAttractorStates,
    "Fields & Levin (2022) Entropy 24(6):819"
);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for MorphospaceOntology {
    type Cat = MorphospaceCategory;
    type Qual = DiseaseSeverity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AllAttractorsHaveVmemRanges));
        axioms.push(Box::new(HealthyIsMostPolarized));
        axioms.push(Box::new(SeverityIncreasesWithDepolarization));
        axioms.push(Box::new(AcidCausesDysplasia));
        axioms.push(Box::new(MechanotransductionCausesRepair));
        axioms.push(Box::new(AcidRemovalCausesRepair));
        axioms.push(Box::new(TwoMechanismGJRequirement));
        axioms.push(Box::new(OnlyMechanicalIsHardwareAccessible));
        axioms.push(Box::new(FiveAttractorStates));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MorphospaceCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MorphospaceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    // -- Domain axiom tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_attractors_have_vmem_ranges() {
        assert!(AllAttractorsHaveVmemRanges.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn healthy_is_most_polarized() {
        assert!(HealthyIsMostPolarized.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn severity_increases_with_depolarization() {
        assert!(SeverityIncreasesWithDepolarization.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn acid_causes_dysplasia() {
        assert!(AcidCausesDysplasia.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanotransduction_causes_repair() {
        assert!(MechanotransductionCausesRepair.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn acid_removal_causes_repair() {
        assert!(AcidRemovalCausesRepair.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_mechanism_gj_requirement() {
        assert!(TwoMechanismGJRequirement.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn only_mechanical_is_hardware_accessible() {
        assert!(OnlyMechanicalIsHardwareAccessible.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_attractor_states() {
        assert!(FiveAttractorStates.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn repair_pathways_classified() {
        for pathway in [
            MorphospaceConcept::BasalTurnover,
            MorphospaceConcept::BioelectricRepair,
            MorphospaceConcept::MechanicalStimulationPathway,
            MorphospaceConcept::CombinedTherapy,
        ] {
            assert!(is_a(pathway, MorphospaceConcept::RepairPathway));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vmem_ranges_match_chernet_levin_2013() {
        assert_eq!(
            AttractorVmemRange
                .get(&MorphospaceConcept::Healthy)
                .unwrap(),
            QuantityRange {
                min: Quantity::from_unit(-70.0, &MILLIVOLT),
                max: Quantity::from_unit(-40.0, &MILLIVOLT),
            }
        );
        assert_eq!(
            AttractorVmemRange
                .get(&MorphospaceConcept::Dysplastic)
                .unwrap(),
            QuantityRange {
                min: Quantity::from_unit(-18.0, &MILLIVOLT),
                max: Quantity::from_unit(0.0, &MILLIVOLT),
            }
        );
    }

    // -- Causal chain tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn acid_damage_chain() {
        for e in [
            MorphospaceConcept::ChronicInflammation,
            MorphospaceConcept::GapJunctionLoss,
            MorphospaceConcept::FibroticRemodeling,
            MorphospaceConcept::MetaplasticTransition,
            MorphospaceConcept::DysplasticTransition,
        ] {
            assert!(
                causes(MorphospaceConcept::AcidDamage, e),
                "AcidDamage should transitively cause {:?}",
                e
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanotransduction_repair_chain() {
        for e in [
            MorphospaceConcept::VmemRepolarization,
            MorphospaceConcept::GapJunctionRestoration,
            MorphospaceConcept::PatternRecognition,
            MorphospaceConcept::AutonomousRepair,
        ] {
            assert!(causes(MorphospaceConcept::MechanotransductionActivation, e));
        }
    }

    // -- Opposition tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn healthy_opposes_dysplastic() {
        let opps: Vec<_> = MorphospaceCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MorphospaceRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(MorphospaceConcept::Healthy, MorphospaceConcept::Dysplastic)));
        assert!(opps.contains(&(MorphospaceConcept::Dysplastic, MorphospaceConcept::Healthy)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn polarized_opposes_depolarized() {
        let opps: Vec<_> = MorphospaceCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MorphospaceRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            MorphospaceConcept::PolarizedVmem,
            MorphospaceConcept::DepolarizedVmem
        )));
    }

    // -- Literature axioms --

    /// Gralnek et al. (2006): PPI-mediated acid removal heals via the
    /// GJ-independent basal-turnover pathway.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_gralnek_2006_basal_turnover_gj_independent() {
        assert_eq!(
            PathwayRequiresGJ.get(&MorphospaceConcept::BasalTurnover),
            Some(false)
        );
    }

    /// Levin (2015): gap-junction blockade alters morphology (proves
    /// bistability in morphospace).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_levin_2015_gj_blockade_bistability() {
        assert!(causes(
            MorphospaceConcept::GapJunctionLoss,
            MorphospaceConcept::FibroticRemodeling
        ));
        let attractors: Vec<_> = MorphospaceConcept::variants()
            .into_iter()
            .filter(|e| {
                is_a(*e, MorphospaceConcept::Attractor) && *e != MorphospaceConcept::Attractor
            })
            .collect();
        assert!(attractors.len() >= 2);
    }

    // -- Proptests --

    fn arb_attractor() -> impl Strategy<Value = MorphospaceConcept> {
        proptest::sample::select(vec![
            MorphospaceConcept::Healthy,
            MorphospaceConcept::Inflamed,
            MorphospaceConcept::Barretts,
            MorphospaceConcept::Dysplastic,
            MorphospaceConcept::Fibrotic,
        ])
    }

    fn arb_main_axis_attractor() -> impl Strategy<Value = MorphospaceConcept> {
        proptest::sample::select(vec![
            MorphospaceConcept::Healthy,
            MorphospaceConcept::Inflamed,
            MorphospaceConcept::Barretts,
            MorphospaceConcept::Dysplastic,
        ])
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MorphospaceCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MorphospaceOntology::axioms() {
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
        fn prop_attractor_vmem_min_lt_max(a in arb_attractor()) {
            let r = AttractorVmemRange.get(&a).unwrap();
            prop_assert!(r.min < r.max);
        }

        #[test]
        fn prop_severity_vmem_monotonicity(
            a in arb_main_axis_attractor(),
            b in arb_main_axis_attractor(),
        ) {
            let sa = DiseaseSeverity.get(&a).unwrap();
            let sb = DiseaseSeverity.get(&b).unwrap();
            if sa < sb {
                let va = AttractorVmemRange.get(&a).unwrap();
                let vb = AttractorVmemRange.get(&b).unwrap();
                prop_assert!(va.max < vb.max);
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = MorphospaceConcept::variants();
            for m in MorphospaceCategory::morphisms() {
                if m.kind() == MorphospaceRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_attractor_vmem_min_lt_max, Verifiable);
    pr4xis::register_praxis_value!(prop_severity_vmem_monotonicity, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
}
