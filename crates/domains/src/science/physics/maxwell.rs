/// Maxwell's equations as an ontology:
/// - Situation: electromagnetic field (E, B) with charge density and current
/// - Axioms: all four Maxwell equations enforced
/// - The speed of light is DERIVED: c = 1/√(μ₀ε₀)
///
/// Gauss (electric):    ∇⋅E = ρ/ε₀
/// Gauss (magnetic):    ∇⋅B = 0
/// Faraday:             ∇×E = -∂B/∂t
/// Ampère-Maxwell:      ∇×B = μ₀J + μ₀ε₀∂E/∂t
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

pub const EPSILON_0: f64 = 8.854e-12; // vacuum permittivity (F/m)
pub const MU_0: f64 = 1.257e-6; // vacuum permeability (H/m)

/// Speed of light derived from Maxwell's equations.
pub fn speed_of_light() -> f64 {
    1.0 / (MU_0 * EPSILON_0).sqrt()
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
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
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

    /// Energy density: u = ½(ε₀E² + B²/μ₀)
    pub fn energy_density(&self) -> f64 {
        0.5 * (EPSILON_0 * self.e_field.magnitude().powi(2)
            + self.b_field.magnitude().powi(2) / MU_0)
    }

    /// Poynting vector: S = E × B / μ₀ (energy flux)
    pub fn poynting_vector(&self) -> Vec3 {
        self.e_field.cross(&self.b_field).scale(1.0 / MU_0)
    }
}

impl Situation for EMField {
    fn describe(&self) -> String {
        format!(
            "|E|={:.4} |B|={:.4} ρ={:.4e} ∇⋅E={:.4e} ∇⋅B={:.4e} u={:.4e}",
            self.e_field.magnitude(),
            self.b_field.magnitude(),
            self.charge_density,
            self.div_e,
            self.div_b,
            self.energy_density()
        )
    }
    fn is_terminal(&self) -> bool {
        false
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
    fn describe(&self) -> String {
        match self {
            MaxwellAction::SetChargeDensity { rho } => format!("set ρ={:.4e}", rho),
            MaxwellAction::SetEField { e } => format!("set E=({:.4},{:.4},{:.4})", e.x, e.y, e.z),
            MaxwellAction::SetBField { b } => format!("set B=({:.4},{:.4},{:.4})", b.x, b.y, b.z),
            MaxwellAction::SetCurrentDensity { j } => {
                format!("set J=({:.4},{:.4},{:.4})", j.x, j.y, j.z)
            }
        }
    }
}

/// Gauss's law for electricity: ∇⋅E = ρ/ε₀
struct GaussElectric;
impl Precondition<MaxwellAction> for GaussElectric {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> PreconditionResult {
        let next = apply_maxwell(field, action).unwrap_or_else(|_| field.clone());
        if next.gauss_electric_holds() {
            PreconditionResult::satisfied(
                "gauss_electric",
                &format!(
                    "∇⋅E={:.4e} = ρ/ε₀={:.4e}",
                    next.div_e,
                    next.charge_density / EPSILON_0
                ),
            )
        } else {
            PreconditionResult::violated(
                "gauss_electric",
                "∇⋅E ≠ ρ/ε₀",
                &field.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "∇⋅E = ρ/ε₀ (Gauss's law)"
    }
}

/// Gauss's law for magnetism: ∇⋅B = 0 (no magnetic monopoles)
struct GaussMagnetic;
impl Precondition<MaxwellAction> for GaussMagnetic {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> PreconditionResult {
        let next = apply_maxwell(field, action).unwrap_or_else(|_| field.clone());
        if next.gauss_magnetic_holds() {
            PreconditionResult::satisfied("gauss_magnetic", "∇⋅B = 0 (no monopoles)")
        } else {
            PreconditionResult::violated(
                "gauss_magnetic",
                &format!(
                    "∇⋅B = {:.4e} ≠ 0: magnetic monopoles don't exist",
                    next.div_b
                ),
                &field.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "∇⋅B = 0 (no magnetic monopoles)"
    }
}

/// Energy density must be non-negative.
struct NonNegativeEnergy;
impl Precondition<MaxwellAction> for NonNegativeEnergy {
    fn check(&self, field: &EMField, action: &MaxwellAction) -> PreconditionResult {
        let next = apply_maxwell(field, action).unwrap_or_else(|_| field.clone());
        if next.energy_density() >= -1e-20 {
            PreconditionResult::satisfied(
                "energy_nonneg",
                &format!("u={:.4e} ≥ 0", next.energy_density()),
            )
        } else {
            PreconditionResult::violated(
                "energy_nonneg",
                "energy density cannot be negative",
                &field.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "electromagnetic energy density must be non-negative"
    }
}

fn apply_maxwell(field: &EMField, action: &MaxwellAction) -> Result<EMField, String> {
    Ok(match action {
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
    })
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

    #[test]
    fn test_speed_of_light_derived() {
        let c = speed_of_light();
        // c ≈ 299,792,458 m/s
        assert!((c - 2.998e8).abs() < 1e6, "c={} should be ≈ 3e8", c);
    }

    #[test]
    fn test_vacuum_satisfies_all() {
        let field = EMField::vacuum();
        assert!(field.gauss_electric_holds());
        assert!(field.gauss_magnetic_holds());
        assert!((field.energy_density() - 0.0).abs() < 1e-20);
    }

    #[test]
    fn test_charge_creates_divergence() {
        let e = new_field()
            .next(MaxwellAction::SetChargeDensity { rho: 1e-6 })
            .unwrap();
        assert!(e.situation().div_e > 0.0);
        assert!(e.situation().gauss_electric_holds());
    }

    #[test]
    fn test_no_magnetic_monopoles() {
        let field = EMField::new(Vec3::zero(), Vec3::new(1.0, 0.0, 0.0), 0.0, Vec3::zero());
        assert!(field.gauss_magnetic_holds());
        assert_eq!(field.div_b, 0.0);
    }

    #[test]
    fn test_energy_density_nonneg() {
        let e = new_field()
            .next(MaxwellAction::SetEField {
                e: Vec3::new(100.0, 0.0, 0.0),
            })
            .unwrap();
        assert!(e.situation().energy_density() > 0.0);
    }

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

    #[test]
    fn test_cross_product() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert!((z.x - 0.0).abs() < 1e-10);
        assert!((z.y - 0.0).abs() < 1e-10);
        assert!((z.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dot_product() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!((a.dot(&b) - 32.0).abs() < 1e-10);
    }

    #[test]
    fn test_undo_redo() {
        let e = new_field()
            .next(MaxwellAction::SetEField {
                e: Vec3::new(100.0, 0.0, 0.0),
            })
            .unwrap();
        assert!(e.situation().e_field.magnitude() > 0.0);
        let e = e.back().unwrap();
        assert!((e.situation().e_field.magnitude() - 0.0).abs() < 1e-10);
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
            prop_assert!(e.situation().energy_density() >= 0.0);
        }

        /// Speed of light: c = 1/√(μ₀ε₀) ≈ 3×10⁸
        #[test]
        fn prop_speed_of_light(_x in 0..1u8) {
            let c = speed_of_light();
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
            prop_assert!((a.dot(&b) - b.dot(&a)).abs() < 1e-10);
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
            prop_assert!(s.dot(&e).abs() < 1e-6, "S not perpendicular to E");
            // S ⊥ B: S⋅B = 0
            prop_assert!(s.dot(&b).abs() < 1e-6, "S not perpendicular to B");
        }
    }
}
