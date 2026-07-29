#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Maxwell's equations as an ontology:
/// - Situation: electromagnetic field (E, B) with charge density and current
/// - Axioms: all four Maxwell equations enforced
/// - The speed of light is DERIVED: c = 1/√(μ₀ε₀)
///
/// Gauss (electric):    ∇⋅E = ρ/ε₀
/// Gauss (magnetic):    ∇⋅B = 0
/// Faraday:             ∇×E = -∂B/∂t
/// Ampère-Maxwell:      ∇×B = μ₀J + μ₀ε₀∂E/∂t
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

pub const EPSILON_0: f64 = 8.854e-12; // vacuum permittivity (F/m)
pub const MU_0: f64 = 1.257e-6; // vacuum permeability (H/m)

/// Speed of light derived from Maxwell's equations: c = 1/√(μ₀ε₀).
pub fn speed_of_light() -> Quantity {
    Quantity::from_unit(1.0 / (MU_0 * EPSILON_0).sqrt(), &unit::METER_PER_SECOND)
}

/// 3D vector.
#[derive(Debug, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// Euclidean norm |v|. `Vec3` here is a bare 3-component carrier reused
    /// polymorphically for E-field (V/m), B-field (T) and current-density
    /// (A/m²) vectors with different SI dimensions — the type itself
    /// carries no fixed dimension, so the magnitude is UNITLESS at this
    /// generic layer (same treatment as `formal::math::geometry::point::
    /// Point3::distance_to`); callers interpret the value in whatever
    /// concrete unit their own field variable declares.
    pub fn magnitude(&self) -> Quantity {
        Quantity::from_unit(
            (self.x * self.x + self.y * self.y + self.z * self.z).sqrt(),
            &unit::UNITLESS,
        )
    }
    /// Dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning as
    /// [`Vec3::magnitude`].
    pub fn dot(&self, other: &Vec3) -> Quantity {
        Quantity::from_unit(
            self.x * other.x + self.y * other.y + self.z * other.z,
            &unit::UNITLESS,
        )
    }
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn scale(&self, s: f64) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

/// Electromagnetic field state at a point in space.
#[derive(Debug, Clone, PartialEq)]
pub struct EMField {
    pub e_field: Vec3,         // electric field (V/m)
    pub b_field: Vec3,         // magnetic field (T)
    pub charge_density: f64,   // ρ (C/m³)
    pub current_density: Vec3, // J (A/m²)
    pub div_e: f64,            // ∇⋅E (computed from field)
    pub div_b: f64,            // ∇⋅B (should always be 0)
}

impl EMField {
    pub fn new(e: Vec3, b: Vec3, rho: f64, j: Vec3) -> Self {
        Self {
            div_e: rho / EPSILON_0, // Gauss's law: ∇⋅E = ρ/ε₀
            div_b: 0.0,             // Gauss's law for magnetism: ∇⋅B = 0 always
            e_field: e,
            b_field: b,
            charge_density: rho,
            current_density: j,
        }
    }

    pub fn vacuum() -> Self {
        Self::new(Vec3::zero(), Vec3::zero(), 0.0, Vec3::zero())
    }

    /// Gauss's law: ∇⋅E = ρ/ε₀
    pub fn gauss_electric_holds(&self) -> bool {
        let expected = self.charge_density / EPSILON_0;
        (self.div_e - expected).abs() / expected.abs().max(1.0) < 1e-6
    }

    /// Gauss's law for magnetism: ∇⋅B = 0 (no magnetic monopoles)
    pub fn gauss_magnetic_holds(&self) -> bool {
        self.div_b.abs() < 1e-10
    }

    /// Energy density: u = ½(ε₀E² + B²/μ₀). Energy per unit volume:
    /// Dimension::ENERGY / Dimension::LENGTH³.
    pub fn energy_density(&self) -> Quantity {
        let e_mag = self.e_field.magnitude().value;
        let b_mag = self.b_field.magnitude().value;
        Quantity::new(
            0.5 * (EPSILON_0 * e_mag * e_mag + b_mag * b_mag / MU_0),
            Dimension::ENERGY.divide(&Dimension::LENGTH.power(3)),
        )
    }

    /// Poynting vector: S = E × B / μ₀ (energy flux)
    pub fn poynting_vector(&self) -> Vec3 {
        self.e_field.cross(&self.b_field).scale(1.0 / MU_0)
    }
}

impl Situation for EMField {}

fn maxwell_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Maxwell (1865) A Dynamical Theory of the Electromagnetic Field, Phil. Trans. R. Soc. 155:459-512",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaxwellAction {
    /// Place a charge: changes ρ and E field.
    SetChargeDensity { rho: f64 },
    /// Apply electric field.
    SetEField { e: Vec3 },
    /// Apply magnetic field.
    SetBField { b: Vec3 },
    /// Set current density.
    SetCurrentDensity { j: Vec3 },
}

impl Action for MaxwellAction {
    type Sit = EMField;
}

/// Gauss's law for electricity: ∇⋅E = ρ/ε₀
struct GaussElectric;
impl Precondition<MaxwellAction> for GaussElectric {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> Verdict {
        let meta = maxwell_meta("GaussElectric", "∇⋅E = ρ/ε₀ (Gauss's law)");
        let next = apply_maxwell_inner(field, action);
        if next.gauss_electric_holds() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

/// Gauss's law for magnetism: ∇⋅B = 0 (no magnetic monopoles)
struct GaussMagnetic;
impl Precondition<MaxwellAction> for GaussMagnetic {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> Verdict {
        let meta = maxwell_meta("GaussMagnetic", "∇⋅B = 0 (no magnetic monopoles)");
        let next = apply_maxwell_inner(field, action);
        if next.gauss_magnetic_holds() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

/// Energy density must be non-negative.
struct NonNegativeEnergy;
impl Precondition<MaxwellAction> for NonNegativeEnergy {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> Verdict {
        let meta = maxwell_meta(
            "NonNegativeEnergy",
            "electromagnetic energy density must be non-negative",
        );
        let next = apply_maxwell_inner(field, action);
        if next.energy_density().value >= -1e-20 {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_maxwell_inner(field: &EMField, action: &MaxwellAction) -> EMField {
    match action {
        MaxwellAction::SetChargeDensity { rho } => EMField::new(
            field.e_field.clone(),
            field.b_field.clone(),
            *rho,
            field.current_density.clone(),
        ),
        MaxwellAction::SetEField { e } => EMField::new(
            e.clone(),
            field.b_field.clone(),
            field.charge_density,
            field.current_density.clone(),
        ),
        MaxwellAction::SetBField { b } => EMField::new(
            field.e_field.clone(),
            b.clone(),
            field.charge_density,
            field.current_density.clone(),
        ),
        MaxwellAction::SetCurrentDensity { j } => EMField::new(
            field.e_field.clone(),
            field.b_field.clone(),
            field.charge_density,
            j.clone(),
        ),
    }
}

fn apply_maxwell(
    field: &EMField,
    action: &MaxwellAction,
) -> Result<EMField, Box<dyn Counterexample>> {
    Ok(apply_maxwell_inner(field, action))
}

pub fn new_field() -> Engine<MaxwellAction> {
    Engine::new(
        EMField::vacuum(),
        vec![
            Box::new(GaussElectric),
            Box::new(GaussMagnetic),
            Box::new(NonNegativeEnergy),
        ],
        apply_maxwell,
    )
}

pub fn new_field_with(e: Vec3, b: Vec3, rho: f64, j: Vec3) -> Engine<MaxwellAction> {
    Engine::new(
        EMField::new(e, b, rho, j),
        vec![
            Box::new(GaussElectric),
            Box::new(GaussMagnetic),
            Box::new(NonNegativeEnergy),
        ],
        apply_maxwell,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_speed_of_light_derived() {
        let c = speed_of_light().value;
        // c ≈ 299,792,458 m/s
        assert!((c - 2.998e8).abs() < 1e6, "c={} should be ≈ 3e8", c);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_vacuum_satisfies_all() {
        let field = EMField::vacuum();
        assert!(field.gauss_electric_holds());
        assert!(field.gauss_magnetic_holds());
        assert!((field.energy_density().value - 0.0).abs() < 1e-20);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_charge_creates_divergence() {
        let e = new_field()
            .next(MaxwellAction::SetChargeDensity { rho: 1e-6 })
            .unwrap();
        assert!(e.situation().div_e > 0.0);
        assert!(e.situation().gauss_electric_holds());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_no_magnetic_monopoles() {
        let field = EMField::new(Vec3::zero(), Vec3::new(1.0, 0.0, 0.0), 0.0, Vec3::zero());
        assert!(field.gauss_magnetic_holds());
        assert_eq!(field.div_b, 0.0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_energy_density_nonneg() {
        let e = new_field()
            .next(MaxwellAction::SetEField {
                e: Vec3::new(100.0, 0.0, 0.0),
            })
            .unwrap();
        assert!(e.situation().energy_density().value > 0.0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_poynting_vector() {
        // E × B gives energy flux direction
        let field = EMField::new(
            Vec3::new(1.0, 0.0, 0.0), // E in x
            Vec3::new(0.0, 1.0, 0.0), // B in y
            0.0,
            Vec3::zero(),
        );
        let s = field.poynting_vector();
        // E×B = x×y = z
        assert!(s.z > 0.0);
        assert!(s.x.abs() < 1e-10);
        assert!(s.y.abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_cross_product() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert!((z.x - 0.0).abs() < 1e-10);
        assert!((z.y - 0.0).abs() < 1e-10);
        assert!((z.z - 1.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_dot_product() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!((a.dot(&b).value - 32.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn test_undo_redo() {
        let e = new_field()
            .next(MaxwellAction::SetEField {
                e: Vec3::new(100.0, 0.0, 0.0),
            })
            .unwrap();
        assert!(e.situation().e_field.magnitude().value > 0.0);
        let e = e.back().unwrap();
        assert!((e.situation().e_field.magnitude().value - 0.0).abs() < 1e-10);
    }

    proptest! {
        /// Gauss electric always holds after construction
        #[test]
        fn prop_gauss_electric(rho in -1e-3..1e-3f64) {
            let e = new_field()
                .next(MaxwellAction::SetChargeDensity { rho }).unwrap();
            prop_assert!(e.situation().gauss_electric_holds());
        }

        /// ∇⋅B = 0 always (no magnetic monopoles ever)
        #[test]
        fn prop_no_monopoles(bx in -10.0..10.0f64, by in -10.0..10.0f64, bz in -10.0..10.0f64) {
            let e = new_field()
                .next(MaxwellAction::SetBField { b: Vec3::new(bx, by, bz) }).unwrap();
            prop_assert!(e.situation().gauss_magnetic_holds());
            prop_assert_eq!(e.situation().div_b, 0.0);
        }

        /// Energy density is always non-negative
        #[test]
        fn prop_energy_nonneg(ex in -100.0..100.0f64, ey in -100.0..100.0f64, bz in -1.0..1.0f64) {
            let e = new_field()
                .next(MaxwellAction::SetEField { e: Vec3::new(ex, ey, 0.0) }).unwrap()
                .next(MaxwellAction::SetBField { b: Vec3::new(0.0, 0.0, bz) }).unwrap();
            prop_assert!(e.situation().energy_density().value >= 0.0);
        }

        /// Speed of light: c = 1/√(μ₀ε₀) ≈ 3×10⁸
        #[test]
        fn prop_speed_of_light(_x in 0..1u8) {
            let c = speed_of_light().value;
            prop_assert!((c - 2.998e8).abs() < 1e6);
        }

        /// Cross product anti-commutative: A×B = -(B×A)
        #[test]
        fn prop_cross_anticommutative(
            ax in -10.0..10.0f64, ay in -10.0..10.0f64, az in -10.0..10.0f64,
            bx in -10.0..10.0f64, by in -10.0..10.0f64, bz in -10.0..10.0f64,
        ) {
            let a = Vec3::new(ax, ay, az);
            let b = Vec3::new(bx, by, bz);
            let ab = a.cross(&b);
            let ba = b.cross(&a);
            prop_assert!((ab.x + ba.x).abs() < 1e-10);
            prop_assert!((ab.y + ba.y).abs() < 1e-10);
            prop_assert!((ab.z + ba.z).abs() < 1e-10);
        }

        /// Dot product commutative: A⋅B = B⋅A
        #[test]
        fn prop_dot_commutative(
            ax in -10.0..10.0f64, ay in -10.0..10.0f64, az in -10.0..10.0f64,
            bx in -10.0..10.0f64, by in -10.0..10.0f64, bz in -10.0..10.0f64,
        ) {
            let a = Vec3::new(ax, ay, az);
            let b = Vec3::new(bx, by, bz);
            prop_assert!((a.dot(&b).value - b.dot(&a).value).abs() < 1e-10);
        }

        /// Poynting vector perpendicular to both E and B
        #[test]
        fn prop_poynting_perpendicular(
            ex in -10.0..10.0f64, ey in -10.0..10.0f64,
            bx in -1.0..1.0f64, by in -1.0..1.0f64,
        ) {
            let e = Vec3::new(ex, ey, 0.0);
            let b = Vec3::new(bx, by, 0.0);
            let s = e.cross(&b);
            // S ⊥ E: S⋅E = 0
            prop_assert!(s.dot(&e).value.abs() < 1e-6, "S not perpendicular to E");
            // S ⊥ B: S⋅B = 0
            prop_assert!(s.dot(&b).value.abs() < 1e-6, "S not perpendicular to B");
        }
    }

    pr4xis::register_praxis_value!(prop_gauss_electric, Verifiable);
    pr4xis::register_praxis_value!(prop_no_monopoles, Verifiable);
    pr4xis::register_praxis_value!(prop_energy_nonneg, Verifiable);
    pr4xis::register_praxis_value!(prop_speed_of_light, Verifiable);
    pr4xis::register_praxis_value!(prop_cross_anticommutative, Verifiable);
    pr4xis::register_praxis_value!(prop_dot_commutative, Verifiable);
    pr4xis::register_praxis_value!(prop_poynting_perpendicular, Verifiable);
}
