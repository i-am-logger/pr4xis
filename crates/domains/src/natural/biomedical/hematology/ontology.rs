//! Hematology ontology — blood and plasma science.
//!
//! Models blood components (whole blood, plasma, serum, cells, platelets),
//! plasma proteins (albumin, globulin, fibrinogen, immunoglobulin), plasma
//! electrolytes (Na⁺, K⁺, Ca²⁺, Cl⁻, HCO₃⁻), blood properties (osmotic
//! and oncotic pressure, pH, hematocrit, viscosity), and the canonical
//! hematology causal chains (hemorrhage → electrolyte imbalance,
//! inflammation → albumin decrease, acid-base disturbance → buffering,
//! coagulation cascade → fibrin formation).
//!
//! Per `feedback_one_ontology_per_module` the original split between
//! `HematologyEntity` and `HematologyCausalEvent` has been merged into
//! one concept list, with events subsumed by the `HematologyEvent`
//! umbrella.
//!
//! # Literature
//!
//! - **Hoffman et al. (2018)** *Hematology: Basic Principles and Practice*,
//!   7th ed., Elsevier — canonical reference for blood-component
//!   taxonomy (whole blood, plasma, serum, RBC, WBC, platelets), plasma
//!   proteins (albumin, globulin, fibrinogen, immunoglobulin), and
//!   the coagulation cascade.
//! - **Greer et al. (eds.) (2018)** *Wintrobe's Clinical Hematology*,
//!   14th ed., Wolters Kluwer — comprehensive reference for blood
//!   physiology, including erythrocyte and leukocyte biology and the
//!   coagulation cascade fibrin-formation pathway.
//! - **Williams** *Hematology*, 10th ed., McGraw-Hill (Kaushansky et al.
//!   eds.) — companion reference for hematopoiesis and the acute-phase
//!   response (Inflammation → AcutePhaseResponse → AlbuminDecrease).
//! - **Guyton & Hall (2020)** *Textbook of Medical Physiology*, 14th ed.,
//!   Elsevier — canonical source for plasma-electrolyte concentrations
//!   (Na⁺ ≈ 140, K⁺ ≈ 4.5, Ca²⁺ ≈ 2.5, Cl⁻ ≈ 100, HCO₃⁻ ≈ 24 mmol/L)
//!   and the bicarbonate-buffering / pH-regulation system (blood pH
//!   7.35–7.45).

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Hematology",
    source: "Hoffman et al. (2018) Hematology: Basic Principles and Practice 7th ed.; Greer et al. (eds.) (2018) Wintrobe's Clinical Hematology 14th ed.; Kaushansky et al. (eds.) Williams Hematology 10th ed.; Guyton & Hall (2020) Textbook of Medical Physiology 14th ed.",

    concepts: [
        // === Blood components (Hoffman et al. 2018 §1) ===
        WholeBlood,
        BloodPlasma,
        Serum,
        RedBloodCell,
        WhiteBloodCell,
        Platelet,

        // === Plasma proteins (Hoffman et al. 2018 §3) ===
        Albumin,
        Globulin,
        Fibrinogen,
        Immunoglobulin,

        // === Plasma electrolytes (Guyton & Hall 2020 Ch. 25) ===
        SodiumPlasma,
        PotassiumPlasma,
        CalciumPlasma,
        ChloridePlasma,
        BicarbonatePlasma,

        // === Blood properties ===
        OsmoticPressure,
        OncoticPressure,
        BloodPH,
        Hematocrit,
        Viscosity,

        // === Abstract umbrellas ===
        BloodComponent,
        PlasmaProtein,
        PlasmaElectrolyte,
        BloodProperty,
        HematologyEvent,

        // === Causal events (merged from HematologyCausalEvent) ===
        Hemorrhage,
        PlasmaVolumeLoss,
        ElectrolyteImbalance,
        Inflammation,
        AcutePhaseResponse,
        AlbuminDecrease,
        AcidBaseDisturbance,
        BicarbonateBuffering,
        PHCorrection,
        CoagulationCascade,
        FibrinFormation,
    ],

    labels: {
        WholeBlood: ("en", "Whole blood",
            "Hoffman et al. (2018) §1: blood as collected, including plasma and cellular elements."),
        BloodPlasma: ("en", "Blood plasma",
            "Hoffman et al. (2018) §1: the liquid component of blood — water, proteins, electrolytes, dissolved gases."),
        Serum: ("en", "Serum",
            "Hoffman et al. (2018) §1: plasma minus the clotting factors — the supernatant after coagulation."),
        RedBloodCell: ("en", "Red blood cell",
            "Greer et al. (2018): erythrocyte — anucleate cell carrying hemoglobin for O₂/CO₂ transport."),
        WhiteBloodCell: ("en", "White blood cell",
            "Greer et al. (2018): leukocyte — nucleated immune cell."),
        Platelet: ("en", "Platelet",
            "Greer et al. (2018): thrombocyte — small anucleate cell fragment mediating primary hemostasis."),

        Albumin: ("en", "Albumin",
            "Hoffman et al. (2018) §3: the dominant plasma protein; maintains oncotic pressure and transports many ligands."),
        Globulin: ("en", "Globulin",
            "Hoffman et al. (2018) §3: plasma protein class including immunoglobulins and transport globulins."),
        Fibrinogen: ("en", "Fibrinogen",
            "Hoffman et al. (2018) §3: plasma protein converted to fibrin during coagulation."),
        Immunoglobulin: ("en", "Immunoglobulin",
            "Hoffman et al. (2018) §3: antibody — gamma-globulin produced by plasma cells."),

        SodiumPlasma: ("en", "Plasma sodium",
            "Guyton & Hall (2020) Ch. 25: Na⁺; normal ~140 mmol/L; dominant plasma cation."),
        PotassiumPlasma: ("en", "Plasma potassium",
            "Guyton & Hall (2020) Ch. 25: K⁺; normal ~4.5 mmol/L."),
        CalciumPlasma: ("en", "Plasma calcium",
            "Guyton & Hall (2020) Ch. 25: Ca²⁺; normal ~2.5 mmol/L."),
        ChloridePlasma: ("en", "Plasma chloride",
            "Guyton & Hall (2020) Ch. 25: Cl⁻; normal ~100 mmol/L; dominant plasma anion."),
        BicarbonatePlasma: ("en", "Plasma bicarbonate",
            "Guyton & Hall (2020) Ch. 25: HCO₃⁻; normal ~24 mmol/L; primary blood buffer."),

        OsmoticPressure: ("en", "Osmotic pressure",
            "Guyton & Hall (2020): total osmotic pressure of plasma — dominated by electrolytes."),
        OncoticPressure: ("en", "Oncotic pressure",
            "Guyton & Hall (2020): the colloid osmotic pressure contributed by plasma proteins, primarily albumin."),
        BloodPH: ("en", "Blood pH",
            "Guyton & Hall (2020): the −log₁₀ H⁺ activity of blood; tightly regulated to 7.35–7.45."),
        Hematocrit: ("en", "Hematocrit",
            "Greer et al. (2018): the volume fraction of erythrocytes in whole blood."),
        Viscosity: ("en", "Blood viscosity",
            "Greer et al. (2018): the resistance of blood to flow — depends on hematocrit and plasma proteins."),

        BloodComponent: ("en", "Blood component (abstract)",
            "Hoffman et al. (2018): umbrella for the components of whole blood."),
        PlasmaProtein: ("en", "Plasma protein (abstract)",
            "Hoffman et al. (2018) §3: umbrella for soluble protein constituents of plasma."),
        PlasmaElectrolyte: ("en", "Plasma electrolyte (abstract)",
            "Guyton & Hall (2020) Ch. 25: umbrella for ionic solutes in plasma."),
        BloodProperty: ("en", "Blood property (abstract)",
            "Greer et al. (2018): umbrella for measurable properties of blood."),
        HematologyEvent: ("en", "Hematology event (abstract)",
            "Hoffman et al. (2018): umbrella for time-extended processes in hematology (hemorrhage, coagulation, buffering, acute-phase response)."),

        Hemorrhage: ("en", "Hemorrhage",
            "Hoffman et al. (2018): loss of blood from the vascular compartment."),
        PlasmaVolumeLoss: ("en", "Plasma volume loss",
            "Hoffman et al. (2018): reduction in circulating plasma volume."),
        ElectrolyteImbalance: ("en", "Electrolyte imbalance",
            "Guyton & Hall (2020): deviation of plasma electrolyte concentrations from normal."),
        Inflammation: ("en", "Inflammation",
            "Williams Hematology: systemic inflammatory state."),
        AcutePhaseResponse: ("en", "Acute-phase response",
            "Williams Hematology: hepatic response to inflammation — altered synthesis of acute-phase proteins."),
        AlbuminDecrease: ("en", "Albumin decrease",
            "Williams Hematology: decreased albumin synthesis during the acute-phase response."),
        AcidBaseDisturbance: ("en", "Acid-base disturbance",
            "Guyton & Hall (2020): perturbation of normal blood pH."),
        BicarbonateBuffering: ("en", "Bicarbonate buffering",
            "Guyton & Hall (2020) Ch. 30: HCO₃⁻/H₂CO₃ buffering — the primary plasma buffer system."),
        PHCorrection: ("en", "pH correction",
            "Guyton & Hall (2020): restoration of blood pH to the 7.35–7.45 range."),
        CoagulationCascade: ("en", "Coagulation cascade",
            "Hoffman et al. (2018) §10: the canonical clotting-factor cascade."),
        FibrinFormation: ("en", "Fibrin formation",
            "Hoffman et al. (2018) §10: thrombin-mediated conversion of fibrinogen to fibrin polymer."),
    },

    is_a: [
        // Blood components.
        (WholeBlood, BloodComponent),
        (BloodPlasma, BloodComponent),
        (Serum, BloodComponent),
        (RedBloodCell, BloodComponent),
        (WhiteBloodCell, BloodComponent),
        (Platelet, BloodComponent),
        // Plasma proteins.
        (Albumin, PlasmaProtein),
        (Globulin, PlasmaProtein),
        (Fibrinogen, PlasmaProtein),
        (Immunoglobulin, PlasmaProtein),
        // Plasma electrolytes.
        (SodiumPlasma, PlasmaElectrolyte),
        (PotassiumPlasma, PlasmaElectrolyte),
        (CalciumPlasma, PlasmaElectrolyte),
        (ChloridePlasma, PlasmaElectrolyte),
        (BicarbonatePlasma, PlasmaElectrolyte),
        // Blood properties.
        (OsmoticPressure, BloodProperty),
        (OncoticPressure, BloodProperty),
        (BloodPH, BloodProperty),
        (Hematocrit, BloodProperty),
        (Viscosity, BloodProperty),
        // Events.
        (Hemorrhage, HematologyEvent),
        (PlasmaVolumeLoss, HematologyEvent),
        (ElectrolyteImbalance, HematologyEvent),
        (Inflammation, HematologyEvent),
        (AcutePhaseResponse, HematologyEvent),
        (AlbuminDecrease, HematologyEvent),
        (AcidBaseDisturbance, HematologyEvent),
        (BicarbonateBuffering, HematologyEvent),
        (PHCorrection, HematologyEvent),
        (CoagulationCascade, HematologyEvent),
        (FibrinFormation, HematologyEvent),
    ],

    has_a: [
        // Whole blood = plasma + cellular elements (Hoffman et al. 2018 §1).
        (WholeBlood, BloodPlasma),
        (WholeBlood, RedBloodCell),
        (WholeBlood, WhiteBloodCell),
        (WholeBlood, Platelet),
        // Plasma contains proteins (Hoffman et al. 2018 §3) and
        // electrolytes (Guyton & Hall 2020 Ch. 25).
        (BloodPlasma, Albumin),
        (BloodPlasma, Globulin),
        (BloodPlasma, Fibrinogen),
        (BloodPlasma, Immunoglobulin),
        (BloodPlasma, SodiumPlasma),
        (BloodPlasma, PotassiumPlasma),
        (BloodPlasma, CalciumPlasma),
        (BloodPlasma, ChloridePlasma),
        (BloodPlasma, BicarbonatePlasma),
    ],

    causes: [
        // Hemorrhage chain (Hoffman et al. 2018).
        (Hemorrhage, PlasmaVolumeLoss),
        (PlasmaVolumeLoss, ElectrolyteImbalance),
        // Acute-phase response (Williams Hematology).
        (Inflammation, AcutePhaseResponse),
        (AcutePhaseResponse, AlbuminDecrease),
        // Buffering / pH regulation (Guyton & Hall 2020 Ch. 30).
        (AcidBaseDisturbance, BicarbonateBuffering),
        (BicarbonateBuffering, PHCorrection),
        // Coagulation cascade (Hoffman et al. 2018 §10).
        (CoagulationCascade, FibrinFormation),
    ],

    opposes: [
        // Transport vs immune function: albumin (oncotic / transport) vs
        // globulin (which includes immunoglobulins).
        (Albumin, Globulin),
        (Globulin, Albumin),
        // Oxygen transport vs immune defense.
        (RedBloodCell, WhiteBloodCell),
        (WhiteBloodCell, RedBloodCell),
    ],
}

// Backward-compatibility re-exports.
pub use HematologyConcept as HematologyEntity;
pub use HematologyRelationKind as HematologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Normal plasma concentration in mmol/L for electrolytes (Guyton & Hall
/// 2020 Ch. 25).
#[derive(Debug, Clone)]
pub struct NormalConcentration;

impl Quality for NormalConcentration {
    type Individual = HematologyConcept;
    type Value = f64;

    fn get(&self, individual: &HematologyConcept) -> Option<f64> {
        use HematologyConcept::*;
        match individual {
            SodiumPlasma => Some(140.0),
            PotassiumPlasma => Some(4.5),
            CalciumPlasma => Some(2.5),
            ChloridePlasma => Some(100.0),
            BicarbonatePlasma => Some(24.0),
            _ => None,
        }
    }
}

/// Quality: is this entity a clotting factor (Hoffman et al. 2018 §10)?
#[derive(Debug, Clone)]
pub struct IsClottingFactor;

impl Quality for IsClottingFactor {
    type Individual = HematologyConcept;
    type Value = bool;

    fn get(&self, individual: &HematologyConcept) -> Option<bool> {
        use HematologyConcept::*;
        Some(matches!(individual, Fibrinogen | Platelet))
    }
}

/// Quality: does this entity contribute to plasma osmolarity (Guyton &
/// Hall 2020 Ch. 25)?
#[derive(Debug, Clone)]
pub struct AffectsOsmolarity;

impl Quality for AffectsOsmolarity {
    type Individual = HematologyConcept;
    type Value = bool;

    fn get(&self, individual: &HematologyConcept) -> Option<bool> {
        use HematologyConcept::*;
        Some(matches!(
            individual,
            SodiumPlasma
                | PotassiumPlasma
                | CalciumPlasma
                | ChloridePlasma
                | BicarbonatePlasma
                | Albumin
        ))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parts_of(whole: HematologyConcept) -> Vec<HematologyConcept> {
    HematologyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == HematologyRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn causes(cause: HematologyConcept, effect: HematologyConcept) -> bool {
    HematologyCategory::morphisms().iter().any(|m| {
        m.kind() == HematologyRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Whole blood contains blood plasma (Hoffman et al. 2018 §1).
pub struct WholeBloodContainsPlasma;

impl Axiom for WholeBloodContainsPlasma {
    fn verify(&self) -> Verdict {
        if parts_of(HematologyConcept::WholeBlood).contains(&HematologyConcept::BloodPlasma) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WholeBloodContainsPlasma",
        "whole blood contains blood plasma",
        "Hoffman et al. (2018) Hematology: Basic Principles and Practice 7th ed. §1"
    );
}

pr4xis::register_axiom!(
    WholeBloodContainsPlasma,
    "Hoffman et al. (2018) Hematology 7th ed. §1"
);

/// Blood plasma contains all five plasma electrolytes (Guyton & Hall 2020
/// Ch. 25).
pub struct PlasmaContainsAllElectrolytes;

impl Axiom for PlasmaContainsAllElectrolytes {
    fn verify(&self) -> Verdict {
        use HematologyConcept::*;
        let parts = parts_of(BloodPlasma);
        let ok = parts.contains(&SodiumPlasma)
            && parts.contains(&PotassiumPlasma)
            && parts.contains(&CalciumPlasma)
            && parts.contains(&ChloridePlasma)
            && parts.contains(&BicarbonatePlasma);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PlasmaContainsAllElectrolytes",
        "blood plasma contains all five canonical plasma electrolytes (Na⁺, K⁺, Ca²⁺, Cl⁻, HCO₃⁻)",
        "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 25"
    );
}

pr4xis::register_axiom!(
    PlasmaContainsAllElectrolytes,
    "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 25"
);

/// Sodium is the dominant plasma cation: [Na⁺] ≫ [K⁺] by more than an
/// order of magnitude (Guyton & Hall 2020 Ch. 25: 140 mmol/L vs 4.5 mmol/L).
pub struct SodiumIsDominantCation;

impl Axiom for SodiumIsDominantCation {
    fn verify(&self) -> Verdict {
        let na = NormalConcentration.get(&HematologyConcept::SodiumPlasma);
        let k = NormalConcentration.get(&HematologyConcept::PotassiumPlasma);
        let ok = match (na, k) {
            (Some(na), Some(k)) if k > 0.0 => na > k * 10.0,
            _ => false,
        };
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SodiumIsDominantCation",
        "sodium is the dominant plasma cation — [Na⁺]/[K⁺] > 10 (140 mmol/L vs 4.5 mmol/L)",
        "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 25"
    );
}

pr4xis::register_axiom!(
    SodiumIsDominantCation,
    "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 25"
);

/// Blood pH is tightly regulated by bicarbonate buffering: acid-base
/// disturbance transitively causes pH correction (Guyton & Hall 2020
/// Ch. 30).
pub struct BloodPHRegulated;

impl Axiom for BloodPHRegulated {
    fn verify(&self) -> Verdict {
        if causes(
            HematologyConcept::AcidBaseDisturbance,
            HematologyConcept::PHCorrection,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BloodPHRegulated",
        "acid-base disturbance is corrected via bicarbonate buffering (blood pH regulated to 7.35–7.45)",
        "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 30"
    );
}

pr4xis::register_axiom!(
    BloodPHRegulated,
    "Guyton & Hall (2020) Textbook of Medical Physiology 14th ed. Ch. 30"
);

/// Hemorrhage transitively causes electrolyte imbalance (Hoffman et al.
/// 2018 — hemorrhage → plasma-volume loss → electrolyte imbalance).
pub struct HemorrhageCausesElectrolyteImbalance;

impl Axiom for HemorrhageCausesElectrolyteImbalance {
    fn verify(&self) -> Verdict {
        if causes(
            HematologyConcept::Hemorrhage,
            HematologyConcept::ElectrolyteImbalance,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HemorrhageCausesElectrolyteImbalance",
        "hemorrhage transitively causes electrolyte imbalance via plasma-volume loss",
        "Hoffman et al. (2018) Hematology 7th ed."
    );
}

pr4xis::register_axiom!(
    HemorrhageCausesElectrolyteImbalance,
    "Hoffman et al. (2018) Hematology 7th ed."
);

/// Inflammation transitively causes albumin decrease (Williams Hematology
/// — acute-phase response down-regulates hepatic albumin synthesis).
pub struct InflammationCausesAlbuminDecrease;

impl Axiom for InflammationCausesAlbuminDecrease {
    fn verify(&self) -> Verdict {
        if causes(
            HematologyConcept::Inflammation,
            HematologyConcept::AlbuminDecrease,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InflammationCausesAlbuminDecrease",
        "inflammation transitively causes albumin decrease via the acute-phase response",
        "Kaushansky et al. (eds.) Williams Hematology 10th ed."
    );
}

pr4xis::register_axiom!(
    InflammationCausesAlbuminDecrease,
    "Kaushansky et al. (eds.) Williams Hematology 10th ed."
);

/// Coagulation cascade transitively causes fibrin formation (Hoffman et
/// al. 2018 §10).
pub struct CoagulationProducesFibrin;

impl Axiom for CoagulationProducesFibrin {
    fn verify(&self) -> Verdict {
        if causes(
            HematologyConcept::CoagulationCascade,
            HematologyConcept::FibrinFormation,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CoagulationProducesFibrin",
        "the coagulation cascade transitively causes fibrin formation",
        "Hoffman et al. (2018) Hematology 7th ed. §10"
    );
}

pr4xis::register_axiom!(
    CoagulationProducesFibrin,
    "Hoffman et al. (2018) Hematology 7th ed. §10"
);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for HematologyOntology {
    type Cat = HematologyCategory;
    type Qual = NormalConcentration;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(WholeBloodContainsPlasma));
        axioms.push(Box::new(PlasmaContainsAllElectrolytes));
        axioms.push(Box::new(SodiumIsDominantCation));
        axioms.push(Box::new(BloodPHRegulated));
        axioms.push(Box::new(HemorrhageCausesElectrolyteImbalance));
        axioms.push(Box::new(InflammationCausesAlbuminDecrease));
        axioms.push(Box::new(CoagulationProducesFibrin));
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

    /// Local subsumption query: is there an `is_a` edge from `child` to `parent`?
    /// (Per `feedback_no_helpers_in_core`, this lives in the test module, not core.)
    fn is_a(child: HematologyConcept, parent: HematologyConcept) -> bool {
        HematologyCategory::morphisms().iter().any(|m| {
            m.kind() == HematologyRelationKind::Subsumption
                && m.source() == child
                && m.target() == parent
        })
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<HematologyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        HematologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    // -- Domain axiom tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn whole_blood_contains_plasma() {
        assert!(WholeBloodContainsPlasma.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn plasma_contains_all_electrolytes() {
        assert!(PlasmaContainsAllElectrolytes.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sodium_is_dominant_cation() {
        assert!(SodiumIsDominantCation.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn blood_ph_regulated() {
        assert!(BloodPHRegulated.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hemorrhage_causes_electrolyte_imbalance() {
        assert!(HemorrhageCausesElectrolyteImbalance.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn inflammation_causes_albumin_decrease() {
        assert!(InflammationCausesAlbuminDecrease.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn coagulation_produces_fibrin() {
        assert!(CoagulationProducesFibrin.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn blood_plasma_is_a_blood_component() {
        assert!(is_a(
            HematologyConcept::BloodPlasma,
            HematologyConcept::BloodComponent
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn albumin_is_a_plasma_protein() {
        assert!(is_a(
            HematologyConcept::Albumin,
            HematologyConcept::PlasmaProtein
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sodium_is_a_plasma_electrolyte() {
        assert!(is_a(
            HematologyConcept::SodiumPlasma,
            HematologyConcept::PlasmaElectrolyte
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hematocrit_is_a_blood_property() {
        assert!(is_a(
            HematologyConcept::Hematocrit,
            HematologyConcept::BloodProperty
        ));
    }

    // -- Mereology / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn whole_blood_contains_rbc() {
        assert!(parts_of(HematologyConcept::WholeBlood).contains(&HematologyConcept::RedBloodCell));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn whole_blood_transitively_contains_sodium() {
        assert!(parts_of(HematologyConcept::WholeBlood).contains(&HematologyConcept::SodiumPlasma));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn whole_blood_transitively_contains_albumin() {
        assert!(parts_of(HematologyConcept::WholeBlood).contains(&HematologyConcept::Albumin));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn plasma_contains_fibrinogen() {
        assert!(parts_of(HematologyConcept::BloodPlasma).contains(&HematologyConcept::Fibrinogen));
    }

    // -- Opposition tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn albumin_opposes_globulin() {
        let opps: Vec<_> = HematologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == HematologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(HematologyConcept::Albumin, HematologyConcept::Globulin)));
        assert!(opps.contains(&(HematologyConcept::Globulin, HematologyConcept::Albumin)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rbc_opposes_wbc() {
        let opps: Vec<_> = HematologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == HematologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            HematologyConcept::RedBloodCell,
            HematologyConcept::WhiteBloodCell
        )));
    }

    // -- Quality tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn electrolyte_concentrations_match_guyton_hall() {
        assert_eq!(
            NormalConcentration.get(&HematologyConcept::SodiumPlasma),
            Some(140.0)
        );
        assert_eq!(
            NormalConcentration.get(&HematologyConcept::PotassiumPlasma),
            Some(4.5)
        );
        assert_eq!(
            NormalConcentration.get(&HematologyConcept::CalciumPlasma),
            Some(2.5)
        );
        assert_eq!(
            NormalConcentration.get(&HematologyConcept::ChloridePlasma),
            Some(100.0)
        );
        assert_eq!(
            NormalConcentration.get(&HematologyConcept::BicarbonatePlasma),
            Some(24.0)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fibrinogen_is_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyConcept::Fibrinogen),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn platelet_is_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyConcept::Platelet),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn albumin_not_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyConcept::Albumin),
            Some(false)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sodium_affects_osmolarity() {
        assert_eq!(
            AffectsOsmolarity.get(&HematologyConcept::SodiumPlasma),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn albumin_affects_osmolarity() {
        // Oncotic pressure (Guyton & Hall 2020).
        assert_eq!(
            AffectsOsmolarity.get(&HematologyConcept::Albumin),
            Some(true)
        );
    }

    // -- Causal chain tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hemorrhage_transitively_causes_electrolyte_imbalance() {
        assert!(causes(
            HematologyConcept::Hemorrhage,
            HematologyConcept::ElectrolyteImbalance
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn inflammation_transitively_causes_albumin_decrease() {
        assert!(causes(
            HematologyConcept::Inflammation,
            HematologyConcept::AlbuminDecrease
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn acid_base_disturbance_causes_ph_correction() {
        assert!(causes(
            HematologyConcept::AcidBaseDisturbance,
            HematologyConcept::PHCorrection
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn coagulation_causes_fibrin_formation() {
        assert!(causes(
            HematologyConcept::CoagulationCascade,
            HematologyConcept::FibrinFormation
        ));
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = HematologyConcept> {
        proptest::sample::select(HematologyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in HematologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in HematologyOntology::axioms() {
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
        fn prop_clotting_factor_total(c in arb_concept()) {
            prop_assert!(IsClottingFactor.get(&c).is_some());
        }

        #[test]
        fn prop_affects_osmolarity_total(c in arb_concept()) {
            prop_assert!(AffectsOsmolarity.get(&c).is_some());
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = HematologyConcept::variants();
            for m in HematologyCategory::morphisms() {
                if m.kind() == HematologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = HematologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == HematologyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_clotting_factor_total, Verifiable);
    pr4xis::register_praxis_value!(prop_affects_osmolarity_total, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}
