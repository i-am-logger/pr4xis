/// Pythagorean theorem as an ontology:
/// - Situation: a right triangle (a, b, c)
/// - Axiom: a² + b² = c² must hold at all times
/// - Actions: scale, set leg (hypotenuse is always derived)
/// - Enforcement: the theorem is a precondition on every transformation
use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

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

impl Situation for Triangle {
    fn describe(&self) -> String {
        format!(
            "a={:.4} b={:.4} c={:.4} (a²+b²={:.4} c²={:.4})",
            self.a,
            self.b,
            self.c,
            self.a * self.a + self.b * self.b,
            self.c * self.c
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriangleAction {
    Scale { factor: f64 },
    SetLegA { value: f64 },
    SetLegB { value: f64 },
}

impl Action for TriangleAction {
    type Sit = Triangle;
    fn describe(&self) -> String {
        match self {
            TriangleAction::Scale { factor } => format!("scale by {}", factor),
            TriangleAction::SetLegA { value } => format!("set a={}", value),
            TriangleAction::SetLegB { value } => format!("set b={}", value),
        }
    }
}

struct PositiveSides;
impl Precondition<TriangleAction> for PositiveSides {
    fn check(&self, tri: &Triangle, action: &TriangleAction) -> PreconditionResult {
        let valid = match action {
            TriangleAction::Scale { factor } => *factor > 0.0,
            TriangleAction::SetLegA { value } => *value > 0.0,
            TriangleAction::SetLegB { value } => *value > 0.0,
        };
        if valid {
            PreconditionResult::satisfied("positive_sides", "all sides positive")
        } else {
            PreconditionResult::violated(
                "positive_sides",
                "sides must be positive",
                &tri.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "all sides must be positive"
    }
}

struct PythagoreanTheorem;
impl Precondition<TriangleAction> for PythagoreanTheorem {
    fn check(&self, tri: &Triangle, action: &TriangleAction) -> PreconditionResult {
        let next = apply(tri, action);
        if next.theorem_holds() {
            PreconditionResult::satisfied(
                "pythagorean_theorem",
                &format!(
                    "a²+b²={:.6} = c²={:.6}",
                    next.a * next.a + next.b * next.b,
                    next.c * next.c
                ),
            )
        } else {
            PreconditionResult::violated(
                "pythagorean_theorem",
                &format!(
                    "a²+b²={:.6} ≠ c²={:.6}",
                    next.a * next.a + next.b * next.b,
                    next.c * next.c
                ),
                &tri.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "a² + b² = c² must hold"
    }
}

fn apply(tri: &Triangle, action: &TriangleAction) -> Triangle {
    match action {
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
    }
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

    #[test]
    fn test_3_4_5() {
        let e = new_triangle(3.0, 4.0).unwrap();
        assert!((e.situation().c - 5.0).abs() < 1e-10);
        assert!(e.situation().theorem_holds());
    }

    #[test]
    fn test_scale_preserves_theorem() {
        let e = new_triangle(3.0, 4.0)
            .unwrap()
            .next(TriangleAction::Scale { factor: 2.0 })
            .unwrap();
        assert!(e.situation().theorem_holds());
        assert!((e.situation().a - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_set_leg_recomputes() {
        let e = new_triangle(3.0, 4.0)
            .unwrap()
            .next(TriangleAction::SetLegA { value: 5.0 })
            .unwrap();
        assert!(e.situation().theorem_holds());
    }

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
}
