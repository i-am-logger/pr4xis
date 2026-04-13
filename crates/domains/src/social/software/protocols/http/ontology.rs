use super::request::Method;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

define_ontology! {
    pub HttpMethodOntology for HttpMethodCategory {
        concepts: Method,
        relation: MethodPair,
    }
}

impl Ontology for HttpMethodOntology {
    type Cat = HttpMethodCategory;
    type Qual = IsSafe;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(SafeImpliesIdempotent)]
    }
}

/// Quality: is this method safe?
#[derive(Debug, Clone)]
pub struct IsSafe;

impl Quality for IsSafe {
    type Individual = Method;
    type Value = ();

    fn get(&self, method: &Method) -> Option<()> {
        if method.is_safe() { Some(()) } else { None }
    }
}

/// Quality: is this method idempotent?
#[derive(Debug, Clone)]
pub struct IsIdempotent;

impl Quality for IsIdempotent {
    type Individual = Method;
    type Value = ();

    fn get(&self, method: &Method) -> Option<()> {
        if method.is_idempotent() {
            Some(())
        } else {
            None
        }
    }
}

/// Axiom: all safe methods are idempotent.
pub struct SafeImpliesIdempotent;

impl Axiom for SafeImpliesIdempotent {
    fn description(&self) -> &str {
        "all safe methods must be idempotent"
    }
    fn holds(&self) -> bool {
        Method::all()
            .iter()
            .all(|m| !m.is_safe() || m.is_idempotent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;

    #[test]
    fn test_7_methods() {
        assert_eq!(Method::variants().len(), 7);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<HttpMethodCategory>().unwrap();
    }

    #[test]
    fn test_safe_methods() {
        assert_eq!(IsSafe.individuals_with().len(), 3); // GET, HEAD, OPTIONS
    }

    #[test]
    fn test_idempotent_methods() {
        assert_eq!(IsIdempotent.individuals_with().len(), 5); // GET, PUT, DELETE, HEAD, OPTIONS
    }

    #[test]
    fn test_safe_implies_idempotent() {
        assert!(SafeImpliesIdempotent.holds());
    }

    #[test]
    fn ontology_validates() {
        HttpMethodOntology::validate().unwrap();
    }
}
