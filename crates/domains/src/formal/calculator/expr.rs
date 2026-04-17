use super::op::{BinaryOp, UnaryOp};
use super::value::{AngleMode, CalcError, Value};

/// A mathematical expression tree.
/// Expressions can be evaluated and simplified while preserving equivalence.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value.
    Lit(Value),
    /// Unary operation on an expression.
    Unary(UnaryOp, Box<Expr>),
    /// Binary operation on two expressions.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn lit(v: Value) -> Self {
        Expr::Lit(v)
    }
    pub fn int(n: i64) -> Self {
        Expr::Lit(Value::int(n))
    }

    pub fn unary(op: UnaryOp, expr: Expr) -> Self {
        Expr::Unary(op, Box::new(expr))
    }

    pub fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Self {
        Expr::Binary(op, Box::new(lhs), Box::new(rhs))
    }

    /// Evaluate the expression to a value.
    pub fn eval(&self, angle_mode: AngleMode) -> Result<Value, CalcError> {
        match self {
            Expr::Lit(v) => Ok(v.clone()),
            Expr::Unary(op, expr) => {
                let val = expr.eval(angle_mode)?;
                op.apply(&val, angle_mode)
            }
            Expr::Binary(op, lhs, rhs) => {
                let l = lhs.eval(angle_mode)?;
                let r = rhs.eval(angle_mode)?;
                op.apply(&l, &r)
            }
        }
    }

    /// Simplify the expression while preserving equivalence.
    /// Applies algebraic identities:
    /// - x + 0 → x, 0 + x → x
    /// - x * 1 → x, 1 * x → x
    /// - x * 0 → 0, 0 * x → 0
    /// - x - 0 → x
    /// - x / 1 → x
    /// - x ^ 0 → 1, x ^ 1 → x
    /// - --x → x (double negation)
    /// - Constant folding: 2 + 3 → 5
    pub fn simplify(&self) -> Expr {
        match self {
            Expr::Lit(_) => self.clone(),

            Expr::Unary(UnaryOp::Negate, inner) => {
                let inner = inner.simplify();
                // Double negation: --x → x
                if let Expr::Unary(UnaryOp::Negate, x) = &inner {
                    return x.simplify();
                }
                // Constant folding
                if let Expr::Lit(v) = &inner {
                    return Expr::Lit(v.negate());
                }
                Expr::unary(UnaryOp::Negate, inner)
            }

            Expr::Unary(op, inner) => {
                let inner = inner.simplify();
                // Constant folding
                if let Expr::Lit(v) = &inner
                    && let Ok(result) = op.apply(v, AngleMode::Radians)
                {
                    return Expr::Lit(result);
                }
                Expr::unary(*op, inner)
            }

            Expr::Binary(op, lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();

                // Identity rules FIRST (preserve exact types)
                match op {
                    BinaryOp::Add => {
                        if is_zero(&rhs) {
                            return lhs;
                        }
                        if is_zero(&lhs) {
                            return rhs;
                        }
                    }
                    BinaryOp::Subtract if is_zero(&rhs) => {
                        return lhs;
                    }
                    BinaryOp::Multiply => {
                        if is_one(&rhs) {
                            return lhs;
                        }
                        if is_one(&lhs) {
                            return rhs;
                        }
                        if is_zero(&rhs) {
                            return Expr::int(0);
                        }
                        if is_zero(&lhs) {
                            return Expr::int(0);
                        }
                    }
                    BinaryOp::Divide if is_one(&rhs) => {
                        return lhs;
                    }
                    BinaryOp::Power => {
                        if is_zero(&rhs) {
                            return Expr::int(1);
                        }
                        if is_one(&rhs) {
                            return lhs;
                        }
                    }
                    _ => {}
                }

                // Constant folding: both sides are literals
                if let (Expr::Lit(l), Expr::Lit(r)) = (&lhs, &rhs)
                    && let Ok(result) = op.apply(l, r)
                {
                    return Expr::Lit(result);
                }

                Expr::binary(*op, lhs, rhs)
            }
        }
    }
}

fn is_zero(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(v) if v.is_zero())
}

fn is_one(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(v) if v.is_one())
}
