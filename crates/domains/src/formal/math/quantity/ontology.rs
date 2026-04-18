//! SI base dimensions — the basis of the dimension group.
//!
//! Source: BIPM SI Brochure (2019), Table 1.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Quantity",
    source: "BIPM SI Brochure (2019)",
    being: AbstractObject,

    concepts: [
        Length,
        Mass,
        Time,
        ElectricCurrent,
        Temperature,
        AmountOfSubstance,
        LuminousIntensity,
        Dimensionless,
    ],

    labels: {
        Length: ("en", "Length", "Length dimension (SI base)."),
        Mass: ("en", "Mass", "Mass dimension (SI base)."),
        Time: ("en", "Time", "Time dimension (SI base)."),
        ElectricCurrent: ("en", "Electric current", "Electric current dimension (SI base)."),
        Temperature: ("en", "Temperature", "Thermodynamic temperature dimension (SI base)."),
        AmountOfSubstance: ("en", "Amount of substance", "Amount of substance dimension (SI base)."),
        LuminousIntensity: ("en", "Luminous intensity", "Luminous intensity dimension (SI base)."),
        Dimensionless: ("en", "Dimensionless", "The dimensionless identity (1)."),
    },
}

#[derive(Debug, Clone)]
pub struct DimensionSymbol;

impl Quality for DimensionSymbol {
    type Individual = QuantityConcept;
    type Value = &'static str;

    fn get(&self, d: &QuantityConcept) -> Option<&'static str> {
        Some(match d {
            QuantityConcept::Length => "L",
            QuantityConcept::Mass => "M",
            QuantityConcept::Time => "T",
            QuantityConcept::ElectricCurrent => "I",
            QuantityConcept::Temperature => "Θ",
            QuantityConcept::AmountOfSubstance => "N",
            QuantityConcept::LuminousIntensity => "J",
            QuantityConcept::Dimensionless => "1",
        })
    }
}

/// Dimensions form an abelian group: multiplication is commutative.
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
pr4xis::register_axiom!(DimensionCommutativity, "BIPM SI Brochure (2019), Table 1.");

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
pr4xis::register_axiom!(DimensionAssociativity, "BIPM SI Brochure (2019), Table 1.");

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
pr4xis::register_axiom!(DimensionIdentity, "BIPM SI Brochure (2019), Table 1.");

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
pr4xis::register_axiom!(DimensionInverse, "BIPM SI Brochure (2019), Table 1.");

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

        let ok = length.add(&Quantity::new(3.0, Dimension::LENGTH));
        if ok.is_none() {
            return false;
        }

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
pr4xis::register_axiom!(
    AdditionRequiresSameDimension,
    "BIPM SI Brochure (2019), Table 1."
);

/// Multiplication produces correct derived dimension.
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
pr4xis::register_axiom!(
    DerivedDimensionConsistency,
    "BIPM SI Brochure (2019), Table 1."
);

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
pr4xis::register_axiom!(UnitConversionRoundtrip, "BIPM SI Brochure (2019), Table 1.");

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
pr4xis::register_axiom!(
    IncompatibleUnitConversionFails,
    "BIPM SI Brochure (2019), Table 1."
);

impl Ontology for QuantityOntology {
    type Cat = QuantityCategory;
    type Qual = DimensionSymbol;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<QuantityCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        QuantityOntology::validate().unwrap();
    }
}
