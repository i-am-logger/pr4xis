#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::calc::Calculator;
use super::op::{BinaryOp, UnaryOp};
use super::value::{AngleMode, Value};
use crate::formal::math::ontology::MathDomain;
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Calculator {
    fn describe(&self) -> String {
        format!(
            "display={} memory={} mode={:?}",
            self.display, self.memory, self.angle_mode
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcAction {
    Enter(Value),
    Unary(UnaryOp),
    Binary(BinaryOp, Value),
    Clear,
    AllClear,
    StoreMemory,
    RecallMemory,
    AddToMemory,
    ClearMemory,
    SetAngleMode(AngleMode),
}

impl Action for CalcAction {
    type Sit = Calculator;

    fn describe(&self) -> String {
        match self {
            CalcAction::Enter(v) => format!("enter {}", v),
            CalcAction::Unary(op) => format!("{:?}", op),
            CalcAction::Binary(op, v) => format!("{:?} {}", op, v),
            CalcAction::Clear => "clear".into(),
            CalcAction::AllClear => "all clear".into(),
            CalcAction::StoreMemory => "M store".into(),
            CalcAction::RecallMemory => "M recall".into(),
            CalcAction::AddToMemory => "M+".into(),
            CalcAction::ClearMemory => "MC".into(),
            CalcAction::SetAngleMode(m) => format!("angle mode {:?}", m),
        }
    }
}

/// Domain enforcement: checks if the operation is valid before applying.
pub struct DomainCheck;

impl Precondition<CalcAction> for DomainCheck {
    fn check(&self, calc: &Calculator, action: &CalcAction) -> PreconditionResult {
        match action {
            CalcAction::Unary(op) => match op.apply(&calc.display, calc.angle_mode) {
                Ok(_) => PreconditionResult::satisfied(
                    "domain_check",
                    &format!("{:?}({}) is valid", op, calc.display),
                ),
                Err(e) => PreconditionResult::violated(
                    "domain_check",
                    &format!("{}", e),
                    &calc.describe(),
                    &action.describe(),
                ),
            },
            CalcAction::Binary(op, rhs) => match op.apply(&calc.display, rhs) {
                Ok(_) => PreconditionResult::satisfied(
                    "domain_check",
                    &format!("{} {:?} {} is valid", calc.display, op, rhs),
                ),
                Err(e) => PreconditionResult::violated(
                    "domain_check",
                    &format!("{}", e),
                    &calc.describe(),
                    &action.describe(),
                ),
            },
            _ => PreconditionResult::satisfied("domain_check", "no domain constraints"),
        }
    }

    fn describe(&self) -> &str {
        "mathematical domain must be valid (no division by zero, sqrt of negative, etc.)"
    }
}

fn apply_calc(calc: &Calculator, action: &CalcAction) -> Result<Calculator, String> {
    let mut next = calc.clone();
    match action {
        CalcAction::Enter(v) => next.enter(v.clone()),
        CalcAction::Unary(op) => {
            let _ = next.unary(*op);
        }
        CalcAction::Binary(op, v) => {
            let _ = next.binary(*op, v.clone());
        }
        CalcAction::Clear => next.clear(),
        CalcAction::AllClear => next.all_clear(),
        CalcAction::StoreMemory => next.memory_op(super::calc::MemoryOp::Store),
        CalcAction::RecallMemory => next.memory_op(super::calc::MemoryOp::Recall),
        CalcAction::AddToMemory => next.memory_op(super::calc::MemoryOp::Add),
        CalcAction::ClearMemory => next.memory_op(super::calc::MemoryOp::Clear),
        CalcAction::SetAngleMode(m) => next.set_angle_mode(*m),
    }
    Ok(next)
}

/// Ontology-driven domain check: classifies the current value into the number
/// hierarchy (N ⊂ Z ⊂ Q ⊂ R ⊂ C) and enforces that operations stay within
/// supported domains.
pub struct NumberDomainCheck;

impl NumberDomainCheck {
    /// Classify a calculator value into the smallest containing MathDomain.
    fn classify(val: &Value) -> MathDomain {
        match val {
            Value::Rational(n, d) => {
                if *d == 1 {
                    if *n >= 0 {
                        MathDomain::NaturalNumbers
                    } else {
                        MathDomain::Integers
                    }
                } else {
                    MathDomain::Rationals
                }
            }
            Value::Float(f) => {
                if f.fract() == 0.0 {
                    if *f >= 0.0 {
                        MathDomain::NaturalNumbers
                    } else {
                        MathDomain::Integers
                    }
                } else {
                    MathDomain::Reals
                }
            }
        }
    }

    /// Does this operation require at least a certain domain?
    fn required_domain(action: &CalcAction, val: &Value) -> Option<(MathDomain, &'static str)> {
        match action {
            CalcAction::Unary(UnaryOp::Factorial) => Some((
                MathDomain::NaturalNumbers,
                "factorial requires natural numbers",
            )),
            CalcAction::Unary(UnaryOp::Sqrt) if val.is_negative() => Some((
                MathDomain::Complex,
                "sqrt of negative requires complex numbers",
            )),
            CalcAction::Unary(UnaryOp::Ln | UnaryOp::Log10 | UnaryOp::Log2)
                if !val.to_f64().is_sign_positive() || val.is_zero() =>
            {
                Some((
                    MathDomain::Complex,
                    "log of non-positive requires complex numbers",
                ))
            }
            CalcAction::Binary(BinaryOp::Divide, rhs) if rhs.is_zero() => {
                // Division by zero isn't in any domain
                None // DomainCheck handles this
            }
            CalcAction::Binary(BinaryOp::Divide, _) => Some((
                MathDomain::Rationals,
                "division requires rationals or above",
            )),
            _ => None,
        }
    }
}

impl Precondition<CalcAction> for NumberDomainCheck {
    fn check(&self, calc: &Calculator, action: &CalcAction) -> PreconditionResult {
        let current_domain = Self::classify(&calc.display);

        if let Some((required, reason)) = Self::required_domain(action, &calc.display) {
            let current_order = domain_order(current_domain);
            let required_order = domain_order(required);

            if current_order > required_order {
                return PreconditionResult::violated(
                    "number_domain",
                    &format!(
                        "{} — value is in {:?} but operation needs {:?}",
                        reason, current_domain, required
                    ),
                    &calc.describe(),
                    &action.describe(),
                );
            }
        }

        PreconditionResult::satisfied(
            "number_domain",
            &format!("value in {:?}, operation valid", current_domain),
        )
    }

    fn describe(&self) -> &str {
        "operations must be valid within the number domain hierarchy (N ⊂ Z ⊂ Q ⊂ R ⊂ C)"
    }
}

fn domain_order(d: MathDomain) -> u8 {
    match d {
        MathDomain::NaturalNumbers => 0,
        MathDomain::Integers => 1,
        MathDomain::Rationals => 2,
        MathDomain::Reals => 3,
        MathDomain::Complex => 4,
    }
}

pub type CalcEngine = Engine<CalcAction>;

pub fn new_calculator() -> CalcEngine {
    Engine::new(
        Calculator::new(),
        vec![Box::new(DomainCheck), Box::new(NumberDomainCheck)],
        apply_calc,
    )
}
