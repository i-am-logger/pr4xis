//! Biophysics — mechanical properties, wave physics, piezoelectricity, and
//! biological media for biomedical modelling.
//!
//! Models mechanical properties (viscoelasticity, elasticity, stress, strain),
//! wave physics (mechanical waves, acoustic impedance, frequency, wavelength),
//! piezoelectric effects (direct/converse, collagen piezoelectricity),
//! membrane biophysics, and biological media (bone matrix, soft tissue, fluid).
//! Causal events cover external vibration → wave propagation → tissue
//! deformation → mechanotransduction, plus bone-conducted transmission and the
//! piezoelectric-charge / impedance-mismatch branches. Per
//! `feedback_one_ontology_per_module` the original split between
//! `BiophysicsEntity` and `BiophysicsCausalEvent` has been merged: events are
//! first-class concepts subsumed by the `BiophysicalEvent` umbrella.
//!
//! # Literature
//!
//! - **Fukada & Yasuda (1957)** "On the Piezoelectric Effect of Bone",
//!   *Journal of the Physical Society of Japan* 12:1158-1162 — foundational
//!   discovery that bone (collagen content) is piezoelectric: mechanical
//!   stress on bone generates electric charge.
//! - **Duck (1990)** *Physical Properties of Tissue: A Comprehensive
//!   Reference Book*, Academic Press — canonical compendium of acoustic
//!   impedance, attenuation, and wave-physics parameters for biological
//!   tissue (bone, soft tissue, fluid).
//! - **Cowin & Doty (2007)** *Tissue Mechanics*, Springer — canonical
//!   reference on viscoelasticity, stress-strain relations, and the
//!   mechanics of biological media.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Biophysics",
    source: "Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162; Duck (1990) Physical Properties of Tissue; Cowin & Doty (2007) Tissue Mechanics.",

    concepts: [
        // === Mechanical properties (Cowin & Doty 2007) ===
        Viscoelasticity,
        Elasticity,
        Viscosity,
        StiffnessModulus,
        StrainRate,
        MechanicalStress,
        MechanicalStrain,

        // === Wave physics (Duck 1990) ===
        MechanicalWave,
        AcousticImpedance,
        Attenuation,
        Frequency,
        Wavelength,
        ResonanceFrequency,

        // === Piezoelectricity (Fukada & Yasuda 1957) ===
        PiezoelectricEffect,
        DirectPiezoelectric,
        ConversePiezoelectric,
        CollagenPiezoelectricity,

        // === Membrane biophysics (Cowin & Doty 2007 ch. on cells) ===
        MembraneCapacitance,
        MembraneTension,
        CellDeformation,

        // === Biological media (Duck 1990) ===
        BoneMatrix,
        SoftTissue,
        FluidMedium,
        CellMembrane,

        // === Abstract umbrellas ===
        MechanicalProperty,
        WaveProperty,
        PiezoelectricProperty,
        BiologicalMedium,
        BiophysicalEvent,

        // === Causal events (Fukada & Yasuda 1957; Duck 1990) ===
        ExternalVibration,
        WavePropagation,
        TissueDeformation,
        CellMembraneStrain,
        MechanotransducerActivation,
        BoneConductionTransmission,
        PiezoelectricChargeGeneration,
        LocalElectricField,
        ImpedanceMismatch,
        WaveReflection,
    ],

    labels: {
        Viscoelasticity: ("en", "Viscoelasticity",
            "Cowin & Doty (2007) §3: combined viscous and elastic response of biological tissue under deformation."),
        Elasticity: ("en", "Elasticity",
            "Cowin & Doty (2007) §2: tendency of a material to recover its original shape after the removal of a deforming stress."),
        Viscosity: ("en", "Viscosity",
            "Cowin & Doty (2007) §3: resistance of a fluid to gradual deformation by shear or tensile stress."),
        StiffnessModulus: ("en", "Stiffness modulus",
            "Cowin & Doty (2007) §2: ratio of stress to strain in the elastic regime (Young's modulus and analogues)."),
        StrainRate: ("en", "Strain rate",
            "Cowin & Doty (2007) §3: time derivative of strain; central to viscoelastic constitutive equations."),
        MechanicalStress: ("en", "Mechanical stress",
            "Cowin & Doty (2007) §2: force per unit area applied to a deformable material."),
        MechanicalStrain: ("en", "Mechanical strain",
            "Cowin & Doty (2007) §2: dimensionless measure of deformation per unit length."),

        MechanicalWave: ("en", "Mechanical wave",
            "Duck (1990) §4: oscillatory disturbance propagating through a medium by particle displacement."),
        AcousticImpedance: ("en", "Acoustic impedance",
            "Duck (1990) §4: product of density and sound speed (Z = ρc), governing wave transmission/reflection at boundaries."),
        Attenuation: ("en", "Attenuation",
            "Duck (1990) §5: exponential decrease in wave amplitude with propagation distance through an absorbing medium."),
        Frequency: ("en", "Frequency",
            "Duck (1990) §4: number of oscillation cycles per unit time of a periodic wave (Hz)."),
        Wavelength: ("en", "Wavelength",
            "Duck (1990) §4: spatial period of a wave; λ = c/f."),
        ResonanceFrequency: ("en", "Resonance frequency",
            "Duck (1990) §6: frequency at which a system absorbs energy maximally; for biological structures, often 20-120 Hz."),

        PiezoelectricEffect: ("en", "Piezoelectric effect",
            "Fukada & Yasuda (1957): generation of electric charge by mechanical stress (direct) and mechanical strain by applied electric field (converse) in non-centrosymmetric materials."),
        DirectPiezoelectric: ("en", "Direct piezoelectric effect",
            "Fukada & Yasuda (1957): mechanical stress → electric polarization in bone/collagen."),
        ConversePiezoelectric: ("en", "Converse piezoelectric effect",
            "Fukada & Yasuda (1957): applied electric field → mechanical strain (inverse of the direct effect)."),
        CollagenPiezoelectricity: ("en", "Collagen piezoelectricity",
            "Fukada & Yasuda (1957): the piezoelectric response of bone is attributable to collagen's non-centrosymmetric crystalline structure."),

        MembraneCapacitance: ("en", "Membrane capacitance",
            "Cowin & Doty (2007): charge-storage property of the lipid bilayer, ~1 µF/cm²; sets the time constant of voltage changes."),
        MembraneTension: ("en", "Membrane tension",
            "Cowin & Doty (2007): in-plane mechanical tension in the lipid bilayer; gates mechanosensitive channels."),
        CellDeformation: ("en", "Cell deformation",
            "Cowin & Doty (2007): geometric change in cell shape under applied mechanical stress."),

        BoneMatrix: ("en", "Bone matrix",
            "Duck (1990) Table 4.1: mineralized collagen extracellular matrix; acoustic impedance ≈ 7.4 MRayl."),
        SoftTissue: ("en", "Soft tissue",
            "Duck (1990) Table 4.1: hydrated cellular tissue; acoustic impedance ≈ 1.6 MRayl."),
        FluidMedium: ("en", "Fluid medium",
            "Duck (1990) Table 4.1: aqueous biological fluid (e.g. interstitial); acoustic impedance ≈ 1.5 MRayl."),
        CellMembrane: ("en", "Cell membrane",
            "Cowin & Doty (2007): lipid-bilayer boundary of a cell; acoustic impedance similar to soft tissue."),

        MechanicalProperty: ("en", "Mechanical property",
            "Cowin & Doty (2007): umbrella for stress, strain, stiffness, viscoelasticity, and related measurable quantities."),
        WaveProperty: ("en", "Wave property",
            "Duck (1990): umbrella for frequency, wavelength, impedance, attenuation, resonance."),
        PiezoelectricProperty: ("en", "Piezoelectric property",
            "Fukada & Yasuda (1957): umbrella for direct/converse effects and material-specific piezoelectric coefficients."),
        BiologicalMedium: ("en", "Biological medium",
            "Duck (1990): umbrella for bone, soft tissue, fluid, and membrane phases through which mechanical waves propagate."),
        BiophysicalEvent: ("en", "Biophysical event",
            "Cowin & Doty (2007); Duck (1990): umbrella for time-extended biophysical processes (vibration, propagation, deformation, charge generation)."),

        ExternalVibration: ("en", "External vibration",
            "Duck (1990) §4: an externally-applied periodic mechanical disturbance to a biological system."),
        WavePropagation: ("en", "Wave propagation",
            "Duck (1990) §4: transmission of a mechanical disturbance through a medium at the local sound speed."),
        TissueDeformation: ("en", "Tissue deformation",
            "Cowin & Doty (2007) §2: geometric change in tissue shape resulting from applied stress."),
        CellMembraneStrain: ("en", "Cell-membrane strain",
            "Cowin & Doty (2007): local strain at the lipid bilayer, transmitted from tissue-level deformation."),
        MechanotransducerActivation: ("en", "Mechanotransducer activation",
            "Cowin & Doty (2007); Fukada & Yasuda (1957): gating of a mechanosensitive transducer (channel or piezo material) by mechanical input."),
        BoneConductionTransmission: ("en", "Bone-conduction transmission",
            "Duck (1990) §6: propagation of vibration through bone owing to its high stiffness and low attenuation."),
        PiezoelectricChargeGeneration: ("en", "Piezoelectric charge generation",
            "Fukada & Yasuda (1957): appearance of bound electric charge at a piezoelectric surface following mechanical deformation."),
        LocalElectricField: ("en", "Local electric field",
            "Fukada & Yasuda (1957): the electric field produced by separated piezoelectric charges; biases nearby ions and membranes."),
        ImpedanceMismatch: ("en", "Impedance mismatch",
            "Duck (1990) §4: difference in acoustic impedance across a tissue boundary; magnitude controls reflection/transmission."),
        WaveReflection: ("en", "Wave reflection",
            "Duck (1990) §4: return of part of an incident wave at a boundary with non-zero impedance mismatch."),
    },

    is_a: [
        // Mechanical properties
        (Viscoelasticity, MechanicalProperty),
        (Elasticity, MechanicalProperty),
        (Viscosity, MechanicalProperty),
        (StiffnessModulus, MechanicalProperty),
        (StrainRate, MechanicalProperty),
        (MechanicalStress, MechanicalProperty),
        (MechanicalStrain, MechanicalProperty),
        (MembraneCapacitance, MechanicalProperty),
        (MembraneTension, MechanicalProperty),
        (CellDeformation, MechanicalProperty),

        // Wave properties
        (MechanicalWave, WaveProperty),
        (AcousticImpedance, WaveProperty),
        (Attenuation, WaveProperty),
        (Frequency, WaveProperty),
        (Wavelength, WaveProperty),
        (ResonanceFrequency, WaveProperty),

        // Piezoelectric properties
        (PiezoelectricEffect, PiezoelectricProperty),
        (DirectPiezoelectric, PiezoelectricProperty),
        (ConversePiezoelectric, PiezoelectricProperty),
        (CollagenPiezoelectricity, PiezoelectricProperty),
        (DirectPiezoelectric, PiezoelectricEffect),
        (ConversePiezoelectric, PiezoelectricEffect),

        // Biological media
        (BoneMatrix, BiologicalMedium),
        (SoftTissue, BiologicalMedium),
        (FluidMedium, BiologicalMedium),
        (CellMembrane, BiologicalMedium),

        // Events under the BiophysicalEvent umbrella
        (ExternalVibration, BiophysicalEvent),
        (WavePropagation, BiophysicalEvent),
        (TissueDeformation, BiophysicalEvent),
        (CellMembraneStrain, BiophysicalEvent),
        (MechanotransducerActivation, BiophysicalEvent),
        (BoneConductionTransmission, BiophysicalEvent),
        (PiezoelectricChargeGeneration, BiophysicalEvent),
        (LocalElectricField, BiophysicalEvent),
        (ImpedanceMismatch, BiophysicalEvent),
        (WaveReflection, BiophysicalEvent),
    ],

    causes: [
        // Duck (1990) §4 — vibration → wave → deformation → strain → mechanotransduction
        (ExternalVibration, WavePropagation),
        (WavePropagation, TissueDeformation),
        (TissueDeformation, CellMembraneStrain),
        (CellMembraneStrain, MechanotransducerActivation),
        // Duck (1990) §6 — bone-conduction branch
        (ExternalVibration, BoneConductionTransmission),
        (BoneConductionTransmission, WavePropagation),
        // Fukada & Yasuda (1957) — direct piezoelectric effect: deformation → charge → field
        (TissueDeformation, PiezoelectricChargeGeneration),
        (PiezoelectricChargeGeneration, LocalElectricField),
        // Duck (1990) §4 — boundary reflection
        (ImpedanceMismatch, WaveReflection),
    ],

    opposes: [
        // DirectPiezoelectric ↔ ConversePiezoelectric: stress→field vs field→strain
        (DirectPiezoelectric, ConversePiezoelectric),
        (ConversePiezoelectric, DirectPiezoelectric),
        // Elasticity ↔ Viscosity: instantaneous recoverable vs time-dependent dissipative
        // (the two limiting responses on the viscoelastic continuum; Cowin & Doty 2007 §3)
        (Elasticity, Viscosity),
        (Viscosity, Elasticity),
        // MechanicalStress ↔ MechanicalStrain: cause (force/area) vs effect (deformation)
        // — paired sides of the constitutive equation (Cowin & Doty 2007 §2)
        (MechanicalStress, MechanicalStrain),
        (MechanicalStrain, MechanicalStress),
    ],
}

// ---------------------------------------------------------------------------
// Backward-compatibility aliases (transitional — pre-1.0)
// ---------------------------------------------------------------------------
//
// Several consumer modules in this crate (and the formal/meta diagnostics
// tree) were written against the old hand-rolled `BiophysicsEntity` /
// `BiophysicsCategoryRelationKind` names. Until those modules migrate to the
// proc-macro-generated names, these aliases keep them compiling. They will be
// removed in a follow-up batch once every consumer has been updated.

/// Transitional alias for the proc-macro-generated `BiophysicsConcept`.
pub type BiophysicsEntity = BiophysicsConcept;
/// Transitional alias for the proc-macro-generated `BiophysicsRelationKind`.
pub type BiophysicsCategoryRelationKind = BiophysicsRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Acoustic impedance Z in MRayl (= 10⁶ kg·m⁻²·s⁻¹) for biological media.
///
/// Values from Duck (1990) Table 4.1.
#[derive(Debug, Clone)]
pub struct AcousticImpedanceValue;

impl Quality for AcousticImpedanceValue {
    type Individual = BiophysicsConcept;
    type Value = f64;

    fn get(&self, c: &BiophysicsConcept) -> Option<f64> {
        use BiophysicsConcept::*;
        match c {
            BoneMatrix => Some(7.4),
            SoftTissue => Some(1.6),
            FluidMedium => Some(1.5),
            CellMembrane => Some(1.6),
            _ => None,
        }
    }
}

/// Whether a concept exhibits piezoelectricity.
///
/// Fukada & Yasuda (1957): only collagen-containing solids in this taxonomy
/// produce piezoelectric charge under deformation.
#[derive(Debug, Clone)]
pub struct IsPiezoelectric;

impl Quality for IsPiezoelectric {
    type Individual = BiophysicsConcept;
    type Value = bool;

    fn get(&self, c: &BiophysicsConcept) -> Option<bool> {
        use BiophysicsConcept::*;
        match c {
            CollagenPiezoelectricity => Some(true),
            BoneMatrix => Some(true),
            CellMembrane => Some(false),
            SoftTissue => Some(false),
            FluidMedium => Some(false),
            _ => None,
        }
    }
}

/// Whether a biological medium transmits mechanical vibration.
#[derive(Debug, Clone)]
pub struct TransmitsVibration;

impl Quality for TransmitsVibration {
    type Individual = BiophysicsConcept;
    type Value = bool;

    fn get(&self, c: &BiophysicsConcept) -> Option<bool> {
        use BiophysicsConcept::*;
        match c {
            BoneMatrix | SoftTissue | FluidMedium | CellMembrane => Some(true),
            _ => None,
        }
    }
}

/// Frequency range (min_hz, max_hz) characteristic of wave concepts.
#[derive(Debug, Clone)]
pub struct FrequencyRange;

impl Quality for FrequencyRange {
    type Individual = BiophysicsConcept;
    type Value = (f64, f64);

    fn get(&self, c: &BiophysicsConcept) -> Option<(f64, f64)> {
        use BiophysicsConcept::*;
        match c {
            MechanicalWave => Some((1.0, 200.0)),
            ResonanceFrequency => Some((20.0, 120.0)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for BiophysicsOntology {
    type Cat = BiophysicsCategory;
    type Qual = AcousticImpedanceValue;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(VibrationCausesMechanotransduction));
        axioms.push(Box::new(PiezoelectricFollowsDeformation));
        axioms.push(Box::new(BoneMatrixIsPiezoelectric));
        axioms.push(Box::new(BoneImpedanceGreaterThanSoftTissue));
        axioms.push(Box::new(ImpedanceMismatchCausesReflection));
        axioms
    }
}

/// Helper: does a `Causation` edge transitively connect `cause` to `effect`?
///
/// Walks `Causation`-kinded morphisms in the category to compute transitive
/// closure.
fn causes_transitively(cause: BiophysicsConcept, effect: BiophysicsConcept) -> bool {
    let direct: Vec<(BiophysicsConcept, BiophysicsConcept)> = BiophysicsCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == BiophysicsRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    // BFS from `cause`
    let mut visited: Vec<BiophysicsConcept> = vec![cause];
    let mut frontier = vec![cause];
    while let Some(c) = frontier.pop() {
        for (s, t) in &direct {
            if *s == c && !visited.contains(t) {
                if *t == effect {
                    return true;
                }
                visited.push(*t);
                frontier.push(*t);
            }
        }
    }
    false
}

/// Axiom: external vibration transitively causes mechanotransducer activation.
///
/// Duck (1990) §4: the chain ExternalVibration → WavePropagation →
/// TissueDeformation → CellMembraneStrain → MechanotransducerActivation.
pub struct VibrationCausesMechanotransduction;

impl Axiom for VibrationCausesMechanotransduction {
    fn verify(&self) -> Verdict {
        if causes_transitively(
            BiophysicsConcept::ExternalVibration,
            BiophysicsConcept::MechanotransducerActivation,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VibrationCausesMechanotransduction",
        "External vibration transitively causes mechanotransducer activation via the wave-deformation-strain chain",
        "Duck (1990) Physical Properties of Tissue §4"
    );
}
pr4xis::register_axiom!(
    VibrationCausesMechanotransduction,
    "Duck (1990) Physical Properties of Tissue §4"
);

/// Axiom: piezoelectric charge generation follows tissue deformation.
///
/// Fukada & Yasuda (1957): the direct piezoelectric effect — mechanical stress
/// on bone produces electric polarization.
pub struct PiezoelectricFollowsDeformation;

impl Axiom for PiezoelectricFollowsDeformation {
    fn verify(&self) -> Verdict {
        if causes_transitively(
            BiophysicsConcept::TissueDeformation,
            BiophysicsConcept::PiezoelectricChargeGeneration,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PiezoelectricFollowsDeformation",
        "Tissue deformation causes piezoelectric charge generation (direct piezoelectric effect)",
        "Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162"
    );
}
pr4xis::register_axiom!(
    PiezoelectricFollowsDeformation,
    "Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162"
);

/// Axiom: bone matrix is piezoelectric (collagen content).
pub struct BoneMatrixIsPiezoelectric;

impl Axiom for BoneMatrixIsPiezoelectric {
    fn verify(&self) -> Verdict {
        if IsPiezoelectric.get(&BiophysicsConcept::BoneMatrix) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BoneMatrixIsPiezoelectric",
        "Bone matrix is piezoelectric owing to its collagen content",
        "Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162"
    );
}
pr4xis::register_axiom!(
    BoneMatrixIsPiezoelectric,
    "Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162"
);

/// Axiom: bone-matrix acoustic impedance exceeds soft-tissue impedance.
///
/// Duck (1990) Table 4.1: Z_bone ≈ 7.4 MRayl vs Z_soft_tissue ≈ 1.6 MRayl. The
/// large mismatch is what produces strong reflections at bone/soft-tissue
/// interfaces (clinical ultrasound shadowing).
pub struct BoneImpedanceGreaterThanSoftTissue;

impl Axiom for BoneImpedanceGreaterThanSoftTissue {
    fn verify(&self) -> Verdict {
        let bone = AcousticImpedanceValue.get(&BiophysicsConcept::BoneMatrix);
        let soft = AcousticImpedanceValue.get(&BiophysicsConcept::SoftTissue);
        match (bone, soft) {
            (Some(b), Some(s)) if b > s => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "BoneImpedanceGreaterThanSoftTissue",
        "Bone matrix acoustic impedance exceeds soft-tissue acoustic impedance",
        "Duck (1990) Physical Properties of Tissue Table 4.1"
    );
}
pr4xis::register_axiom!(
    BoneImpedanceGreaterThanSoftTissue,
    "Duck (1990) Physical Properties of Tissue Table 4.1"
);

/// Axiom: impedance mismatch causes wave reflection.
///
/// Duck (1990) §4: amplitude of the reflection coefficient is governed by
/// (Z₂ − Z₁)/(Z₂ + Z₁); non-zero mismatch ⇒ non-zero reflection.
pub struct ImpedanceMismatchCausesReflection;

impl Axiom for ImpedanceMismatchCausesReflection {
    fn verify(&self) -> Verdict {
        if causes_transitively(
            BiophysicsConcept::ImpedanceMismatch,
            BiophysicsConcept::WaveReflection,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ImpedanceMismatchCausesReflection",
        "Acoustic impedance mismatch at a boundary causes wave reflection",
        "Duck (1990) Physical Properties of Tissue §4"
    );
}
pr4xis::register_axiom!(
    ImpedanceMismatchCausesReflection,
    "Duck (1990) Physical Properties of Tissue §4"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<BiophysicsCategory>();
    }

    #[test]
    fn ontology_validates() {
        BiophysicsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 7 mech + 6 wave + 4 piezo + 3 membrane + 4 media + 5 umbrellas + 10 events = 39
        assert_eq!(BiophysicsConcept::variants().len(), 39);
    }

    // -- Domain-axiom tests --

    #[test]
    fn vibration_causes_mechanotransduction_axiom() {
        assert!(VibrationCausesMechanotransduction.verify().is_ok());
    }

    #[test]
    fn piezoelectric_follows_deformation_axiom() {
        assert!(PiezoelectricFollowsDeformation.verify().is_ok());
    }

    #[test]
    fn bone_matrix_is_piezoelectric_axiom() {
        assert!(BoneMatrixIsPiezoelectric.verify().is_ok());
    }

    #[test]
    fn bone_impedance_greater_than_soft_tissue_axiom() {
        assert!(BoneImpedanceGreaterThanSoftTissue.verify().is_ok());
    }

    #[test]
    fn impedance_mismatch_causes_reflection_axiom() {
        assert!(ImpedanceMismatchCausesReflection.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    fn subsumptions() -> Vec<(BiophysicsConcept, BiophysicsConcept)> {
        BiophysicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiophysicsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect()
    }

    #[test]
    fn mechanical_properties_subsume() {
        let subs = subsumptions();
        for c in [
            BiophysicsConcept::Viscoelasticity,
            BiophysicsConcept::Elasticity,
            BiophysicsConcept::Viscosity,
            BiophysicsConcept::StiffnessModulus,
            BiophysicsConcept::StrainRate,
            BiophysicsConcept::MechanicalStress,
            BiophysicsConcept::MechanicalStrain,
            BiophysicsConcept::MembraneCapacitance,
            BiophysicsConcept::MembraneTension,
            BiophysicsConcept::CellDeformation,
        ] {
            assert!(
                subs.contains(&(c, BiophysicsConcept::MechanicalProperty)),
                "{:?} should subsume under MechanicalProperty",
                c
            );
        }
    }

    #[test]
    fn wave_properties_subsume() {
        let subs = subsumptions();
        for c in [
            BiophysicsConcept::MechanicalWave,
            BiophysicsConcept::AcousticImpedance,
            BiophysicsConcept::Attenuation,
            BiophysicsConcept::Frequency,
            BiophysicsConcept::Wavelength,
            BiophysicsConcept::ResonanceFrequency,
        ] {
            assert!(
                subs.contains(&(c, BiophysicsConcept::WaveProperty)),
                "{:?} should subsume under WaveProperty",
                c
            );
        }
    }

    #[test]
    fn piezoelectric_properties_subsume() {
        let subs = subsumptions();
        for c in [
            BiophysicsConcept::PiezoelectricEffect,
            BiophysicsConcept::DirectPiezoelectric,
            BiophysicsConcept::ConversePiezoelectric,
            BiophysicsConcept::CollagenPiezoelectricity,
        ] {
            assert!(
                subs.contains(&(c, BiophysicsConcept::PiezoelectricProperty)),
                "{:?} should subsume under PiezoelectricProperty",
                c
            );
        }
    }

    #[test]
    fn direct_and_converse_subsume_piezoelectric_effect() {
        let subs = subsumptions();
        assert!(subs.contains(&(
            BiophysicsConcept::DirectPiezoelectric,
            BiophysicsConcept::PiezoelectricEffect
        )));
        assert!(subs.contains(&(
            BiophysicsConcept::ConversePiezoelectric,
            BiophysicsConcept::PiezoelectricEffect
        )));
    }

    #[test]
    fn media_subsume_biological_medium() {
        let subs = subsumptions();
        for c in [
            BiophysicsConcept::BoneMatrix,
            BiophysicsConcept::SoftTissue,
            BiophysicsConcept::FluidMedium,
            BiophysicsConcept::CellMembrane,
        ] {
            assert!(
                subs.contains(&(c, BiophysicsConcept::BiologicalMedium)),
                "{:?} should subsume under BiologicalMedium",
                c
            );
        }
    }

    #[test]
    fn events_subsume_under_biophysical_event() {
        let subs = subsumptions();
        for ev in [
            BiophysicsConcept::ExternalVibration,
            BiophysicsConcept::WavePropagation,
            BiophysicsConcept::TissueDeformation,
            BiophysicsConcept::PiezoelectricChargeGeneration,
            BiophysicsConcept::ImpedanceMismatch,
            BiophysicsConcept::WaveReflection,
        ] {
            assert!(
                subs.contains(&(ev, BiophysicsConcept::BiophysicalEvent)),
                "{:?} should subsume under BiophysicalEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[test]
    fn external_vibration_causes_wave_propagation_directly() {
        let direct: Vec<_> = BiophysicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiophysicsRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(direct.contains(&(
            BiophysicsConcept::ExternalVibration,
            BiophysicsConcept::WavePropagation
        )));
    }

    #[test]
    fn vibration_to_mechanotransduction_chain() {
        assert!(causes_transitively(
            BiophysicsConcept::ExternalVibration,
            BiophysicsConcept::MechanotransducerActivation,
        ));
    }

    #[test]
    fn deformation_to_local_electric_field() {
        // TissueDeformation → PiezoelectricChargeGeneration → LocalElectricField
        assert!(causes_transitively(
            BiophysicsConcept::TissueDeformation,
            BiophysicsConcept::LocalElectricField,
        ));
    }

    // -- Opposition-kind tests --

    #[test]
    fn direct_opposes_converse_piezoelectric() {
        let opps: Vec<_> = BiophysicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiophysicsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            BiophysicsConcept::DirectPiezoelectric,
            BiophysicsConcept::ConversePiezoelectric
        )));
        assert!(opps.contains(&(
            BiophysicsConcept::ConversePiezoelectric,
            BiophysicsConcept::DirectPiezoelectric
        )));
    }

    #[test]
    fn elasticity_opposes_viscosity() {
        let opps: Vec<_> = BiophysicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiophysicsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(BiophysicsConcept::Elasticity, BiophysicsConcept::Viscosity)));
    }

    #[test]
    fn stress_opposes_strain() {
        let opps: Vec<_> = BiophysicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiophysicsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            BiophysicsConcept::MechanicalStress,
            BiophysicsConcept::MechanicalStrain
        )));
    }

    // -- Quality tests --

    #[test]
    fn bone_impedance_value() {
        assert_eq!(
            AcousticImpedanceValue.get(&BiophysicsConcept::BoneMatrix),
            Some(7.4)
        );
    }

    #[test]
    fn soft_tissue_impedance_value() {
        assert_eq!(
            AcousticImpedanceValue.get(&BiophysicsConcept::SoftTissue),
            Some(1.6)
        );
    }

    #[test]
    fn fluid_impedance_value() {
        assert_eq!(
            AcousticImpedanceValue.get(&BiophysicsConcept::FluidMedium),
            Some(1.5)
        );
    }

    #[test]
    fn bone_is_piezoelectric() {
        assert_eq!(
            IsPiezoelectric.get(&BiophysicsConcept::BoneMatrix),
            Some(true)
        );
    }

    #[test]
    fn soft_tissue_is_not_piezoelectric() {
        assert_eq!(
            IsPiezoelectric.get(&BiophysicsConcept::SoftTissue),
            Some(false)
        );
    }

    #[test]
    fn all_media_transmit_vibration() {
        for medium in [
            BiophysicsConcept::BoneMatrix,
            BiophysicsConcept::SoftTissue,
            BiophysicsConcept::FluidMedium,
            BiophysicsConcept::CellMembrane,
        ] {
            assert_eq!(
                TransmitsVibration.get(&medium),
                Some(true),
                "{:?} should transmit vibration",
                medium
            );
        }
    }

    #[test]
    fn mechanical_wave_frequency_range() {
        assert_eq!(
            FrequencyRange.get(&BiophysicsConcept::MechanicalWave),
            Some((1.0, 200.0))
        );
    }

    #[test]
    fn resonance_frequency_range() {
        assert_eq!(
            FrequencyRange.get(&BiophysicsConcept::ResonanceFrequency),
            Some((20.0, 120.0))
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = BiophysicsConcept> {
        proptest::sample::select(BiophysicsConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BiophysicsCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BiophysicsOntology::axioms() {
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
            let variants: Vec<_> = BiophysicsConcept::variants();
            for m in BiophysicsCategory::morphisms() {
                if m.kind() == BiophysicsRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = BiophysicsCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == BiophysicsRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(
                    opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back",
                    a,
                    b
                );
            }
        }

        #[test]
        fn prop_impedance_positive(c in arb_concept()) {
            if let Some(z) = AcousticImpedanceValue.get(&c) {
                prop_assert!(z > 0.0, "impedance must be positive for {:?}", c);
            }
        }

        #[test]
        fn prop_frequency_range_min_lt_max(c in arb_concept()) {
            if let Some((lo, hi)) = FrequencyRange.get(&c) {
                prop_assert!(lo < hi, "frequency range invalid for {:?}", c);
            }
        }
    }
}
