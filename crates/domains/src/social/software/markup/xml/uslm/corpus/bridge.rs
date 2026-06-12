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
//! name, its USLM kind its kind, its prose its lexical), and the USLM Composes
//! hierarchy a `Parthood` closure (a subdivision is PART-OF its parent; Casati &
//! Varzi 1999).
//!
//! It is the SUBSTRATE grounding rides: a statute provision is now a `Definition`
//! that can carry a typed
//! [`EdgeTarget::Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge
//! into a connected ontology (the lexical `denotes` floor, and later `cites` /
//! `defines`), resolved by the generic `AtomResolver` — never a bespoke string
//! side-channel.

use alloc::string::ToString;
use alloc::vec::Vec;

use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::{Definition, EdgeTarget};

use super::UsCode;
use super::section_aux::UscSubdivision;

/// The praxis kind of a section node.
pub const SECTION_KIND: &str = "Section";

/// The praxis relation kind of the USLM Composes hierarchy — a subdivision is
/// PART-OF its parent. Parthood is one of the canonically transitive kinds
/// [`materialize`](pr4xis_runtime::ontology) folds, so the projection's mereology
/// is a real closure (a clause is transitively part-of its section).
pub const COMPOSES_REL: &str = "Parthood";

/// Project a loaded [`UsCode`] into the generic runtime [`Archive`].
///
/// Each section → a [`Definition`] `{kind: "Section", name: urn, lexical:
/// heading}`; each subdivision → `{kind: its USLM tag (`subsection` / `paragraph`
/// / …), name: urn, lexical: heading∣chapeau∣content, edges: [(Parthood,
/// parent_urn)]}`. Every Composes target is a declared section/subdivision node,
/// so the archive is referentially closed and
/// [`materialize`](pr4xis_runtime::ontology::materialize)s.
pub fn project_archive(usc: &UsCode) -> Archive {
    // Project one subdivision (and its descendants) — each composes INTO its
    // parent, so the Composes hierarchy is read straight off the tree (the
    // `relations` list is a redundant projection of the same structure and may be
    // empty). A subdivision is PART-OF its parent — Parthood, by content-addressed
    // name agreement within this archive.
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
    for section in usc.all_sections() {
        let section_urn = section.urn.value();
        nodes.push(Definition {
            kind: SECTION_KIND.to_string(),
            name: section_urn.to_string(),
            edges: Vec::new(), // a section is a root in this projection
            axioms: Vec::new(),
            lexical: Some(section.heading.clone()),
        });
        // Top-level subdivisions compose into the section; nested ones into their
        // parent subdivision (tracked through the walk).
        for top in section.subdivisions {
            project_subdivision(top, section_urn, &mut nodes);
        }
    }
    Archive {
        nodes,
        connections: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(
            archive.nodes.len(),
            expected,
            "one node per section + per subdivision"
        );
        assert!(archive.connections.is_empty());
        // Every section node carries the Section kind; nothing is unnamed.
        assert!(archive.nodes.iter().any(|n| n.kind == SECTION_KIND));
        assert!(archive.nodes.iter().all(|n| !n.name.is_empty()));
    }

    #[test]
    fn every_projected_edge_is_a_referentially_closed_parthood() {
        // The only edge kind the projection emits is the Composes mereology, and
        // every target is a declared section/subdivision (the precondition
        // `materialize` enforces). `UsCode::sample()` is FLAT (no subdivision
        // tree), so it has no Parthood edges — the invariant holds vacuously here
        // and is exercised over a real, nested title in the heavy corpus lane
        // (`praxis-corpus-tests`).
        let archive = project_archive(&UsCode::sample());
        let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                assert_eq!(kind, COMPOSES_REL, "the only projected edge is Parthood");
                let parent = target
                    .local_name()
                    .expect("a Composes edge is a same-archive Parthood edge");
                assert!(
                    declared.contains(parent),
                    "Composes edge {}--part-of-->{parent} names an undeclared node",
                    n.name
                );
            }
        }
    }

    #[test]
    fn the_projected_archive_materializes_into_a_runtime_ontology() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::ontology::materialize;
        // The whole point: the projection is a valid GENERIC ontology a
        // source-agnostic engine reasons over — it materializes (referential
        // closure holds) and folds the Parthood mereology into a closure.
        let archive = project_archive(&UsCode::sample());
        let onto = materialize(archive, OntologyName::new_static("us_code"))
            .expect("the USC projection materializes");
        assert!(onto.archive().nodes.len() > 1);
    }
}
