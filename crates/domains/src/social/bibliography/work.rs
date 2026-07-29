//! Realized FRBR Group-1 mechanics (IFLA FRBR 1998 §3.2): whether a Work is
//! realized (has at least one Expression) and whether an Expression is
//! embodied (has at least one Manifestation), plus the authorOf/appearsIn
//! accessors.

use alloc::{string::String, vec::Vec};

/// A concrete Work record — IFLA FRBR (1998) §3.2.1.
#[derive(Debug, Clone)]
pub struct WorkRecord {
    pub title: String,
    pub author: String,
    pub expressions: Vec<String>,
}

/// A concrete Expression record — IFLA FRBR (1998) §3.2.2.
#[derive(Debug, Clone)]
pub struct ExpressionRecord {
    pub title: String,
    pub manifestations: Vec<String>,
}

/// Is `work` realized (does it have at least one Expression)? IFLA FRBR
/// (1998) §3.2.1: "a work is always realized through one or more
/// expressions."
pub fn is_realized(work: &WorkRecord) -> bool {
    !work.expressions.is_empty()
}

/// Is `expression` embodied (does it have at least one Manifestation)?
/// IFLA FRBR (1998) §3.2.2.
pub fn is_embodied(expression: &ExpressionRecord) -> bool {
    !expression.manifestations.is_empty()
}

/// The author of `work` — the FRBR "createdBy" relationship (IFLA FRBR
/// 1998 §5.2, Group 1 to Group 2 responsibility relationships), realized
/// here as the record's own field.
pub fn author_of(work: &WorkRecord) -> &str {
    &work.author
}

/// Every Manifestation `title` appears in, across `expressions` — the
/// "appearsIn" accessor a bibliographic lookup needs (which editions carry
/// this expression).
pub fn appears_in(expression: &ExpressionRecord) -> &[String] {
    &expression.manifestations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn realized_work() -> WorkRecord {
        WorkRecord {
            title: String::from("Iliad"),
            author: String::from("Homer"),
            expressions: alloc::vec![String::from("Lattimore translation")],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_work_with_an_expression_is_realized() {
        assert!(is_realized(&realized_work()));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_work_with_no_expressions_is_not_realized() {
        let w = WorkRecord {
            expressions: Vec::new(),
            ..realized_work()
        };
        assert!(!is_realized(&w));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn author_of_returns_the_records_author() {
        assert_eq!(author_of(&realized_work()), "Homer");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_expression_with_a_manifestation_is_embodied() {
        let e = ExpressionRecord {
            title: String::from("Lattimore translation"),
            manifestations: alloc::vec![String::from("1951 University of Chicago Press edition")],
        };
        assert!(is_embodied(&e));
        assert_eq!(
            appears_in(&e),
            &[String::from("1951 University of Chicago Press edition")]
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_expression_with_no_manifestations_is_not_embodied() {
        let e = ExpressionRecord {
            title: String::from("unpublished draft"),
            manifestations: Vec::new(),
        };
        assert!(!is_embodied(&e));
    }
}
