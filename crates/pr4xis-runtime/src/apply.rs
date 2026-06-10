//! `apply` — the data-driven `FreeExtension`: interpret a loaded projection
//! ([`GeneratorAction`]) over a source [`Archive`], producing the target.
//!
//! This is the one irreducible runtime primitive behind "projections live in
//! `.prx`, not code". A projection — e.g. WordNet's relations into praxis kinds
//! (`hypernym` → `Subsumption`, a synset → a `ConceptNode`) — IS a functor, and
//! a functor's whole content is its finite action on generators (the
//! finite-presentation theorem; Lawvere functorial semantics, Fong & Spivak
//! *Seven Sketches* Ch. 3), which [`GeneratorAction::Functor`] already
//! serializes as data. So a projection ships as a content-addressed
//! [`Connection`](crate::connection) node in a `.prx` and is APPLIED here —
//! re-emitting the node updates the projection with no recompile.
//!
//! This is the runtime, data-driven generalization of the compile-time
//! `FreeExtension` (`pr4xis::category::quiver`): where that functor's
//! `on_vertex` / `on_edge` are compiled trait methods, here they are lookups
//! into the loaded `map_object` / `map_morphism` tables. The finite action on
//! generators is *sufficient* — no AST/lambda evaluator is needed (the Unison
//! floor: a content hash is inert, but praxis's evaluator is the cheapest
//! possible one, a finite table lookup), because a structure-preserving map is
//! fully determined by its action on the schema's generators.

use std::collections::BTreeMap;

use crate::archive::Archive;
use crate::connection::GeneratorAction;
use crate::definition::Definition;

/// Why a projection could not be applied. Fail-closed: an unsupported action is
/// refused, never silently producing a wrong archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    /// The action is not a [`GeneratorAction::Functor`]. Only the functor
    /// projection is interpreted today; lens / adjunction / natural-
    /// transformation replay are tracked follow-ups, not stubbed here.
    UnsupportedAction { kind: &'static str },
}

impl core::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApplyError::UnsupportedAction { kind } => write!(
                f,
                "apply: only a Functor projection is interpreted; got {kind} \
                 (lens/adjunction/nat-trans replay is a tracked follow-up)"
            ),
        }
    }
}

impl std::error::Error for ApplyError {}

/// The categorical-family name of an action — for the fail-closed error.
fn action_kind(action: &GeneratorAction) -> &'static str {
    match action {
        GeneratorAction::Functor { .. } => "Functor",
        GeneratorAction::NaturalTransformation { .. } => "NaturalTransformation",
        GeneratorAction::Lens { .. } => "Lens",
        GeneratorAction::Adjunction { .. } => "Adjunction",
    }
}

/// Apply a loaded projection `action` over a `source` [`Archive`], producing the
/// target archive.
///
/// The functor is a SCHEMA relabeling applied element-wise: `map_object` maps a
/// source node's KIND to its target kind, `map_morphism` maps a source edge's
/// KIND to its target kind. A node's name, an edge's target, the lexical
/// grounding and the axioms are the atom's identity-bearing content and are
/// carried UNCHANGED — the functor relabels the *kinds* (the schema's
/// generators), never the instances. (Connections on the source archive are
/// carried through; B1 projects only nodes + edges.)
///
/// A kind absent from the relevant map is the IDENTITY image (carried with its
/// own name) — the open-world stance: an unmapped relation (e.g. `antonym`,
/// pending the full relation-kind vocabulary) is REPRESENTABLE, carried into the
/// target; it is simply not yet folded into a closure by
/// `materialize` (it is not one of the transitive kinds). Nothing is dropped.
pub fn apply(action: &GeneratorAction, source: &Archive) -> Result<Archive, ApplyError> {
    let GeneratorAction::Functor {
        map_object,
        map_morphism,
    } = action
    else {
        return Err(ApplyError::UnsupportedAction {
            kind: action_kind(action),
        });
    };

    // The finite action on generators, as O(1) lookups — the data-driven
    // `on_vertex` / `on_edge` of the free extension.
    let on_vertex: BTreeMap<&str, &str> = map_object
        .iter()
        .map(|(s, t)| (s.as_str(), t.as_str()))
        .collect();
    let on_edge: BTreeMap<&str, &str> = map_morphism
        .iter()
        .map(|(s, t)| (s.as_str(), t.as_str()))
        .collect();

    let nodes = source
        .nodes
        .iter()
        .map(|d| Definition {
            kind: on_vertex
                .get(d.kind.as_str())
                .map_or_else(|| d.kind.clone(), |t| t.to_string()),
            name: d.name.clone(),
            edges: d
                .edges
                .iter()
                .map(|(edge_kind, target)| {
                    (
                        on_edge
                            .get(edge_kind.as_str())
                            .map_or_else(|| edge_kind.clone(), |t| t.to_string()),
                        target.clone(),
                    )
                })
                .collect(),
            axioms: d.axioms.clone(),
            lexical: d.lexical.clone(),
        })
        .collect();

    Ok(Archive {
        nodes,
        connections: source.connections.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synset(name: &str, hypernym: &str) -> Definition {
        Definition {
            kind: "Synset".into(),
            name: name.into(),
            edges: vec![("hypernym".into(), hypernym.into())],
            axioms: vec![],
            lexical: Some("a four-legged animal".into()),
        }
    }

    /// The WordNet→praxis projection-as-data: a `Synset` node relabels to
    /// `ConceptNode`, a `hypernym` edge to `Subsumption` — names, targets and
    /// the gloss carried unchanged. This is the whole point: the relabeling is
    /// the functor's data, applied here, never a compiled `match`.
    fn wordnet_functor() -> GeneratorAction {
        GeneratorAction::Functor {
            map_object: vec![("Synset".into(), "ConceptNode".into())],
            map_morphism: vec![
                ("hypernym".into(), "Subsumption".into()),
                ("holo_part".into(), "Parthood".into()),
            ],
        }
    }

    #[test]
    fn relabels_kinds_carries_identity_content() {
        let source = Archive {
            nodes: vec![synset("dog", "mammal")],
            connections: vec![],
        };
        let target = apply(&wordnet_functor(), &source).unwrap();
        let node = &target.nodes[0];
        assert_eq!(
            node.kind, "ConceptNode",
            "node kind relabeled by map_object"
        );
        assert_eq!(node.name, "dog", "name (identity) carried unchanged");
        assert_eq!(
            node.edges,
            vec![("Subsumption".to_string(), "mammal".to_string())]
        );
        assert_eq!(
            node.lexical.as_deref(),
            Some("a four-legged animal"),
            "the gloss rides the object map's image — it is the node's lexical"
        );
    }

    #[test]
    fn unmapped_kind_is_the_identity_image_not_dropped() {
        // `antonym` is not in the functor (pending the full relation-kind
        // vocabulary). It is carried with its own kind — representable, just not
        // closure-folded — never silently dropped.
        let source = Archive {
            nodes: vec![Definition {
                kind: "Synset".into(),
                name: "hot".into(),
                edges: vec![("antonym".into(), "cold".into())],
                axioms: vec![],
                lexical: None,
            }],
            connections: vec![],
        };
        let target = apply(&wordnet_functor(), &source).unwrap();
        assert_eq!(target.nodes[0].kind, "ConceptNode");
        assert_eq!(
            target.nodes[0].edges,
            vec![("antonym".to_string(), "cold".to_string())],
            "an unmapped relation is carried as-is (representable), not dropped"
        );
    }

    #[test]
    fn re_emitting_a_different_functor_changes_the_projection() {
        // The user's directive realized: the projection is data. A different
        // functor (hypernym → Parthood instead of Subsumption) yields a
        // different target — no code change.
        let source = Archive {
            nodes: vec![synset("dog", "mammal")],
            connections: vec![],
        };
        let remapped = GeneratorAction::Functor {
            map_object: vec![("Synset".into(), "ConceptNode".into())],
            map_morphism: vec![("hypernym".into(), "Parthood".into())],
        };
        let target = apply(&remapped, &source).unwrap();
        assert_eq!(
            target.nodes[0].edges,
            vec![("Parthood".to_string(), "mammal".to_string())]
        );
    }

    #[test]
    fn refuses_a_non_functor_action_fail_closed() {
        let lens = GeneratorAction::Lens {
            view: "Source".into(),
            get: "parse".into(),
            put: "generate".into(),
        };
        let source = Archive {
            nodes: vec![synset("dog", "mammal")],
            connections: vec![],
        };
        assert_eq!(
            apply(&lens, &source).unwrap_err(),
            ApplyError::UnsupportedAction { kind: "Lens" }
        );
    }
}
