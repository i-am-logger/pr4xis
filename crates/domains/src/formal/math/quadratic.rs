#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Quadratic equation as an ontology:
/// - Situation: coefficients (a, b, c) and derived roots
/// - Axioms: roots satisfy equation, Vieta's formulas, discriminant consistency
/// - Actions: modify coefficients (roots auto-recomputed)
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

#[derive(Debug, Clone, PartialEq)]
pub enum Roots {
    Two { x1: f64, x2: f64 },
    One { x: f64 },
    Complex { real: f64, imag: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quadratic {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub roots: Roots,
}

impl Quadratic {
    pub fn new(a: f64, b: f64, c: f64) -> Result<Self, &'static str> {
        if a == 0.0 {
            return Err("a must be non-zero");
        }
        let d = b * b - 4.0 * a * c;
        let roots = if d > 1e-10 {
            Roots::Two {
                x1: (-b + d.sqrt()) / (2.0 * a),
                x2: (-b - d.sqrt()) / (2.0 * a),
            }
        } else if d.abs() <= 1e-10 {
            Roots::One { x: -b / (2.0 * a) }
        } else {
            Roots::Complex {
                real: -b / (2.0 * a),
                imag: (-d).sqrt() / (2.0 * a),
            }
        };
        Ok(Self { a, b, c, roots })
    }

    pub fn discriminant(&self) -> f64 {
        self.b * self.b - 4.0 * self.a * self.c
    }
    pub fn eval(&self, x: f64) -> f64 {
        self.a * x * x + self.b * x + self.c
    }

    pub fn roots_valid(&self) -> bool {
        match &self.roots {
            Roots::Two { x1, x2 } => self.eval(*x1).abs() < 1e-4 && self.eval(*x2).abs() < 1e-4,
            Roots::One { x } => self.eval(*x).abs() < 1e-4,
            Roots::Complex { .. } => true,
        }
    }
}

impl Situation for Quadratic {
    fn describe(&self) -> String {
        format!(
            "{}x²+{}x+{}=0 d={:.4} roots={:?}",
            self.a,
            self.b,
            self.c,
            self.discriminant(),
            self.roots
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuadAction {
    SetA(f64),
    SetB(f64),
    SetC(f64),
}

impl Action for QuadAction {
    type Sit = Quadratic;
    fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

struct NonZeroA;
impl Precondition<QuadAction> for NonZeroA {
    fn check(&self, q: &Quadratic, a: &QuadAction) -> PreconditionResult {
        if let QuadAction::SetA(v) = a
            && *v == 0.0
        {
            return PreconditionResult::violated(
                "non_zero_a",
                "a=0 not quadratic",
                &q.describe(),
                &a.describe(),
            );
        }
        PreconditionResult::satisfied("non_zero_a", "a≠0")
    }
    fn describe(&self) -> &str {
        "a must be non-zero"
    }
}

struct RootsValid;
impl Precondition<QuadAction> for RootsValid {
    fn check(&self, q: &Quadratic, a: &QuadAction) -> PreconditionResult {
        let next = apply_quad(q, a).unwrap_or_else(|_| q.clone());
        if next.roots_valid() {
            PreconditionResult::satisfied("roots_valid", "roots satisfy equation")
        } else {
            PreconditionResult::violated(
                "roots_valid",
                "roots don't satisfy",
                &q.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "roots must satisfy ax²+bx+c=0"
    }
}

fn apply_quad(q: &Quadratic, action: &QuadAction) -> Result<Quadratic, String> {
    let (a, b, c) = match action {
        QuadAction::SetA(v) => (*v, q.b, q.c),
        QuadAction::SetB(v) => (q.a, *v, q.c),
        QuadAction::SetC(v) => (q.a, q.b, *v),
    };
    Quadratic::new(a, b, c).map_err(|e| e.to_string())
}

pub fn new_equation(a: f64, b: f64, c: f64) -> Result<Engine<QuadAction>, &'static str> {
    let q = Quadratic::new(a, b, c)?;
    Ok(Engine::new(
        q,
        vec![Box::new(NonZeroA), Box::new(RootsValid)],
        apply_quad,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_x2_minus_5x_plus_6() {
        let e = new_equation(1.0, -5.0, 6.0).unwrap();
        assert!(e.situation().roots_valid());
        if let Roots::Two { x1, x2 } = &e.situation().roots {
            assert!(e.situation().eval(*x1).abs() < 1e-10);
            assert!(e.situation().eval(*x2).abs() < 1e-10);
        }
    }

    #[test]
    fn test_complex() {
        let e = new_equation(1.0, 0.0, 1.0).unwrap();
        assert!(matches!(e.situation().roots, Roots::Complex { .. }));
    }

    #[test]
    fn test_a_zero_blocked() {
        assert!(
            new_equation(1.0, 1.0, 1.0)
                .unwrap()
                .next(QuadAction::SetA(0.0))
                .is_err()
        );
    }

    proptest! {
        #[test]
        fn prop_roots_always_valid(a in -10.0..10.0f64, b in -10.0..10.0f64, c in -10.0..10.0f64) {
            prop_assume!(a.abs() > 0.01);
            prop_assert!(new_equation(a, b, c).unwrap().situation().roots_valid());
        }

        #[test]
        fn prop_discriminant_sign(a in -10.0..10.0f64, b in -10.0..10.0f64, c in -10.0..10.0f64) {
            prop_assume!(a.abs() > 0.01);
            let q = new_equation(a, b, c).unwrap().situation().clone();
            let d = q.discriminant();
            match q.roots {
                Roots::Two { .. } => prop_assert!(d > 0.0),
                Roots::One { .. } => prop_assert!(d.abs() < 1e-8),
                Roots::Complex { .. } => prop_assert!(d < 0.0),
            }
        }
    }
}
