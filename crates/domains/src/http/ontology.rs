use super::request::Method;
use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::{Axiom, Quality};

/// HTTP methods as entities.
impl Entity for Method {
    fn variants() -> Vec<Self> {
        Method::all()
    }
}

/// Relationship: method compatibility (can follow each other in a session).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodPair {
    pub first: Method,
    pub second: Method,
}

impl Relationship for MethodPair {
    type Object = Method;
    fn source(&self) -> Method {
        self.first
    }
    fn target(&self) -> Method {
        self.second
    }
}

pub struct HttpMethodCategory;

impl Category for HttpMethodCategory {
    type Object = Method;
    type Morphism = MethodPair;

    fn identity(obj: &Method) -> MethodPair {
        MethodPair {
            first: *obj,
            second: *obj,
        }
    }

    fn compose(f: &MethodPair, g: &MethodPair) -> Option<MethodPair> {
        if f.second != g.first {
            return None;
        }
        Some(MethodPair {
            first: f.first,
            second: g.second,
        })
    }

    fn morphisms() -> Vec<MethodPair> {
        let m = Method::all();
        m.iter()
            .flat_map(|&a| {
                m.iter().map(move |&b| MethodPair {
                    first: a,
                    second: b,
                })
            })
            .collect()
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

impl Axiom<HttpMethodCategory> for SafeImpliesIdempotent {
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

    #[test]
    fn test_7_methods() {
        assert_eq!(Method::variants().len(), 7);
    }

    #[test]
    fn test_category_laws() {
        praxis::category::validate::check_category_laws::<HttpMethodCategory>().unwrap();
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
}
