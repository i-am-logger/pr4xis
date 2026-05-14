//! Chemistry — foundational chemistry of matter for biomedical modelling.
//!
//! States of matter, chemical bonding, physical properties, and solution
//! components. Causal events cover dissolution → ion dissociation →
//! electrolyte formation, acid-base reaction → pH change → protein
//! denaturation, temperature change → phase transition, and concentration
//! gradient → diffusion. Per `feedback_one_ontology_per_module` the
//! original split between `ChemistryEntity` and `ChemistryCausalEvent`
//! has been merged: events are first-class concepts subsumed by the
//! `ChemicalEvent` umbrella.
//!
//! # Literature
//!
//! - **Atkins & de Paula (2017)** *Physical Chemistry*, 11th ed., Oxford
//!   University Press — canonical undergraduate reference for states of
//!   matter, thermodynamics, electrolyte solutions, phase transitions,
//!   and diffusion (Fick's laws).
//! - **Pauling (1960)** *The Nature of the Chemical Bond*, 3rd ed.,
//!   Cornell University Press — foundational classification of ionic,
//!   covalent, hydrogen, van der Waals, and metallic bonding.
//! - **IUPAC (2014)** *Compendium of Chemical Terminology* ("Gold Book"),
//!   3rd ed., Royal Society of Chemistry — authoritative terminology for
//!   solute/solvent, electrolyte, buffer, pH, osmolarity, and related
//!   physical-property nomenclature.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Chemistry",
    source: "Atkins & de Paula (2017) Physical Chemistry 11th ed.; Pauling (1960) The Nature of the Chemical Bond 3rd ed.; IUPAC (2014) Compendium of Chemical Terminology (Gold Book) 3rd ed.",

    concepts: [
        // === States of matter (Atkins 2017 Ch. 1, 17) ===
        Solid,
        Liquid,
        Gas,
        Plasma,
        Gel,
        Colloid,

        // === Bonding (Pauling 1960) ===
        IonicBond,
        CovalentBond,
        HydrogenBond,
        VanDerWaals,
        Metallic,

        // === Physical properties (IUPAC Gold Book; Atkins 2017 Ch. 1, 5) ===
        PH,
        Concentration,
        Osmolarity,
        Temperature,
        Pressure,

        // === Solution components (IUPAC Gold Book) ===
        Solvent,
        Solute,
        Electrolyte,
        Buffer,

        // === Abstract umbrellas ===
        StateOfMatter,
        ChemicalBond,
        PhysicalProperty,
        SolutionComponent,
        ChemicalEvent,

        // === Causal events (Atkins 2017 Ch. 5 dissolution, Ch. 7 acid-base,
        //     Ch. 17 phase transitions, Ch. 19 diffusion) ===
        Dissolution,
        IonDissociation,
        ElectrolyteFormation,
        AcidBaseReaction,
        PHChange,
        ProteinDenaturation,
        TemperatureChange,
        PhaseTransition,
        ConcentrationGradient,
        Diffusion,
    ],

    labels: {
        Solid: ("en", "Solid",
            "Atkins (2017) §1: a state of matter in which constituent particles are held in fixed positions; rigid shape and definite volume."),
        Liquid: ("en", "Liquid",
            "Atkins (2017) §1: a state of matter with definite volume but no fixed shape; particles cohere but flow past one another."),
        Gas: ("en", "Gas",
            "Atkins (2017) §1: a state of matter with neither fixed shape nor volume; particles move freely and fill any container."),
        Plasma: ("en", "Plasma",
            "Atkins (2017) §17: an ionised state of matter consisting of free electrons and ions; conducts electricity."),
        Gel: ("en", "Gel",
            "IUPAC Gold Book entry G02600: a non-fluid colloidal network with a fluid expanded throughout its whole volume."),
        Colloid: ("en", "Colloid",
            "IUPAC Gold Book entry C01172: a dispersion of one substance in another in which dispersed particles are 1 nm-1 µm."),

        IonicBond: ("en", "Ionic bond",
            "Pauling (1960) §3: chemical bond formed by electrostatic attraction between ions of opposite charge."),
        CovalentBond: ("en", "Covalent bond",
            "Pauling (1960) §2: chemical bond formed by sharing of one or more electron pairs between atoms."),
        HydrogenBond: ("en", "Hydrogen bond",
            "Pauling (1960) §12: a weak directional bond between a hydrogen atom and an electronegative atom (N, O, F)."),
        VanDerWaals: ("en", "Van der Waals interaction",
            "Pauling (1960) §13: weak intermolecular forces (London dispersion, dipole-dipole, induced dipole) arising from electronic-density fluctuations."),
        Metallic: ("en", "Metallic bond",
            "Pauling (1960) §11: bonding in metals arising from a 'sea' of delocalised valence electrons shared across a lattice of cations."),

        PH: ("en", "pH",
            "IUPAC Gold Book entry P04524: the negative decadic logarithm of the activity of hydrogen ions in solution."),
        Concentration: ("en", "Concentration",
            "IUPAC Gold Book entry C01222: amount of solute per unit volume of solution (e.g. mol/L)."),
        Osmolarity: ("en", "Osmolarity",
            "IUPAC Gold Book entry O04342: total concentration of osmotically-active particles per litre of solution (osmol/L)."),
        Temperature: ("en", "Temperature",
            "Atkins (2017) §3 (zeroth and second laws): intensive property determining heat flow direction between systems."),
        Pressure: ("en", "Pressure",
            "Atkins (2017) §1: force per unit area exerted by a fluid; in gases, follows kinetic-theory dependence on temperature and number density."),

        Solvent: ("en", "Solvent",
            "IUPAC Gold Book entry S05746: the component of a solution that is present in the largest amount; the dissolving medium."),
        Solute: ("en", "Solute",
            "IUPAC Gold Book entry S05747: a substance dissolved in a solvent to form a solution."),
        Electrolyte: ("en", "Electrolyte",
            "IUPAC Gold Book entry E02008: a substance whose ionic constituents allow it to conduct electric current when dissolved in a polar solvent or molten."),
        Buffer: ("en", "Buffer",
            "IUPAC Gold Book entry B00863: a solution that resists changes in pH on dilution or on addition of small amounts of acid or base."),

        StateOfMatter: ("en", "State of matter",
            "Atkins (2017) §1: classification of macroscopic phases (solid, liquid, gas, plasma, ...) distinguished by structural order and mobility."),
        ChemicalBond: ("en", "Chemical bond",
            "Pauling (1960) §1: any attractive interaction between atoms that holds them together in a chemical species."),
        PhysicalProperty: ("en", "Physical property",
            "IUPAC Gold Book: a measurable property whose value describes a state of a physical system."),
        SolutionComponent: ("en", "Solution component",
            "IUPAC Gold Book: an identifiable constituent of a homogeneous mixture (solution)."),
        ChemicalEvent: ("en", "Chemical event",
            "Atkins (2017): umbrella for time-extended chemical processes (dissolution, reaction, phase change, diffusion)."),

        Dissolution: ("en", "Dissolution",
            "Atkins (2017) §5: the process by which a solute becomes uniformly dispersed in a solvent."),
        IonDissociation: ("en", "Ion dissociation",
            "Atkins (2017) §5: separation of an electrolyte into its constituent ions in solution."),
        ElectrolyteFormation: ("en", "Electrolyte formation",
            "Atkins (2017) §5: appearance of a charge-carrying solution following dissociation of an electrolyte solute."),
        AcidBaseReaction: ("en", "Acid-base reaction",
            "Atkins (2017) §7 (Brønsted-Lowry): proton transfer between an acid and a base."),
        PHChange: ("en", "pH change",
            "Atkins (2017) §7: a shift in hydrogen-ion activity (and hence pH) following an acid-base reaction or dilution."),
        ProteinDenaturation: ("en", "Protein denaturation",
            "IUPAC Gold Book entry D01580: loss of the higher-order structure of a protein, often driven by pH or temperature changes."),
        TemperatureChange: ("en", "Temperature change",
            "Atkins (2017) §3: alteration of thermal state of a system due to heat transfer or work."),
        PhaseTransition: ("en", "Phase transition",
            "Atkins (2017) §17: discontinuous change between states of matter (melting, vaporisation, sublimation, etc.)."),
        ConcentrationGradient: ("en", "Concentration gradient",
            "Atkins (2017) §19: spatial variation of solute concentration that drives diffusive flux."),
        Diffusion: ("en", "Diffusion",
            "Atkins (2017) §19 (Fick's first law): net transport of matter down a concentration gradient."),
    },

    is_a: [
        // States of matter
        (Solid, StateOfMatter),
        (Liquid, StateOfMatter),
        (Gas, StateOfMatter),
        (Plasma, StateOfMatter),
        (Colloid, StateOfMatter),
        (Gel, Colloid),
        (Colloid, Liquid),

        // Bonds
        (IonicBond, ChemicalBond),
        (CovalentBond, ChemicalBond),
        (HydrogenBond, ChemicalBond),
        (VanDerWaals, ChemicalBond),
        (Metallic, ChemicalBond),

        // Properties
        (PH, PhysicalProperty),
        (Concentration, PhysicalProperty),
        (Osmolarity, PhysicalProperty),
        (Temperature, PhysicalProperty),
        (Pressure, PhysicalProperty),

        // Solution components
        (Solvent, SolutionComponent),
        (Solute, SolutionComponent),
        (Electrolyte, SolutionComponent),
        (Buffer, SolutionComponent),

        // Events under the ChemicalEvent umbrella
        (Dissolution, ChemicalEvent),
        (IonDissociation, ChemicalEvent),
        (ElectrolyteFormation, ChemicalEvent),
        (AcidBaseReaction, ChemicalEvent),
        (PHChange, ChemicalEvent),
        (ProteinDenaturation, ChemicalEvent),
        (TemperatureChange, ChemicalEvent),
        (PhaseTransition, ChemicalEvent),
        (ConcentrationGradient, ChemicalEvent),
        (Diffusion, ChemicalEvent),
    ],

    causes: [
        // Atkins (2017) §5 — solvation: dissolution leads to dissociation,
        // which produces electrolyte (charge-carrying) solutions.
        (Dissolution, IonDissociation),
        (IonDissociation, ElectrolyteFormation),
        // Atkins (2017) §7 — Brønsted-Lowry: acid-base reactions shift pH;
        // large pH excursions denature protein tertiary structure.
        (AcidBaseReaction, PHChange),
        (PHChange, ProteinDenaturation),
        // Atkins (2017) §17 — Gibbs phase rule: temperature drives transitions
        // between states of matter at characteristic phase boundaries.
        (TemperatureChange, PhaseTransition),
        // Atkins (2017) §19 — Fick's first law: gradients drive diffusive flux.
        (ConcentrationGradient, Diffusion),
    ],

    opposes: [
        // Solvent ↔ Solute: dissolving agent vs dissolved substance
        // (IUPAC Gold Book S05746/S05747).
        (Solvent, Solute),
        (Solute, Solvent),
        // IonicBond ↔ CovalentBond: electrostatic transfer of electrons vs
        // shared electron-pair (Pauling 1960 §2-3 — the two paradigmatic
        // limiting bond types on the ionicity continuum).
        (IonicBond, CovalentBond),
        (CovalentBond, IonicBond),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a concept (as represented in this taxonomy) conducts electricity.
///
/// Atkins (2017) §5 — only solutions/states containing mobile charge carriers
/// conduct: electrolyte solutions (free ions) and plasma (free electrons).
#[derive(Debug, Clone)]
pub struct ConductsElectricity;

impl Quality for ConductsElectricity {
    type Individual = ChemistryConcept;
    type Value = bool;

    fn get(&self, c: &ChemistryConcept) -> Option<bool> {
        use ChemistryConcept::*;
        Some(matches!(c, Electrolyte | Plasma))
    }
}

/// Whether a concept describes a (partly-)aqueous phase.
///
/// IUPAC Gold Book — gel and colloid in this context denote biological
/// aqueous gels/colloids; pure liquid is the canonical aqueous phase here.
#[derive(Debug, Clone)]
pub struct IsAqueous;

impl Quality for IsAqueous {
    type Individual = ChemistryConcept;
    type Value = bool;

    fn get(&self, c: &ChemistryConcept) -> Option<bool> {
        use ChemistryConcept::*;
        Some(matches!(c, Liquid | Gel | Colloid))
    }
}

/// Bond-strength classification (kJ/mol bands from Pauling 1960).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondStrengthLevel {
    /// Pauling §2-3, §11: covalent / ionic / metallic, hundreds of kJ/mol.
    Strong,
    /// Pauling §12: hydrogen bonds, ~20-40 kJ/mol.
    Moderate,
    /// Pauling §13: van der Waals interactions, ~0.4-4 kJ/mol.
    Weak,
}

/// Quality: strength class of a chemical bond.
#[derive(Debug, Clone)]
pub struct BondStrength;

impl Quality for BondStrength {
    type Individual = ChemistryConcept;
    type Value = BondStrengthLevel;

    fn get(&self, c: &ChemistryConcept) -> Option<BondStrengthLevel> {
        use ChemistryConcept::*;
        match c {
            CovalentBond | IonicBond | Metallic => Some(BondStrengthLevel::Strong),
            HydrogenBond => Some(BondStrengthLevel::Moderate),
            VanDerWaals => Some(BondStrengthLevel::Weak),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for ChemistryOntology {
    type Cat = ChemistryCategory;
    type Qual = ConductsElectricity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DissolutionCausesIonDissociation));
        axioms.push(Box::new(AcidBaseCausesPHChange));
        axioms.push(Box::new(ElectrolytesConductElectricity));
        axioms.push(Box::new(BondStrengthOrder));
        axioms
    }
}

/// Helper: does a `Causation` edge exist from `cause` to `effect`?
fn causes(cause: ChemistryConcept, effect: ChemistryConcept) -> bool {
    ChemistryCategory::morphisms().iter().any(|m| {
        m.kind() == ChemistryRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

/// Axiom: Dissolution causes IonDissociation (Atkins 2017 §5).
pub struct DissolutionCausesIonDissociation;

impl Axiom for DissolutionCausesIonDissociation {
    fn verify(&self) -> Verdict {
        if causes(
            ChemistryConcept::Dissolution,
            ChemistryConcept::IonDissociation,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DissolutionCausesIonDissociation",
        "Dissolution of an electrolyte solute causes its dissociation into ions in solution",
        "Atkins & de Paula (2017) Physical Chemistry 11th ed. §5 (Solutions and chemical equilibrium)"
    );
}

pr4xis::register_axiom!(
    DissolutionCausesIonDissociation,
    "Atkins & de Paula (2017) Physical Chemistry 11th ed. §5"
);

/// Axiom: AcidBaseReaction causes PHChange (Atkins 2017 §7 Brønsted-Lowry).
pub struct AcidBaseCausesPHChange;

impl Axiom for AcidBaseCausesPHChange {
    fn verify(&self) -> Verdict {
        if causes(
            ChemistryConcept::AcidBaseReaction,
            ChemistryConcept::PHChange,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcidBaseCausesPHChange",
        "An acid-base (proton-transfer) reaction shifts the activity of H+ and therefore the pH",
        "Atkins & de Paula (2017) Physical Chemistry 11th ed. §7 (Acids and bases)"
    );
}

pr4xis::register_axiom!(
    AcidBaseCausesPHChange,
    "Atkins & de Paula (2017) Physical Chemistry 11th ed. §7"
);

/// Axiom: Electrolytes conduct electricity (IUPAC Gold Book E02008).
pub struct ElectrolytesConductElectricity;

impl Axiom for ElectrolytesConductElectricity {
    fn verify(&self) -> Verdict {
        if ConductsElectricity.get(&ChemistryConcept::Electrolyte) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ElectrolytesConductElectricity",
        "Electrolyte solutions conduct electric current via mobile ions",
        "IUPAC (2014) Compendium of Chemical Terminology (Gold Book) entry E02008"
    );
}

pr4xis::register_axiom!(
    ElectrolytesConductElectricity,
    "IUPAC (2014) Compendium of Chemical Terminology (Gold Book) entry E02008"
);

/// Axiom: Strong > Moderate > Weak bond ordering follows Pauling (1960).
///
/// Covalent / ionic / metallic bonds are hundreds of kJ/mol (Strong); hydrogen
/// bonds are tens of kJ/mol (Moderate); van der Waals interactions are a few
/// kJ/mol (Weak).
pub struct BondStrengthOrder;

impl Axiom for BondStrengthOrder {
    fn verify(&self) -> Verdict {
        let cov = BondStrength.get(&ChemistryConcept::CovalentBond);
        let ion = BondStrength.get(&ChemistryConcept::IonicBond);
        let met = BondStrength.get(&ChemistryConcept::Metallic);
        let h = BondStrength.get(&ChemistryConcept::HydrogenBond);
        let vdw = BondStrength.get(&ChemistryConcept::VanDerWaals);
        if cov == Some(BondStrengthLevel::Strong)
            && ion == Some(BondStrengthLevel::Strong)
            && met == Some(BondStrengthLevel::Strong)
            && h == Some(BondStrengthLevel::Moderate)
            && vdw == Some(BondStrengthLevel::Weak)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BondStrengthOrder",
        "Covalent / ionic / metallic bonds are Strong; hydrogen bonds Moderate; van der Waals Weak",
        "Pauling (1960) The Nature of the Chemical Bond 3rd ed. §2-3, §11-13"
    );
}

pr4xis::register_axiom!(
    BondStrengthOrder,
    "Pauling (1960) The Nature of the Chemical Bond 3rd ed. §2-3, §11-13"
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
        assert_category_laws::<ChemistryCategory>();
    }

    #[test]
    fn ontology_validates() {
        ChemistryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 6 states + 5 bonds + 5 properties + 4 solution components
        // + 5 abstract umbrellas + 10 events = 35.
        assert_eq!(ChemistryConcept::variants().len(), 35);
    }

    // -- Domain axiom tests --

    #[test]
    fn dissolution_causes_ion_dissociation_axiom() {
        assert!(DissolutionCausesIonDissociation.verify().is_ok());
    }

    #[test]
    fn acid_base_causes_ph_change_axiom() {
        assert!(AcidBaseCausesPHChange.verify().is_ok());
    }

    #[test]
    fn electrolytes_conduct_electricity_axiom() {
        assert!(ElectrolytesConductElectricity.verify().is_ok());
    }

    #[test]
    fn bond_strength_order_axiom() {
        assert!(BondStrengthOrder.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    #[test]
    fn states_of_matter_subsume_under_umbrella() {
        let subs: Vec<_> = ChemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChemistryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(subs.contains(&(ChemistryConcept::Solid, ChemistryConcept::StateOfMatter)));
        assert!(subs.contains(&(ChemistryConcept::Gel, ChemistryConcept::Colloid)));
        assert!(subs.contains(&(ChemistryConcept::Colloid, ChemistryConcept::Liquid)));
    }

    #[test]
    fn events_subsume_under_chemical_event() {
        let subs: Vec<_> = ChemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChemistryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
            ChemistryConcept::Dissolution,
            ChemistryConcept::AcidBaseReaction,
            ChemistryConcept::TemperatureChange,
            ChemistryConcept::ConcentrationGradient,
            ChemistryConcept::PhaseTransition,
        ] {
            assert!(
                subs.contains(&(ev, ChemistryConcept::ChemicalEvent)),
                "{:?} should subsume under ChemicalEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[test]
    fn dissolution_causes_dissociation_via_kind() {
        assert!(causes(
            ChemistryConcept::Dissolution,
            ChemistryConcept::IonDissociation
        ));
    }

    #[test]
    fn temperature_change_causes_phase_transition() {
        assert!(causes(
            ChemistryConcept::TemperatureChange,
            ChemistryConcept::PhaseTransition
        ));
    }

    #[test]
    fn concentration_gradient_causes_diffusion() {
        assert!(causes(
            ChemistryConcept::ConcentrationGradient,
            ChemistryConcept::Diffusion
        ));
    }

    // -- Opposition-kind tests --

    #[test]
    fn solvent_and_solute_oppose() {
        let opps: Vec<_> = ChemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChemistryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(ChemistryConcept::Solvent, ChemistryConcept::Solute)));
        assert!(opps.contains(&(ChemistryConcept::Solute, ChemistryConcept::Solvent)));
    }

    #[test]
    fn ionic_and_covalent_oppose() {
        let opps: Vec<_> = ChemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChemistryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(ChemistryConcept::IonicBond, ChemistryConcept::CovalentBond)));
        assert!(opps.contains(&(ChemistryConcept::CovalentBond, ChemistryConcept::IonicBond)));
    }

    // -- Quality tests --

    #[test]
    fn electrolyte_conducts() {
        assert_eq!(
            ConductsElectricity.get(&ChemistryConcept::Electrolyte),
            Some(true)
        );
    }

    #[test]
    fn plasma_conducts() {
        assert_eq!(
            ConductsElectricity.get(&ChemistryConcept::Plasma),
            Some(true)
        );
    }

    #[test]
    fn gas_does_not_conduct() {
        assert_eq!(ConductsElectricity.get(&ChemistryConcept::Gas), Some(false));
    }

    #[test]
    fn liquid_is_aqueous() {
        assert_eq!(IsAqueous.get(&ChemistryConcept::Liquid), Some(true));
    }

    #[test]
    fn solid_not_aqueous() {
        assert_eq!(IsAqueous.get(&ChemistryConcept::Solid), Some(false));
    }

    #[test]
    fn bond_strength_levels() {
        assert_eq!(
            BondStrength.get(&ChemistryConcept::CovalentBond),
            Some(BondStrengthLevel::Strong)
        );
        assert_eq!(
            BondStrength.get(&ChemistryConcept::HydrogenBond),
            Some(BondStrengthLevel::Moderate)
        );
        assert_eq!(
            BondStrength.get(&ChemistryConcept::VanDerWaals),
            Some(BondStrengthLevel::Weak)
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = ChemistryConcept> {
        proptest::sample::select(ChemistryConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ChemistryCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ChemistryOntology::axioms() {
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
            let variants: Vec<_> = ChemistryConcept::variants();
            for m in ChemistryCategory::morphisms() {
                if m.kind() == ChemistryRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = ChemistryCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == ChemistryRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_conductivity_total(c in arb_concept()) {
            // ConductsElectricity is total over every concept.
            prop_assert!(ConductsElectricity.get(&c).is_some());
        }

        #[test]
        fn prop_aqueous_total(c in arb_concept()) {
            prop_assert!(IsAqueous.get(&c).is_some());
        }
    }
}
