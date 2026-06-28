#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Pythagorean theorem as an ontology:
/// - Situation: a right triangle (a, b, c)
/// - Axiom: a² + b² = c² must hold at all times
/// - Actions: scale, set leg (hypotenuse is always derived)
/// - Enforcement: the theorem is a precondition on every transformation
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Triangle {
    pub fn from_legs(a: f64, b: f64) -> Result<Self, &'static str> {
        if a <= 0.0 || b <= 0.0 {
            return Err("sides must be positive");
        }
        Ok(Self {
            a,
            b,
            c: (a * a + b * b).sqrt(),
        })
    }

    pub fn theorem_holds(&self) -> bool {
        let lhs = self.a * self.a + self.b * self.b;
        let rhs = self.c * self.c;
        (lhs - rhs).abs() / lhs.max(rhs).max(1.0) < 1e-10
    }

    pub fn is_triple(&self) -> bool {
        let (a, b, c) = (self.a.round(), self.b.round(), self.c.round());
        (self.a - a).abs() < 1e-10 && (self.b - b).abs() < 1e-10 && (self.c - c).abs() < 1e-10
    }
}

impl Situation for Triangle {}

#[derive(Debug, Clone, PartialEq)]
pub enum TriangleAction {
    Scale { factor: f64 },
    SetLegA { value: f64 },
    SetLegB { value: f64 },
}

impl Action for TriangleAction {
    type Sit = Triangle;
}

fn pyth_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Euclid (c. 300 BCE) Elements, Book I Proposition 47; Heath (1908) The Thirteen Books of Euclid's Elements",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

struct PositiveSides;
impl Precondition<TriangleAction> for PositiveSides {
    fn check(&self, _tri: &Triangle, action: &TriangleAction) -> Verdict {
        let meta = pyth_meta("PositiveSides", "all sides must be positive");
        let valid = match action {
            TriangleAction::Scale { factor } => *factor > 0.0,
            TriangleAction::SetLegA { value } => *value > 0.0,
            TriangleAction::SetLegB { value } => *value > 0.0,
        };
        if valid {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

struct PythagoreanTheorem;
impl Precondition<TriangleAction> for PythagoreanTheorem {
    fn check(&self, tri: &Triangle, action: &TriangleAction) -> Verdict {
        let meta = pyth_meta("PythagoreanTheorem", "a² + b² = c² must hold");
        let next = apply_inner(tri, action).unwrap_or_else(|_| tri.clone());
        if next.theorem_holds() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_inner(tri: &Triangle, action: &TriangleAction) -> Result<Triangle, &'static str> {
    Ok(match action {
        TriangleAction::Scale { factor } => Triangle {
            a: tri.a * factor,
            b: tri.b * factor,
            c: tri.c * factor,
        },
        TriangleAction::SetLegA { value } => {
            Triangle::from_legs(*value, tri.b).unwrap_or_else(|_| tri.clone())
        }
        TriangleAction::SetLegB { value } => {
            Triangle::from_legs(tri.a, *value).unwrap_or_else(|_| tri.clone())
        }
    })
}

fn apply(tri: &Triangle, action: &TriangleAction) -> Result<Triangle, Box<dyn Counterexample>> {
    apply_inner(tri, action).map_err(|_| {
        let meta = pyth_meta(
            "ApplyFailed",
            "triangle transformation could not be applied",
        );
        Box::new(SimpleCounterexample::new(meta)) as Box<dyn Counterexample>
    })
}

pub fn new_triangle(a: f64, b: f64) -> Result<Engine<TriangleAction>, &'static str> {
    let tri = Triangle::from_legs(a, b)?;
    Ok(Engine::new(
        tri,
        vec![Box::new(PositiveSides), Box::new(PythagoreanTheorem)],
        apply,
    ))
}

pub fn triples(max_c: u64) -> Vec<(u64, u64, u64)> {
    let mut result = Vec::new();
    for a in 1..max_c {
        for b in a..max_c {
            let c_sq = a * a + b * b;
            let c = (c_sq as f64).sqrt() as u64;
            if c > max_c {
                break;
            }
            if c * c == c_sq {
                result.push((a, b, c));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_3_4_5() {
        let e = new_triangle(3.0, 4.0).unwrap();
        assert!((e.situation().c - 5.0).abs() < 1e-10);
        assert!(e.situation().theorem_holds());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_scale_preserves_theorem() {
        let e = new_triangle(3.0, 4.0)
            .unwrap()
            .next(TriangleAction::Scale { factor: 2.0 })
            .unwrap();
        assert!(e.situation().theorem_holds());
        assert!((e.situation().a - 6.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_set_leg_recomputes() {
        let e = new_triangle(3.0, 4.0)
            .unwrap()
            .next(TriangleAction::SetLegA { value: 5.0 })
            .unwrap();
        assert!(e.situation().theorem_holds());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn test_negative_blocked() {
        let e = new_triangle(3.0, 4.0).unwrap();
        assert!(e.next(TriangleAction::Scale { factor: -1.0 }).is_err());
        assert!(
            new_triangle(3.0, 4.0)
                .unwrap()
                .next(TriangleAction::SetLegA { value: 0.0 })
                .is_err()
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn test_undo_redo() {
        let e = new_triangle(3.0, 4.0)
            .unwrap()
            .next(TriangleAction::Scale { factor: 2.0 })
            .unwrap();
        let e = e.back().unwrap();
        assert!((e.situation().a - 3.0).abs() < 1e-10);
        let e = e.forward().unwrap();
        assert!((e.situation().a - 6.0).abs() < 1e-10);
    }

    proptest! {
        #[test]
        fn prop_theorem_always_holds(a in 0.1..1000.0f64, b in 0.1..1000.0f64) {
            prop_assert!(new_triangle(a, b).unwrap().situation().theorem_holds());
        }

        #[test]
        fn prop_theorem_after_scale(a in 0.1..100.0f64, b in 0.1..100.0f64, f in 0.01..100.0f64) {
            let e = new_triangle(a, b).unwrap().next(TriangleAction::Scale { factor: f }).unwrap();
            prop_assert!(e.situation().theorem_holds());
        }

        #[test]
        fn prop_hypotenuse_longest(a in 0.1..1000.0f64, b in 0.1..1000.0f64) {
            let t = new_triangle(a, b).unwrap().situation().clone();
            prop_assert!(t.c > t.a);
            prop_assert!(t.c > t.b);
        }

        #[test]
        fn prop_triangle_inequality(a in 0.1..1000.0f64, b in 0.1..1000.0f64) {
            let t = new_triangle(a, b).unwrap().situation().clone();
            prop_assert!(t.a + t.b > t.c);
        }

        #[test]
        fn prop_negative_blocked(a in 0.1..100.0f64, b in 0.1..100.0f64, neg in -100.0..-0.01f64) {
            let result = new_triangle(a, b).unwrap().next(TriangleAction::SetLegA { value: neg });
            prop_assert!(result.is_err());
        }
    }

    pr4xis::register_praxis_value!(prop_theorem_always_holds, Verifiable);
    pr4xis::register_praxis_value!(prop_theorem_after_scale, Verifiable);
    pr4xis::register_praxis_value!(prop_hypotenuse_longest, Verifiable);
    pr4xis::register_praxis_value!(prop_triangle_inequality, Verifiable);
    pr4xis::register_praxis_value!(prop_negative_blocked, Honest);
}
