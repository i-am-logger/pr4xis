use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::quantity::dimension::Dimension;
use crate::science::math::quantity::unit;
use crate::science::math::quantity::value::Quantity;

// ---------------------------------------------------------------------------
// Entity: SI base dimensions
// ---------------------------------------------------------------------------

/// The 7 SI base dimensions — the basis of the dimension group.
///
/// Source: BIPM SI Brochure (2019), Table 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseDimension {
    Length,
    Mass,
    Time,
    ElectricCurrent,
    Temperature,
    AmountOfSubstance,
    LuminousIntensity,
    Dimensionless,
}

impl Entity for BaseDimension {
    fn variants() -> Vec<Self> {
        vec![
            Self::Length,
            Self::Mass,
            Self::Time,
            Self::ElectricCurrent,
            Self::Temperature,
            Self::AmountOfSubstance,
            Self::LuminousIntensity,
            Self::Dimensionless,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionRelation {
    pub from: BaseDimension,
    pub to: BaseDimension,
}

impl Relationship for DimensionRelation {
    type Object = BaseDimension;
    fn source(&self) -> BaseDimension {
        self.from
    }
    fn target(&self) -> BaseDimension {
        self.to
    }
}

pub struct DimensionCategory;

impl Category for DimensionCategory {
    type Object = BaseDimension;
    type Morphism = DimensionRelation;

    fn identity(obj: &BaseDimension) -> DimensionRelation {
        DimensionRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &DimensionRelation, g: &DimensionRelation) -> Option<DimensionRelation> {
        if f.to != g.from {
            return None;
        }
        Some(DimensionRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<DimensionRelation> {
        let dims = BaseDimension::variants();
        dims.iter()
            .flat_map(|&from| dims.iter().map(move |&to| DimensionRelation { from, to }))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DimensionSymbol;

impl Quality for DimensionSymbol {
    type Individual = BaseDimension;
    type Value = &'static str;

    fn get(&self, d: &BaseDimension) -> Option<&'static str> {
        Some(match d {
            BaseDimension::Length => "L",
            BaseDimension::Mass => "M",
            BaseDimension::Time => "T",
            BaseDimension::ElectricCurrent => "I",
            BaseDimension::Temperature => "Θ",
            BaseDimension::AmountOfSubstance => "N",
            BaseDimension::LuminousIntensity => "J",
            BaseDimension::Dimensionless => "1",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — dimension group and quantity calculus
// ---------------------------------------------------------------------------

/// Dimensions form an abelian group: multiplication is commutative.
/// [A]·[B] = [B]·[A].
pub struct DimensionCommutativity;

impl Axiom for DimensionCommutativity {
    fn description(&self) -> &str {
        "dimension multiplication is commutative: [A]·[B] = [B]·[A]"
    }
    fn holds(&self) -> bool {
        let dims = canonical_dimensions();
        for a in &dims {
            for b in &dims {
                if a.multiply(b) != b.multiply(a) {
                    return false;
                }
            }
        }
        true
    }
}

/// Dimensions form an abelian group: multiplication is associative.
pub struct DimensionAssociativity;

impl Axiom for DimensionAssociativity {
    fn description(&self) -> &str {
        "dimension multiplication is associative: ([A]·[B])·[C] = [A]·([B]·[C])"
    }
    fn holds(&self) -> bool {
        let dims = canonical_dimensions();
        for a in &dims {
            for b in &dims {
                for c in &dims {
                    if a.multiply(b).multiply(c) != a.multiply(&b.multiply(c)) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Dimensionless is the identity element.
pub struct DimensionIdentity;

impl Axiom for DimensionIdentity {
    fn description(&self) -> &str {
        "dimensionless is the identity: [A]·1 = [A]"
    }
    fn holds(&self) -> bool {
        let one = Dimension::DIMENSIONLESS;
        for d in &canonical_dimensions() {
            if d.multiply(&one) != *d || one.multiply(d) != *d {
                return false;
            }
        }
        true
    }
}

/// Every dimension has an inverse: [A]·[A]⁻¹ = 1.
pub struct DimensionInverse;

impl Axiom for DimensionInverse {
    fn description(&self) -> &str {
        "every dimension has an inverse: [A]·[A]^{-1} = dimensionless"
    }
    fn holds(&self) -> bool {
        for d in &canonical_dimensions() {
            if !d.multiply(&d.inverse()).is_dimensionless() {
                return false;
            }
        }
        true
    }
}

/// Cannot add quantities with different dimensions.
pub struct AdditionRequiresSameDimension;

impl Axiom for AdditionRequiresSameDimension {
    fn description(&self) -> &str {
        "addition requires same dimension: meters + seconds = error"
    }
    fn holds(&self) -> bool {
        let length = Quantity::new(5.0, Dimension::LENGTH);
        let time = Quantity::new(3.0, Dimension::TIME);
        let velocity = Quantity::new(2.0, Dimension::VELOCITY);

        // Same dimension: OK
        let ok = length.add(&Quantity::new(3.0, Dimension::LENGTH));
        if ok.is_none() {
            return false;
        }

        // Different dimension: error
        let err1 = length.add(&time);
        if err1.is_some() {
            return false;
        }

        let err2 = length.add(&velocity);
        if err2.is_some() {
            return false;
        }

        true
    }
}

/// Multiplication produces correct derived dimension.
/// velocity = length / time → [v] = L·T⁻¹.
pub struct DerivedDimensionConsistency;

impl Axiom for DerivedDimensionConsistency {
    fn description(&self) -> &str {
        "velocity = length / time: [v] = L·T^{-1}"
    }
    fn holds(&self) -> bool {
        let l = Quantity::new(10.0, Dimension::LENGTH);
        let t = Quantity::new(2.0, Dimension::TIME);
        let v = l.div(&t);

        v.dimension == Dimension::VELOCITY && (v.value - 5.0).abs() < 1e-10
    }
}

/// Unit conversion roundtrip: km → m → km.
pub struct UnitConversionRoundtrip;

impl Axiom for UnitConversionRoundtrip {
    fn description(&self) -> &str {
        "unit conversion is invertible: km -> m -> km roundtrip"
    }
    fn holds(&self) -> bool {
        let km_val = 5.0;
        let m_val = unit::KILOMETER.to_si(km_val);
        let km_back = unit::KILOMETER.from_si(m_val);
        (km_val - km_back).abs() < 1e-10 && (m_val - 5000.0).abs() < 1e-10
    }
}

/// Incompatible units cannot convert: meters cannot become seconds.
pub struct IncompatibleUnitConversionFails;

impl Axiom for IncompatibleUnitConversionFails {
    fn description(&self) -> &str {
        "incompatible unit conversion returns None: meters ≠ seconds"
    }
    fn holds(&self) -> bool {
        unit::METER.convert(5.0, &unit::SECOND).is_none()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The quantity ontology — dimensional analysis as formal algebra.
///
/// Founded on:
///   - BIPM SI Brochure (2019). International System of Units.
///   - Tao, T. (2012). "A mathematical formalization of dimensional analysis."
///   - Hart, J. (2021). "Dimensioned Algebra." ArXiv 2108.08703.
///   - QUDT (qudt.org). Quantities, Units, Dimensions and Types.
///   - Sonin, A.A. "The Physical Basis of Dimensional Analysis." MIT.
pub struct QuantityOntology;

impl Ontology for QuantityOntology {
    type Cat = DimensionCategory;
    type Qual = DimensionSymbol;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DimensionCommutativity),
            Box::new(DimensionAssociativity),
            Box::new(DimensionIdentity),
            Box::new(DimensionInverse),
            Box::new(AdditionRequiresSameDimension),
            Box::new(DerivedDimensionConsistency),
            Box::new(UnitConversionRoundtrip),
            Box::new(IncompatibleUnitConversionFails),
        ]
    }
}

fn canonical_dimensions() -> Vec<Dimension> {
    vec![
        Dimension::DIMENSIONLESS,
        Dimension::LENGTH,
        Dimension::MASS,
        Dimension::TIME,
        Dimension::VELOCITY,
        Dimension::ACCELERATION,
        Dimension::FORCE,
        Dimension::ENERGY,
        Dimension::FREQUENCY,
    ]
}
