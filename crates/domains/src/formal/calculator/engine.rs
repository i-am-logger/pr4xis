#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::calc::Calculator;
use super::op::{BinaryOp, UnaryOp};
use super::value::{AngleMode, Value};
use crate::formal::math::ontology::NumberConcept;
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

impl Situation for Calculator {}

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
}

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

/// Domain enforcement: checks if the operation is valid before applying.
pub struct DomainCheck;

impl Precondition<CalcAction> for DomainCheck {
    fn check(&self, calc: &Calculator, action: &CalcAction) -> Verdict {
        let meta = axiom_meta(
            "domain_check",
            "mathematical domain must be valid (no division by zero, sqrt of negative, etc.)",
            "Knuth (1997) TAOCP Vol. 2 §4.2.2; IEEE 754-2019 §7 invalid operation",
        );
        match action {
            CalcAction::Unary(op) => match op.apply(&calc.display, calc.angle_mode) {
                Ok(_) => Ok(Box::new(SimpleProof::new(meta))),
                Err(_) => Err(Box::new(SimpleCounterexample::new(meta))),
            },
            CalcAction::Binary(op, rhs) => match op.apply(&calc.display, rhs) {
                Ok(_) => Ok(Box::new(SimpleProof::new(meta))),
                Err(_) => Err(Box::new(SimpleCounterexample::new(meta))),
            },
            _ => Ok(Box::new(SimpleProof::new(meta))),
        }
    }
}

fn apply_calc(
    calc: &Calculator,
    action: &CalcAction,
) -> Result<Calculator, Box<dyn Counterexample>> {
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
    /// Classify a calculator value into the smallest containing NumberConcept.
    fn classify(val: &Value) -> NumberConcept {
        match val {
            Value::Rational(n, d) => {
                if *d == 1 {
                    if *n >= 0 {
                        NumberConcept::NaturalNumbers
                    } else {
                        NumberConcept::Integers
                    }
                } else {
                    NumberConcept::Rationals
                }
            }
            Value::Float(f) => {
                if f.fract() == 0.0 {
                    if *f >= 0.0 {
                        NumberConcept::NaturalNumbers
                    } else {
                        NumberConcept::Integers
                    }
                } else {
                    NumberConcept::Reals
                }
            }
        }
    }

    /// Does this operation require at least a certain domain?
    fn required_domain(action: &CalcAction, val: &Value) -> Option<(NumberConcept, &'static str)> {
        match action {
            CalcAction::Unary(UnaryOp::Factorial) => Some((
                NumberConcept::NaturalNumbers,
                "factorial requires natural numbers",
            )),
            CalcAction::Unary(UnaryOp::Sqrt) if val.is_negative() => Some((
                NumberConcept::Complex,
                "sqrt of negative requires complex numbers",
            )),
            CalcAction::Unary(UnaryOp::Ln | UnaryOp::Log10 | UnaryOp::Log2)
                if !val.to_f64().is_sign_positive() || val.is_zero() =>
            {
                Some((
                    NumberConcept::Complex,
                    "log of non-positive requires complex numbers",
                ))
            }
            CalcAction::Binary(BinaryOp::Divide, rhs) if rhs.is_zero() => {
                // Division by zero isn't in any domain
                None // DomainCheck handles this
            }
            CalcAction::Binary(BinaryOp::Divide, _) => Some((
                NumberConcept::Rationals,
                "division requires rationals or above",
            )),
            _ => None,
        }
    }
}

impl Precondition<CalcAction> for NumberDomainCheck {
    fn check(&self, calc: &Calculator, action: &CalcAction) -> Verdict {
        let meta = axiom_meta(
            "number_domain",
            "operations must be valid within the number domain hierarchy (N \u{2282} Z \u{2282} Q \u{2282} R \u{2282} C)",
            "Bourbaki (1939) Theory of Sets; Mac Lane & Birkhoff (1967) Algebra",
        );
        let current_domain = Self::classify(&calc.display);

        if let Some((required, _reason)) = Self::required_domain(action, &calc.display) {
            let current_order = domain_order(current_domain);
            let required_order = domain_order(required);

            if current_order > required_order {
                return Err(Box::new(SimpleCounterexample::new(meta)));
            }
        }

        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn domain_order(d: NumberConcept) -> u8 {
    match d {
        NumberConcept::NaturalNumbers => 0,
        NumberConcept::Integers => 1,
        NumberConcept::Rationals => 2,
        NumberConcept::Reals => 3,
        NumberConcept::Complex => 4,
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
