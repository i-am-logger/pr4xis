//! Realized qualitative-process mechanics (Forbus 1984 *Qualitative Process
//! Theory*, Artificial Intelligence 24(1-3), §2.2-2.3): whether a process is
//! active (every precondition satisfied) and what qualitative derivative
//! sign it imposes on an influenced quantity.

use alloc::{string::String, vec::Vec};

/// The qualitative sign of a Quantity's time-derivative (Forbus 1984 §2.1)
/// — the central QP-theory abstraction: a magnitude's change is tracked by
/// SIGN against a QuantitySpace, never a numeric rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivativeSign {
    Increasing,
    Steady,
    Decreasing,
}

/// The sign of a Process's direct influence on a Quantity — I+ or I-
/// (Forbus 1984 §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfluenceSign {
    Positive,
    Negative,
}

/// One precondition of a process instance — satisfied or not, in THIS run.
#[derive(Debug, Clone)]
pub struct PreconditionInstance {
    pub description: String,
    pub satisfied: bool,
}

/// One direct influence a process exerts on a named quantity while active.
#[derive(Debug, Clone)]
pub struct InfluenceInstance {
    pub quantity: String,
    pub sign: InfluenceSign,
}

/// A concrete instantiation of a Process (Forbus 1984 §2.2): named
/// preconditions plus the influences it exerts on quantities WHEN active.
#[derive(Debug, Clone)]
pub struct ProcessInstance {
    pub name: String,
    pub preconditions: Vec<PreconditionInstance>,
    pub influences: Vec<InfluenceInstance>,
}

/// Is `p` active? Forbus (1984) §2.2: a process is active exactly when
/// every one of its preconditions is satisfied (individual existence,
/// quantity conditions, and any relational preconditions collapse to one
/// boolean per precondition at this level of abstraction — a process with
/// zero preconditions is vacuously active).
pub fn is_active(p: &ProcessInstance) -> bool {
    p.preconditions.iter().all(|c| c.satisfied)
}

/// The predicted qualitative derivative sign `p` imposes on `quantity`, or
/// `None` if `p` does not influence it or is not active. Forbus (1984)
/// §2.3: an active process's direct influence sets the sign of the
/// influenced quantity's derivative — I+ increases it, I- decreases it.
///
/// Scoped to a SINGLE unopposed influence per quantity — Forbus's general
/// influence-RESOLUTION step (summing multiple simultaneous processes'
/// influences on the same quantity, §3) is out of scope here; this
/// function answers the direct, unopposed case the 1.5a test exercises.
pub fn predicted_derivative(p: &ProcessInstance, quantity: &str) -> Option<DerivativeSign> {
    if !is_active(p) {
        return None;
    }
    p.influences
        .iter()
        .find(|i| i.quantity == quantity)
        .map(|i| match i.sign {
            InfluenceSign::Positive => DerivativeSign::Increasing,
            InfluenceSign::Negative => DerivativeSign::Decreasing,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heating_process(precondition_met: bool) -> ProcessInstance {
        ProcessInstance {
            name: String::from("Heating"),
            preconditions: alloc::vec![PreconditionInstance {
                description: String::from("heat source in contact with liquid"),
                satisfied: precondition_met,
            }],
            influences: alloc::vec![InfluenceInstance {
                quantity: String::from("temperature"),
                sign: InfluenceSign::Positive,
            }],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn active_when_all_preconditions_satisfied() {
        assert!(is_active(&heating_process(true)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn inactive_when_a_precondition_fails() {
        assert!(!is_active(&heating_process(false)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn active_process_predicts_its_influenced_quantitys_sign() {
        assert_eq!(
            predicted_derivative(&heating_process(true), "temperature"),
            Some(DerivativeSign::Increasing)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn inactive_process_predicts_nothing() {
        assert_eq!(
            predicted_derivative(&heating_process(false), "temperature"),
            None
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_uninfluenced_quantity_predicts_nothing() {
        assert_eq!(
            predicted_derivative(&heating_process(true), "pressure"),
            None
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn negative_influence_predicts_decreasing() {
        let p = ProcessInstance {
            name: String::from("Cooling"),
            preconditions: Vec::new(),
            influences: alloc::vec![InfluenceInstance {
                quantity: String::from("temperature"),
                sign: InfluenceSign::Negative,
            }],
        };
        assert_eq!(
            predicted_derivative(&p, "temperature"),
            Some(DerivativeSign::Decreasing)
        );
    }
}
