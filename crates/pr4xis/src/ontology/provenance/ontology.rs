//! Provenance — the substrate ontology grounding `ontology/meta.rs`.
//!
//! # Why this ontology lives in core
//!
//! `Provenance` (name, description, citation, module_path) is a
//! per-instance provenance record. Citation points to literature;
//! module_path points to code. These are PROV-O (W3C 2013) concepts:
//! Artifact, Source, Agent, Activity. Core's meta structs are instances
//! of those concepts — the ontology naming them belongs in core too,
//! alongside the machinery it grounds.
//!
//! # PROV-O core
//!
//! W3C PROV-O (2013) §3.1 defines three core concepts:
//! - **Entity / Artifact** — a thing with provenance (an ontology, a
//!   commit, a concept, a citation).
//! - **Activity** — something that generates or transforms artifacts
//!   (a derivation, a build, a commit action).
//! - **Agent** — bearer of responsibility (a person, a tool, a CI system).
//!
//! Extended with version-control and academic-source concepts relevant
//! to pr4xis: Repository, Commit, Branch, Tag, Version, Source, Citation.
//!
//! Literature:
//! - W3C PROV-O (2013) — <https://www.w3.org/TR/prov-o/>
//! - W3C PROV-DM (2013) — <https://www.w3.org/TR/prov-dm/>

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Provenance",
    source: "W3C PROV-O (2013) https://www.w3.org/TR/prov-o/; W3C PROV-DM (2013)",

    concepts: [
        // === PROV-O core (W3C §3.1) ===
        Artifact,
        Activity,
        Agent,

        // === Version control ===
        Repository,
        Commit,
        Branch,
        Tag,
        Version,

        // === Knowledge sources ===
        Source,
        Citation,
    ],

    labels: {
        Artifact: ("en", "Artifact (prov:Entity)",
            "W3C PROV-O §3.1: a thing with provenance — an ontology, a dataset, a concept, a document. The primary subject of provenance statements."),
        Activity: ("en", "Activity (prov:Activity)",
            "W3C PROV-O §3.1: something that generates or transforms artifacts — a derivation, a build, a commit."),
        Agent: ("en", "Agent (prov:Agent)",
            "W3C PROV-O §3.1: bearer of responsibility for an artifact or activity — a person, a tool, a CI system."),

        Repository: ("en", "Repository",
            "A version-controlled collection of artifacts and their history."),
        Commit: ("en", "Commit",
            "An atomic change to a repository — a specific version of its contents."),
        Branch: ("en", "Branch",
            "A named line of development pointing to a commit."),
        Tag: ("en", "Tag",
            "A named marker at a specific commit (e.g. `v1.0`)."),
        Version: ("en", "Version",
            "A semantic version identifier for an artifact."),

        Source: ("en", "Source",
            "An academic paper, specification, or other primary reference that defines concepts."),
        Citation: ("en", "Citation",
            "A reference to a specific section or definition within a Source."),
    },

    is_a: [
        // Version-control concepts are artifacts
        (Repository, Artifact),
        (Commit, Artifact),
        (Branch, Artifact),
        (Tag, Artifact),
        (Version, Artifact),

        // Knowledge sources are artifacts
        (Source, Artifact),
        (Citation, Artifact),
    ],

    has_a: [
        // Composition: a repository has commits, branches, tags.
        (Repository, Commit),
        (Repository, Branch),
        (Repository, Tag),

        // A version identifies an artifact.
        (Version, Artifact),

        // A citation references a source.
        (Citation, Source),
    ],
}

/// Whether a provenance concept is from the W3C PROV-O core (vs. an
/// extension concept — version-control or knowledge-source).
#[derive(Debug, Clone)]
pub struct IsProvOCore;

impl Quality for IsProvOCore {
    type Individual = ProvenanceConcept;
    type Value = bool;

    fn get(&self, individual: &ProvenanceConcept) -> Option<bool> {
        use ProvenanceConcept as P;
        Some(matches!(individual, P::Artifact | P::Activity | P::Agent))
    }
}

impl Ontology for ProvenanceOntology {
    type Cat = ProvenanceCategory;
    type Qual = IsProvOCore;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<ProvenanceCategory>();
    }

    #[test]
    fn ontology_validates() {
        ProvenanceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
