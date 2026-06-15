//! USC → [`Archive`](pr4xis_runtime::archive::Archive) — project a loaded
//! [`UsCode`] into the GENERIC runtime substrate, the analog of the English
//! bridge ([`english::bridge`](crate::cognitive::linguistics::english::bridge)).
//!
//! This dissolves the same SUBSTRATE SPLIT B1 dissolved for English: a loaded
//! `UsCode` is a closed domain struct (`&'static` sections + subdivisions), not a
//! runtime [`Archive`](pr4xis_runtime::archive::Archive), so a generic engine has
//! no addressable atom for a statute provision and no traverser over its
//! structure. [`project_archive`](crate::social::software::markup::xml::uslm::corpus::bridge::project_archive) makes each section and
//! subdivision a definition-bearing
//! [`Definition`](pr4xis_runtime::definition::Definition) node (its URN is its
//! name, its RAW USLM tag its kind, its prose its lexical), and the USLM Composes
//! hierarchy a RAW `Composes` graph.
//!
//! # The relabeling is data, not code
//!
//! Like the WordNet and OWL bridges, the semantic map is NOT baked into the
//! projector: `project_archive` emits the RAW USLM generators (a `<section>` tag,
//! a `Composes` edge); mapping `section ↦ Section` and `Composes ↦ Parthood`
//! (Casati & Varzi 1999 — a subdivision is PART-OF its parent) is a separate
//! FUNCTOR carried AS `.prx` DATA ([`usc_to_praxis_functor`]) and interpreted by
//! the one runtime primitive [`apply`](pr4xis_runtime::apply::apply).
//! [`usc_runtime_ontology`] is the whole pipeline (`project → apply → materialize`),
//! the verbatim shape of `english_runtime_ontology`. Parthood is a canonically
//! transitive kind [`materialize`](pr4xis_runtime::ontology) folds, so the
//! mereology is a real closure post-apply.
//!
//! It is the SUBSTRATE grounding rides: a statute provision is now a `Definition`
//! that can carry a typed
//! [`EdgeTarget::Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge
//! into a connected ontology (the lexical `denotes` floor, and later `cites` /
//! `defines`), resolved by the generic `AtomResolver` — never a bespoke string
//! side-channel.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::{Connection, GeneratorAction};
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::ontology::{MaterializeError, RuntimeOntology, materialize};

use super::UsCode;
use super::section_aux::UscSubdivision;
use crate::cognitive::linguistics::english::bridge::form_atom;

/// The RAW USLM node tag of a section in the SOURCE archive — the `<section>`
/// element name the projector emits, before [`usc_to_praxis_functor`] relabels it
/// to [`SECTION_KIND`]. (Subdivisions already carry their raw USLM tag via
/// `sub.kind.tag()`, so only the section root needed a name.)
pub const SECTION_TAG: &str = "section";

/// The praxis node kind a section relabels to — appears ONLY in the functor DATA,
/// never baked into the structural projection.
pub const SECTION_KIND: &str = "Section";

/// The RAW USLM relation of the Composes hierarchy in the SOURCE archive — a
/// subdivision Composes INTO its parent. The schema generator
/// [`usc_to_praxis_functor`] maps to [`PARTHOOD_REL`]; emitted raw so the
/// mereological reading is loaded data, not baked into the projector.
pub const COMPOSES_REL: &str = "Composes";

/// The praxis relation kind a Composes edge relabels to — Parthood (Casati &
/// Varzi 1999), one of the canonically transitive kinds
/// [`materialize`](pr4xis_runtime::ontology) folds, so the projection's mereology
/// is a real closure (a clause is transitively part-of its section). Appears ONLY
/// in the functor DATA.
pub const PARTHOOD_REL: &str = "Parthood";

/// The raw relation linking a section to its HEADING surface, before the functor
/// relabels it to [`CANONICAL_FORM_REL`] (§9 lexicalization).
pub const HEADING_REL: &str = "heading";

/// The raw relation linking a section to its CITATION surface ("section &lt;num&gt;"),
/// before the functor relabels it to [`OTHER_FORM_REL`].
pub const CITATION_REL: &str = "citation";

/// The praxis lexicalization role a `heading` edge relabels to — Lemon
/// `ontolex:canonicalForm` (the section's one canonical surface). Functor DATA.
pub const CANONICAL_FORM_REL: &str = "canonicalForm";

/// The praxis lexicalization role a `citation` edge relabels to — Lemon
/// `ontolex:otherForm` (a curated variant surface). Functor DATA.
pub const OTHER_FORM_REL: &str = "otherForm";

/// The citation surface for a section URN — `"section <num>"`, the curated
/// `otherForm` a person actually types ("section 1514a"). Derived from the section
/// URN (`/us/usc/tNN/s<num>`): the trailing `/s<num>` segment with `s` stripped.
/// `None` if the URN has no `s`-prefixed section segment (defensive — every USLM
/// section URN has one).
fn section_citation(urn: &str) -> Option<String> {
    let num = urn.rsplit('/').next()?.strip_prefix('s')?;
    (!num.is_empty()).then(|| format!("section {num}"))
}

/// Project a loaded [`UsCode`] into the RAW generic runtime [`Archive`] — the
/// structural transcription only (the praxis relabel is [`usc_to_praxis_functor`]).
///
/// Each section → a [`Definition`] `{kind: `[`SECTION_TAG`]`, name: urn, lexical:
/// heading}`; each subdivision → `{kind: its raw USLM tag (`subsection` /
/// `paragraph` / …), name: urn, lexical: heading∣chapeau∣content, edges:
/// [(`[`COMPOSES_REL`]`, parent_urn)]}`. Every Composes target is a declared
/// section/subdivision node, so the archive is referentially closed and (after
/// the functor relabels Composes→Parthood)
/// [`materialize`](pr4xis_runtime::ontology::materialize)s into a real mereology.
pub fn project_archive(usc: &UsCode) -> Archive {
    // Project one subdivision (and its descendants) — each composes INTO its
    // parent, so the Composes hierarchy is read straight off the tree (the
    // `relations` list is a redundant projection of the same structure and may be
    // empty). The raw `Composes` edge becomes Parthood under the functor — by
    // content-addressed name agreement within this archive.
    fn project_subdivision(
        sub: &'static UscSubdivision,
        parent_urn: &str,
        nodes: &mut Vec<Definition>,
    ) {
        nodes.push(Definition {
            kind: sub.kind.tag().to_string(),
            name: sub.urn.value().to_string(),
            edges: alloc::vec![(
                COMPOSES_REL.to_string(),
                EdgeTarget::Local(parent_urn.to_string()),
            )],
            axioms: Vec::new(),
            lexical: sub
                .heading
                .or(sub.chapeau)
                .or(sub.content)
                .map(ToString::to_string),
        });
        for child in sub.children {
            project_subdivision(child, sub.urn.value(), nodes);
        }
    }

    let mut nodes = Vec::new();
    // Distinct lexicalization surfaces (heading + citation), minted as Form atoms
    // after the walk so the archive stays referentially closed with no duplicate.
    let mut form_surfaces: BTreeSet<String> = BTreeSet::new();
    for section in usc.all_sections() {
        let section_urn = section.urn.value();
        // Lexicalization (§9): a section's heading is its canonicalForm surface,
        // and "section <num>" (from the URN) is a curated otherForm citation — raw
        // edges the functor maps to canonicalForm/otherForm, pointing at the Form
        // atoms below. So the chat answers "what is section 1514a", not only the URN.
        let mut edges: Vec<(String, EdgeTarget)> = Vec::new();
        if !section.heading.is_empty() {
            edges.push((
                HEADING_REL.to_string(),
                EdgeTarget::Local(section.heading.clone()),
            ));
            form_surfaces.insert(section.heading.clone());
        }
        if let Some(citation) = section_citation(section_urn) {
            edges.push((
                CITATION_REL.to_string(),
                EdgeTarget::Local(citation.clone()),
            ));
            form_surfaces.insert(citation);
        }
        nodes.push(Definition {
            kind: SECTION_TAG.to_string(),
            name: section_urn.to_string(),
            edges, // a section is a Composes root; its lexicalization edges ride here
            axioms: Vec::new(),
            lexical: Some(section.heading.clone()),
        });
        // Top-level subdivisions compose into the section; nested ones into their
        // parent subdivision (tracked through the walk).
        for top in section.subdivisions {
            project_subdivision(top, section_urn, &mut nodes);
        }
    }
    // One `ontolex:Form` atom per distinct surface (the writtenRep the composed
    // reasoner indexes as queryable).
    for surface in &form_surfaces {
        nodes.push(form_atom(surface));
    }
    Archive {
        nodes,
        connections: Vec::new(),
    }
}

/// The USC → praxis projection, carried AS DATA — the [`Connection`] a `.prx`
/// ships so the relabeling re-emits to update with no recompile (the OWL/WordNet
/// pattern, applied to statutes).
///
/// The semantic claims of the USC projection — a `<section>` is a praxis
/// [`SECTION_KIND`], and the USLM `Composes` hierarchy is mereological
/// [`PARTHOOD_REL`] — are this DATA, not a baked projector kind. Interpreted by
/// [`apply`] over a raw [`project_archive`] source. Re-emitting it (say
/// `Composes ↦ Containment`) re-aims the projection without touching code.
pub fn usc_to_praxis_functor() -> Connection {
    Connection {
        kind: "Faithful".to_string(),
        source: "UsCode".to_string(),
        target: "PraxisOntology".to_string(),
        action: GeneratorAction::Functor {
            map_object: vec![(SECTION_TAG.to_string(), SECTION_KIND.to_string())],
            map_morphism: vec![
                (COMPOSES_REL.to_string(), PARTHOOD_REL.to_string()),
                (HEADING_REL.to_string(), CANONICAL_FORM_REL.to_string()),
                (CITATION_REL.to_string(), OTHER_FORM_REL.to_string()),
            ],
        },
        laws: vec![
            "PreservesIdentity".to_string(),
            "PreservesComposition".to_string(),
        ],
    }
}

/// Bridge a loaded [`UsCode`] into a generic [`RuntimeOntology`] — the whole
/// pipeline in one call: [`project_archive`] → [`apply`]`(`[`usc_to_praxis_functor`]`)`
/// → [`materialize`]. The verbatim shape of
/// [`english_runtime_ontology`](crate::cognitive::linguistics::english::bridge::english_runtime_ontology)
/// and `owl_runtime_ontology`.
///
/// `apply` cannot fail here ([`usc_to_praxis_functor`] is always a `Functor`
/// action); materialization can still fail closed (a codec error on the root),
/// propagated typed.
pub fn usc_runtime_ontology(
    usc: &UsCode,
    name: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    let source = project_archive(usc);
    let praxis = apply(&usc_to_praxis_functor().action, &source)
        .expect("usc_to_praxis_functor is a Functor action, which apply always interprets");
    materialize(praxis, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::bridge::FORM_KIND;
    use alloc::collections::BTreeSet;

    #[test]
    fn projects_every_section_and_subdivision_as_a_node() {
        let usc = UsCode::sample();
        let archive = project_archive(&usc);
        let expected: usize = usc.section_count()
            + usc
                .all_sections()
                .iter()
                .map(|s| s.subdivision_count())
                .sum::<usize>();
        // CONCEPTS (non-Form nodes) — one per section + per subdivision; the §9
        // Form atoms (heading / citation surfaces) are excluded.
        let concepts = archive.nodes.iter().filter(|n| n.kind != FORM_KIND).count();
        assert_eq!(
            concepts, expected,
            "one concept node per section + subdivision"
        );
        assert!(archive.connections.is_empty());
        // The structural projection carries the RAW USLM section tag — never the
        // praxis "Section" kind (that relabel is the functor's job).
        assert!(archive.nodes.iter().any(|n| n.kind == SECTION_TAG));
        assert!(archive.nodes.iter().all(|n| !n.name.is_empty()));
    }

    #[test]
    fn every_projected_edge_is_referentially_closed() {
        // Every projected edge — the RAW Composes mereology OR the §9
        // heading/citation lexicalization — is a same-archive local edge to a
        // DECLARED node (the referential closure `materialize` enforces). The
        // praxis relabels (Composes→Parthood, heading→canonicalForm) are the
        // functor's job; the structural projection emits only these raw kinds.
        let archive = project_archive(&UsCode::sample());
        let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                let to = target
                    .local_name()
                    .expect("a projected edge is a same-archive local edge");
                assert!(
                    declared.contains(to),
                    "edge {}--{kind}-->{to} names an undeclared node",
                    n.name
                );
                assert!(
                    kind == COMPOSES_REL || kind == HEADING_REL || kind == CITATION_REL,
                    "projected edge kinds are Composes / heading / citation; got {kind}"
                );
            }
        }
    }

    #[test]
    fn the_functor_relabels_raw_section_and_composes_to_praxis_kinds() {
        // The byte-identity / relabel gate: apply (kind-relabel only) over a raw
        // section⟵subsection Composes archive reproduces EXACTLY the praxis kinds
        // the old direct projector baked — section→Section, Composes→Parthood —
        // so the materialized ontology is byte-identical to before the lift.
        let raw = Archive {
            nodes: vec![
                Definition {
                    kind: SECTION_TAG.to_string(),
                    name: "/us/usc/t1/s1".to_string(),
                    edges: Vec::new(),
                    axioms: Vec::new(),
                    lexical: Some("Words denoting number, gender, and so forth.".to_string()),
                },
                Definition {
                    kind: "subsection".to_string(),
                    name: "/us/usc/t1/s1/a".to_string(),
                    edges: vec![(
                        COMPOSES_REL.to_string(),
                        EdgeTarget::Local("/us/usc/t1/s1".to_string()),
                    )],
                    axioms: Vec::new(),
                    lexical: None,
                },
            ],
            connections: Vec::new(),
        };
        let praxis = apply(&usc_to_praxis_functor().action, &raw).expect("Functor applies");
        let section = praxis
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t1/s1")
            .unwrap();
        assert_eq!(
            section.kind, SECTION_KIND,
            "raw 'section' → praxis 'Section'"
        );
        let subsection = praxis
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t1/s1/a")
            .unwrap();
        assert_eq!(
            subsection.kind, "subsection",
            "raw subdivision tags carry through (identity)"
        );
        assert_eq!(
            subsection.edges[0].0, PARTHOOD_REL,
            "raw 'Composes' edge → praxis 'Parthood'"
        );
    }

    #[test]
    fn usc_runtime_ontology_materializes_the_full_pipeline() {
        // The whole point: project → apply(functor) → materialize gives a generic
        // ontology a source-agnostic engine reasons over (referential closure holds,
        // the Parthood mereology folds into a closure). Verbatim shape of
        // english/owl runtime_ontology.
        let onto = usc_runtime_ontology(&UsCode::sample(), OntologyName::new_static("us_code"))
            .expect("the USC pipeline materializes");
        assert!(onto.archive().nodes.len() > 1);
        // Post-apply, the section nodes carry the praxis Section kind.
        assert!(onto.archive().nodes.iter().any(|n| n.kind == SECTION_KIND));
    }
}
