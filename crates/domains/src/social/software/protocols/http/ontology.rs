//! HTTP method semantics — the safety / idempotence lattice.
//!
//! This ontology models the abstract semantic categories RFC 9110 attaches
//! to HTTP request methods: every method is *safe*, *idempotent*, or
//! *unsafe non-idempotent*, with safety strictly implying idempotence.
//! The rich `Method` enum in `request.rs` carries the seven concrete
//! method instances and the `is_safe()` / `is_idempotent()` / `has_body()`
//! predicates; this ontology is the upper-layer category over those
//! semantic groupings.
//!
//! # Literature
//!
//! - **RFC 9110 (2022)** *HTTP Semantics* — §9.2.1 safe methods, §9.2.2
//!   idempotent methods. Defines `GET`, `HEAD`, `OPTIONS` as safe and
//!   `GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE` as idempotent. Asserts
//!   safety implies idempotence.
//! - **RFC 9112 (2022)** *HTTP/1.1* — the wire format companion.
//! - **Fielding (2000)** *Architectural Styles and the Design of
//!   Network-based Software Architectures* (REST dissertation) —
//!   originating discussion of uniform-interface method semantics.

use super::request::Method;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Http",
    source: "RFC 9110 (2022) HTTP Semantics; RFC 9112 (2022) HTTP/1.1; Fielding (2000) Architectural Styles and the Design of Network-based Software Architectures",

    concepts: [
        // The seven RFC 9110 methods, plus the three semantic groupings
        // (Safe / Idempotent / WithBody) that organise them.
        Get,
        Post,
        Put,
        Delete,
        Patch,
        Head,
        Options,

        // Semantic categories (RFC 9110 §9.2).
        Safe,
        Idempotent,
        WithBody,
    ],

    labels: {
        Get: ("en", "GET", "RFC 9110 §9.3.1 — retrieve a representation; safe, idempotent."),
        Post: ("en", "POST", "RFC 9110 §9.3.3 — perform target-specific processing on the request payload; neither safe nor idempotent."),
        Put: ("en", "PUT", "RFC 9110 §9.3.4 — replace the target with the request payload; idempotent, not safe."),
        Delete: ("en", "DELETE", "RFC 9110 §9.3.5 — remove the target; idempotent, not safe."),
        Patch: ("en", "PATCH", "RFC 5789 — partial modification; neither safe nor idempotent in general."),
        Head: ("en", "HEAD", "RFC 9110 §9.3.2 — like GET but returns headers only; safe, idempotent."),
        Options: ("en", "OPTIONS", "RFC 9110 §9.3.7 — discover the communication options; safe, idempotent."),

        Safe: ("en", "Safe method",
            "RFC 9110 §9.2.1 — a method whose defined semantics are essentially read-only; the client does not request, and does not expect, any state change on the origin server."),
        Idempotent: ("en", "Idempotent method",
            "RFC 9110 §9.2.2 — a method where the intended effect on the server of multiple identical requests is the same as for a single such request."),
        WithBody: ("en", "Method with request payload",
            "RFC 9110 §6.4 — a method whose semantics allow a request content (POST, PUT, PATCH)."),
    },

    is_a: [
        // Safe ⊂ Idempotent (RFC 9110 §9.2.2 — all safe methods are idempotent).
        (Safe, Idempotent),

        // Per RFC 9110 §9.2.1 / §9.2.2: classification of the seven methods.
        (Get, Safe),
        (Head, Safe),
        (Options, Safe),
        (Put, Idempotent),
        (Delete, Idempotent),

        // Per RFC 9110 §6.4: methods that carry a request payload.
        (Post, WithBody),
        (Put, WithBody),
        (Patch, WithBody),
    ],

    opposes: [
        // Safe and WithBody are disjoint per RFC 9110 §9.2.1 (a safe method
        // performs no action other than retrieval; carrying a payload that
        // modifies state contradicts that intent).
        (Safe, WithBody),
        (WithBody, Safe),
    ],
}

/// The RFC 9110 §9.2 semantic class a request method falls into.
///
/// A closed classification of the three mutually exclusive method
/// semantics defined by RFC 9110 (2022) *HTTP Semantics*: §9.2.1 *safe*
/// (read-only), §9.2.2 *idempotent* (repeatable with the same effect) but
/// not safe, and *non-idempotent* (neither). Safe strictly implies
/// idempotent, so the safe class names the strongest of the three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodSemanticClass {
    /// RFC 9110 §9.2.1 — essentially read-only; no requested state change.
    Safe,
    /// RFC 9110 §9.2.2 — repeatable with the same effect, but not safe.
    Idempotent,
    /// Neither safe nor idempotent (RFC 9110 §9.2.2 ¶ on POST/PATCH).
    NonIdempotent,
}

/// Quality: which [`MethodSemanticClass`] each method falls into.
///
/// Returns the RFC 9110 §9.2 semantic class for each of the seven concrete
/// methods; None for the abstract grouping concepts.
#[derive(Debug, Clone)]
pub struct MethodSemantics;

impl Quality for MethodSemantics {
    type Individual = HttpConcept;
    type Value = MethodSemanticClass;

    fn get(&self, c: &HttpConcept) -> Option<MethodSemanticClass> {
        use HttpConcept as H;
        match c {
            H::Get | H::Head | H::Options => Some(MethodSemanticClass::Safe),
            H::Put | H::Delete => Some(MethodSemanticClass::Idempotent),
            H::Post | H::Patch => Some(MethodSemanticClass::NonIdempotent),
            // Grouping concepts have no per-method semantics tag.
            H::Safe | H::Idempotent | H::WithBody => None,
        }
    }
}

/// Map a concrete `HttpConcept` method variant to the rich `Method` enum
/// in `request.rs`. Identity on the seven methods; None on the groupings.
///
/// This is the bridge between the ontology layer (abstract categories
/// reasoned about by Praxis) and the runtime layer (`Method` driving the
/// `Request` type's body-permission enforcement).
pub fn concept_to_method(c: HttpConcept) -> Option<Method> {
    Some(match c {
        HttpConcept::Get => Method::Get,
        HttpConcept::Post => Method::Post,
        HttpConcept::Put => Method::Put,
        HttpConcept::Delete => Method::Delete,
        HttpConcept::Patch => Method::Patch,
        HttpConcept::Head => Method::Head,
        HttpConcept::Options => Method::Options,
        HttpConcept::Safe | HttpConcept::Idempotent | HttpConcept::WithBody => return None,
    })
}

impl Ontology for HttpOntology {
    type Cat = HttpCategory;
    type Qual = MethodSemantics;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SafeImpliesIdempotent));
        axioms.push(Box::new(SafeMethodsHaveNoBody));
        axioms
    }
}

/// Axiom: every safe method is idempotent (RFC 9110 §9.2.2).
///
/// "All of the safe methods that are defined in this specification are
///  also idempotent." — RFC 9110 §9.2.2 ¶2. Verified against the rich
///  `Method` enum: for each of the seven methods, `is_safe()` implies
///  `is_idempotent()`.
pub struct SafeImpliesIdempotent;

impl Axiom for SafeImpliesIdempotent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = Method::all()
            .iter()
            .all(|m| !m.is_safe() || m.is_idempotent());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SafeImpliesIdempotent",
        "every safe HTTP method is idempotent",
        "RFC 9110 (2022) HTTP Semantics §9.2.2"
    );
}

pr4xis::register_axiom!(
    SafeImpliesIdempotent,
    "RFC 9110 (2022) HTTP Semantics §9.2.2"
);

/// Axiom: no safe method carries a request body (RFC 9110 §9.2.1 / §6.4).
///
/// The Safe / WithBody concepts are declared disjoint by the `opposes:`
/// clause above; this axiom verifies the disjointness at the rich-type
/// level by walking `Method::all()` and checking `!is_safe() || !has_body()`.
pub struct SafeMethodsHaveNoBody;

impl Axiom for SafeMethodsHaveNoBody {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = Method::all().iter().all(|m| !m.is_safe() || !m.has_body());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SafeMethodsHaveNoBody",
        "no safe HTTP method carries a request payload",
        "RFC 9110 (2022) HTTP Semantics §9.2.1, §6.4"
    );
}

pr4xis::register_axiom!(
    SafeMethodsHaveNoBody,
    "RFC 9110 (2022) HTTP Semantics §9.2.1, §6.4"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<HttpCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        HttpOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_concepts() {
        // 7 methods + 3 groupings (Safe, Idempotent, WithBody).
        assert_eq!(HttpConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn safe_subsumes_idempotent_via_is_a() {
        // RFC 9110 §9.2.2: all safe methods are idempotent.
        let sub: Vec<_> = HttpCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == HttpRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(HttpConcept::Safe, HttpConcept::Idempotent)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn safe_methods_are_get_head_options() {
        let sub: Vec<_> = HttpCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == HttpRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(HttpConcept::Get, HttpConcept::Safe)));
        assert!(sub.contains(&(HttpConcept::Head, HttpConcept::Safe)));
        assert!(sub.contains(&(HttpConcept::Options, HttpConcept::Safe)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn safe_implies_idempotent_holds() {
        match SafeImpliesIdempotent.verify() {
            Ok(_) => {}
            Err(c) => panic!("SafeImpliesIdempotent failed: {}", c.meta().name.as_str()),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn safe_methods_have_no_body_holds() {
        match SafeMethodsHaveNoBody.verify() {
            Ok(_) => {}
            Err(c) => panic!("SafeMethodsHaveNoBody failed: {}", c.meta().name.as_str()),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn method_semantics_totality_over_methods() {
        // The seven concrete methods each get a semantics tag.
        let q = MethodSemantics;
        for m in [
            HttpConcept::Get,
            HttpConcept::Post,
            HttpConcept::Put,
            HttpConcept::Delete,
            HttpConcept::Patch,
            HttpConcept::Head,
            HttpConcept::Options,
        ] {
            assert!(q.get(&m).is_some(), "{:?} missing semantics tag", m);
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn concept_to_method_round_trip() {
        // Round-trip the seven concrete methods.
        assert_eq!(concept_to_method(HttpConcept::Get), Some(Method::Get));
        assert_eq!(concept_to_method(HttpConcept::Post), Some(Method::Post));
        assert_eq!(concept_to_method(HttpConcept::Put), Some(Method::Put));
        assert_eq!(concept_to_method(HttpConcept::Delete), Some(Method::Delete));
        assert_eq!(concept_to_method(HttpConcept::Patch), Some(Method::Patch));
        assert_eq!(concept_to_method(HttpConcept::Head), Some(Method::Head));
        assert_eq!(
            concept_to_method(HttpConcept::Options),
            Some(Method::Options)
        );
        assert_eq!(concept_to_method(HttpConcept::Safe), None);
    }

    fn arb_concept() -> impl Strategy<Value = HttpConcept> {
        proptest::sample::select(HttpConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in HttpCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in HttpOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = HttpConcept::variants();
            for m in HttpCategory::morphisms() {
                if m.kind() == HttpRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_method_semantics_total_on_methods(c in arb_concept()) {
            // Total function: the 7 concrete methods each have a tag;
            // calling .get is always safe (returns None for groupings).
            let _ = MethodSemantics.get(&c);
        }

        #[test]
        fn prop_rfc_9110_safe_implies_idempotent(_seed in any::<u32>()) {
            // For every concrete `Method`, RFC 9110 §9.2.2 holds.
            for m in Method::all() {
                if m.is_safe() {
                    prop_assert!(m.is_idempotent(), "{:?} is safe but not idempotent", m);
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_method_semantics_total_on_methods, Honest);
    pr4xis::register_praxis_value!(prop_rfc_9110_safe_implies_idempotent, Verifiable);
}
