//! Test-only `Arbitrary OwlOntology` strategy shared by the
//! `writer.rs` and `lens.rs` proptest modules.
//!
//! The generator covers the structural shapes the canonical writer and
//! lens must handle robustly: 0–3 named classes (each with optional
//! labels, 0–2 comments, 0–2 superclass IRIs or anonymous
//! `SomeValuesFrom` / `HasValue` restrictions), 0–3 object properties
//! (similar shape — labels / comments / superproperties / inverse_of),
//! and 0–2 anonymous restrictions referenced by the subclass edges.
//! Every IRI is drawn from a small fixed namespace (`http://ex.org/…`)
//! so the writer's NCName projection and the reader's QName lookup both
//! exercise the deterministic-namespace-binding path.
//!
//! The shape is intentionally a subset of the full W3C OWL 2 RDF Mapping
//! (Patel-Schneider & Motik 2012); the corpus-wide audit on the six
//! bundled vocabularies covers the full structural variety, this
//! proptest covers structural-robustness invariants — determinism,
//! canonical ordering, round-trip equivalence — against any
//! within-subset instance.

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use proptest::prelude::*;

use super::ontology::{
    OwlAnnotation, OwlAnnotationValue, OwlClass, OwlClassExpression, OwlIndividual, OwlLiteral,
    OwlObjectProperty, OwlOntology, OwlRestriction, OwlRestrictionKind,
};

/// Small fixed namespace every generated IRI is drawn from.
const NS: &str = "http://ex.org/";

fn arb_local() -> impl Strategy<Value = String> {
    // Six local names cover every collision/order case for blocks of
    // up to 3 entities while keeping shrinkable cardinality small.
    proptest::sample::select(&["A", "B", "C", "D", "E", "F"][..]).prop_map(|s| format!("{NS}{s}"))
}

fn arb_property_iri() -> impl Strategy<Value = String> {
    proptest::sample::select(&["p1", "p2", "p3", "p4"][..]).prop_map(|s| format!("{NS}{s}"))
}

fn arb_literal() -> impl Strategy<Value = OwlLiteral> {
    // Three lexical-form shapes (plain / lang-tagged / typed) so the
    // writer's literal-sort key sees mixed (lang, datatype, lexical)
    // tuples; the value strings stay ASCII-safe so XML escaping isn't
    // exercised here.
    prop_oneof![
        proptest::sample::select(&["a", "b", "c"][..]).prop_map(|s| OwlLiteral {
            lexical: s.to_string(),
            lang: None,
            datatype: None,
        }),
        (
            proptest::sample::select(&["a", "b"][..]),
            proptest::sample::select(&["en", "fr"][..]),
        )
            .prop_map(|(s, lang)| OwlLiteral {
                lexical: s.to_string(),
                lang: Some(lang.to_string()),
                datatype: None,
            }),
        proptest::sample::select(&["1", "2"][..]).prop_map(|s| OwlLiteral {
            lexical: s.to_string(),
            lang: None,
            datatype: Some("http://www.w3.org/2001/XMLSchema#integer".to_string()),
        }),
    ]
}

fn arb_annotation_value() -> impl Strategy<Value = OwlAnnotationValue> {
    prop_oneof![
        arb_local().prop_map(OwlAnnotationValue::Iri),
        arb_literal().prop_map(OwlAnnotationValue::Literal),
    ]
}

/// Restriction strategy: `SomeValuesFrom(Named)` or `HasValue(Iri|Literal)`.
/// These are the two paths the lens must handle without
/// content-addressed-blank-node collisions on the generated subset.
fn arb_restriction() -> impl Strategy<Value = OwlRestriction> {
    let on_property = arb_property_iri();
    let kind = prop_oneof![
        arb_local().prop_map(|i| OwlRestrictionKind::SomeValuesFrom(Box::new(
            OwlClassExpression::Named(i),
        ))),
        arb_annotation_value().prop_map(OwlRestrictionKind::HasValue),
    ];
    (on_property, kind).prop_map(|(on_property, kind)| OwlRestriction { on_property, kind })
}

fn arb_superclass_expr() -> impl Strategy<Value = OwlClassExpression> {
    prop_oneof![
        arb_local().prop_map(OwlClassExpression::Named),
        arb_restriction().prop_map(OwlClassExpression::Restriction),
    ]
}

/// Content-dedupe a literal multiset — the reader applies this same
/// normalization (`dedup_literals`) so the generator pre-applies it so
/// the round-trip is meaningful.
fn dedup_literals(mut v: Vec<OwlLiteral>) -> Vec<OwlLiteral> {
    let mut seen: Vec<OwlLiteral> = Vec::new();
    v.retain(|l| {
        if seen.iter().any(|x| x == l) {
            false
        } else {
            seen.push(l.clone());
            true
        }
    });
    v
}

/// Content-dedupe a class-expression list — mirrors
/// `dedup_class_expressions` in the reader.
fn dedup_exprs(mut v: Vec<OwlClassExpression>) -> Vec<OwlClassExpression> {
    let mut seen: Vec<OwlClassExpression> = Vec::new();
    v.retain(|e| {
        if seen.iter().any(|x| x == e) {
            false
        } else {
            seen.push(e.clone());
            true
        }
    });
    v
}

/// Content-dedupe a string IRI list — mirrors `dedup_strings` in the
/// reader (superproperties, equivalent_properties, …).
fn dedup_strings(mut v: Vec<String>) -> Vec<String> {
    let mut seen: alloc::collections::BTreeSet<String> = alloc::collections::BTreeSet::new();
    v.retain(|s| seen.insert(s.clone()));
    v
}

fn arb_class() -> impl Strategy<Value = OwlClass> {
    (
        arb_local(),
        proptest::option::of(arb_literal()),
        prop::collection::vec(arb_literal(), 0..=2),
        prop::collection::vec(arb_superclass_expr(), 0..=2),
    )
        .prop_map(|(iri, label_opt, comments, supers)| {
            let labels: Vec<OwlLiteral> = dedup_literals(label_opt.iter().cloned().collect());
            let comments = dedup_literals(comments);
            let supers = dedup_exprs(supers);
            let label_first = labels.first().map(|l| l.lexical.clone());
            let comment_first = comments.first().map(|l| l.lexical.clone());
            // Named-superclass projection list mirrors what the reader
            // would compute from the parsed expression set.
            let mut superclasses: Vec<String> = Vec::new();
            for e in &supers {
                if let OwlClassExpression::Named(s) = e
                    && !superclasses.iter().any(|x| x == s)
                {
                    superclasses.push(s.clone());
                }
            }
            OwlClass {
                iri,
                label: label_first,
                comment: comment_first,
                superclasses,
                labels,
                comments,
                annotations: Vec::new(),
                superclass_expressions: supers,
                equivalent_classes: Vec::new(),
                disjoint_classes: Vec::new(),
            }
        })
}

fn arb_property() -> impl Strategy<Value = OwlObjectProperty> {
    (
        arb_property_iri(),
        proptest::option::of(arb_literal()),
        prop::collection::vec(arb_literal(), 0..=2),
        prop::collection::vec(arb_property_iri(), 0..=2),
        proptest::option::of(arb_property_iri()),
    )
        .prop_map(|(iri, label_opt, comments, supers, inverse_of)| {
            let labels: Vec<OwlLiteral> = dedup_literals(label_opt.iter().cloned().collect());
            let comments = dedup_literals(comments);
            let supers = dedup_strings(supers);
            let label_first = labels.first().map(|l| l.lexical.clone());
            let comment_first = comments.first().map(|l| l.lexical.clone());
            OwlObjectProperty {
                iri,
                label: label_first,
                comment: comment_first,
                domain: None,
                range: None,
                superproperties: supers,
                labels,
                comments,
                annotations: Vec::new(),
                inverse_of,
                equivalent_properties: Vec::new(),
                disjoint_properties: Vec::new(),
            }
        })
}

/// Deduplicate by IRI so the writer's per-block lexicographic
/// projection isn't fed two `OwlClass` entries with the same IRI (the
/// `owl_equivalent` projection compares the IRI *set*, so duplicates
/// would falsely shrink one side's set).
fn dedup_classes(mut cs: Vec<OwlClass>) -> Vec<OwlClass> {
    let mut seen: alloc::collections::BTreeSet<String> = alloc::collections::BTreeSet::new();
    cs.retain(|c| seen.insert(c.iri.clone()));
    cs
}

fn dedup_properties(mut ps: Vec<OwlObjectProperty>) -> Vec<OwlObjectProperty> {
    let mut seen: alloc::collections::BTreeSet<String> = alloc::collections::BTreeSet::new();
    ps.retain(|p| seen.insert(p.iri.clone()));
    ps
}

/// Top-level `Arbitrary OwlOntology` strategy — 0..=3 classes and
/// 0..=3 properties, dedup'd by IRI, no individuals (the named-class /
/// object-property paths are the audit's gap focus).
pub(crate) fn arb_ontology() -> impl Strategy<Value = OwlOntology> {
    (
        prop::collection::vec(arb_class(), 0..=3),
        prop::collection::vec(arb_property(), 0..=3),
    )
        .prop_map(|(classes, properties)| {
            let classes = dedup_classes(classes);
            let properties = dedup_properties(properties);
            // taxonomy: derived from named-superclass projection.
            let mut taxonomy: Vec<(String, String)> = Vec::new();
            for c in &classes {
                for s in &c.superclasses {
                    taxonomy.push((c.iri.clone(), s.clone()));
                }
            }
            let mut property_taxonomy: Vec<(String, String)> = Vec::new();
            for p in &properties {
                for s in &p.superproperties {
                    property_taxonomy.push((p.iri.clone(), s.clone()));
                }
            }
            OwlOntology {
                iri: "http://ex.org/ont".to_string(),
                classes,
                properties,
                individuals: Vec::<OwlIndividual>::new(),
                taxonomy,
                property_taxonomy,
                ontology_annotations: Vec::<OwlAnnotation>::new(),
            }
        })
}
