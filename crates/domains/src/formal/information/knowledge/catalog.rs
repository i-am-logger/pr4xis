//! The source catalog — the system's model of the boundary between what
//! it *knows* and what it *could know*.
//!
//! The [`crate::formal::information::knowledge::SelfModelInstance`] (the
//! eigenform of self-observation) models the *loaded* ontologies. But a
//! genuinely meta-aware system also models its **knowledge boundary**:
//! the registered sources it has not yet materialized — its *known
//! unknowns*. This module supplies that model.
//!
//! ## Literature — metacognitive monitoring & control
//!
//! Nelson & Narens (1990), "Metamemory: A Theoretical Framework and New
//! Findings," *Psychology of Learning and Motivation* 26:125–173, split
//! cognition into a **meta-level** holding a *dynamic model* of an
//! **object-level**, connected by two flows:
//!
//! - **Monitoring** (object → meta): the meta-level observes the state of
//!   the object-level. Here: which registered sources are loaded, and
//!   their concept / morphism counts.
//! - **Control** (meta → object): the meta-level modifies the
//!   object-level. Here: *loading* a source — moving it from
//!   [`SourceAvailability::Available`] to [`SourceAvailability::Loaded`].
//!
//! Cox (2005), "Metacognition in computation: A selected research
//! review," *Artificial Intelligence* 169(2):104–141, carries the same
//! monitoring/control structure into computational systems. Smith (1984),
//! "Reflection and Semantics in Lisp," POPL, grounds a system's access to
//! its own structure (already cited by the self-model axioms).
//!
//! The catalog is the meta-level's model of the object-level's
//! *instantiation status*. It is built generically from the registry
//! (`data_sources()` — every `[sources.*]` entry in praxis.toml); it has
//! no knowledge of any particular source kind. A source is `Loaded` iff
//! the runtime reports it in its loaded set; otherwise it is `Available`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::ontology::Staging;

use crate::applied::data_provisioning::registry::data_sources;
use crate::formal::meta::source_taxonomy::ontology::concept_name;

pr4xis::ontology! {
    name: "KnowledgeBoundary",
    source: "Nelson & Narens (1990) Metamemory: A Theoretical Framework and New Findings, Psychology of Learning and Motivation 26:125-173; Cox (2005) Metacognition in computation: A selected research review, Artificial Intelligence 169(2):104-141; Smith (1984) Reflection and Semantics in Lisp, POPL",

    concepts: [
        KnowledgeBoundary,
        LoadedSource,
        AvailableSource,
        Monitoring,
        Control,
    ],

    labels: {
        KnowledgeBoundary: ("en", "Knowledge boundary", "The line between what the system has materialized and what it has only registered — its known unknowns (Nelson & Narens 1990 meta-level model of the object-level)."),
        LoadedSource: ("en", "Loaded source", "A registered source the runtime has materialized into a live ontology (object-level, instantiated)."),
        AvailableSource: ("en", "Available source", "A registered source not yet materialized — a known unknown the meta-level can choose to load."),
        Monitoring: ("en", "Monitoring", "The object→meta information flow: the meta-level observes which sources are loaded (Nelson & Narens 1990)."),
        Control: ("en", "Control", "The meta→object information flow: the meta-level loads an available source (Nelson & Narens 1990). The 'load' action."),
    },

    edges: [
        // The boundary partitions the catalog.
        (KnowledgeBoundary, LoadedSource, Separates),
        (KnowledgeBoundary, AvailableSource, Separates),
        // Monitoring observes the object-level (object → meta).
        (Monitoring, LoadedSource, Observes),
        // Control acts on the object-level (meta → object): loading.
        (Control, AvailableSource, Loads),
        // The state transition control effects.
        (AvailableSource, LoadedSource, BecomesViaControl),
    ],
}

/// Instantiation status of a registered source — the meta-level's tag for
/// one object-level source.
///
/// `Available` (registered, not materialized) is the *known unknown*;
/// `Loaded` (materialized into a live ontology) is the *known known*. The
/// `Available → Loaded` transition is the Nelson-Narens *control*
/// operation ("load").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceAvailability {
    Available,
    Loaded,
}

impl SourceAvailability {
    /// Lowercase wire label used by the self-model JSON surface.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Loaded => "loaded",
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded)
    }
}

/// A loaded source as reported by the runtime — the *monitoring* input.
///
/// `name` is the registry primary key (matches `[sources.<name>]` in
/// praxis.toml), so it joins against [`data_sources`].
#[derive(Debug, Clone)]
pub struct LoadedRef {
    pub name: String,
    pub staging: Staging,
    pub concepts: usize,
    pub morphisms: usize,
}

impl LoadedRef {
    pub fn new(
        name: impl Into<String>,
        staging: Staging,
        concepts: usize,
        morphisms: usize,
    ) -> Self {
        Self {
            name: name.into(),
            staging,
            concepts,
            morphisms,
        }
    }
}

/// One entry in the source catalog — a registered source tagged with its
/// instantiation status.
#[derive(Debug, Clone)]
pub struct SourceStatus {
    /// Registry primary key.
    pub name: String,
    /// Publication identifier (calendar year, edition, …).
    pub version: String,
    /// Semantic kind, as the registry taxonomy concept's name.
    pub kind: String,
    /// Provenance line for display (description, falling back to URL).
    pub citation: String,
    pub availability: SourceAvailability,
    /// The staging a loaded source arrived through; `None` while available.
    pub staging: Option<Staging>,
    /// Concept / morphism counts when loaded; `0` while available.
    pub concepts: usize,
    pub morphisms: usize,
}

/// Build the catalog: every registered source tagged Loaded/Available by
/// joining [`data_sources`] (the full registry) against the runtime's
/// reported loaded set (the *monitoring* input).
///
/// Generic over source kind — it reasons only about registry identity and
/// load membership, never about what a particular source *is*.
pub fn source_catalog(loaded: &[LoadedRef]) -> Vec<SourceStatus> {
    data_sources()
        .iter()
        .map(|entry| {
            let hit = loaded.iter().find(|l| l.name == entry.name);
            let (availability, staging, concepts, morphisms) = match hit {
                Some(l) => (
                    SourceAvailability::Loaded,
                    Some(l.staging),
                    l.concepts,
                    l.morphisms,
                ),
                None => (SourceAvailability::Available, None, 0, 0),
            };
            SourceStatus {
                name: entry.name.clone(),
                version: entry.version.clone(),
                kind: concept_name(entry.kind).to_string(),
                citation: entry
                    .description
                    .clone()
                    .unwrap_or_else(|| entry.url.clone()),
                availability,
                staging,
                concepts,
                morphisms,
            }
        })
        .collect()
}

/// Map a [`Staging`] to its lowercase wire label.
pub fn staging_label(staging: Staging) -> &'static str {
    match staging {
        Staging::Embedded => "embedded",
        Staging::Async => "async",
        Staging::Mmap => "mmap",
        Staging::Composed => "composed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_every_registered_source() {
        let catalog = source_catalog(&[]);
        assert_eq!(
            catalog.len(),
            data_sources().len(),
            "catalog must enumerate every registered source"
        );
    }

    #[test]
    fn empty_loaded_set_means_everything_available() {
        // With nothing reported loaded, the whole catalog is the
        // knowledge boundary — every source is a known unknown.
        let catalog = source_catalog(&[]);
        assert!(!catalog.is_empty(), "registry is non-empty");
        assert!(
            catalog
                .iter()
                .all(|s| s.availability == SourceAvailability::Available),
            "no source is loaded when the loaded set is empty"
        );
        assert!(catalog.iter().all(|s| s.staging.is_none()));
    }

    #[test]
    fn a_reported_source_is_marked_loaded() {
        // Pick a real registry name to report as loaded.
        let some = data_sources()
            .first()
            .expect("registry non-empty")
            .name
            .clone();
        let loaded = [LoadedRef::new(some.clone(), Staging::Embedded, 42, 7)];
        let catalog = source_catalog(&loaded);
        let hit = catalog
            .iter()
            .find(|s| s.name == some)
            .expect("reported source present in catalog");
        assert_eq!(hit.availability, SourceAvailability::Loaded);
        assert_eq!(hit.staging, Some(Staging::Embedded));
        assert_eq!(hit.concepts, 42);
        assert_eq!(hit.morphisms, 7);
        // Every other source stays available — the boundary holds.
        assert!(
            catalog
                .iter()
                .filter(|s| s.name != some)
                .all(|s| s.availability == SourceAvailability::Available)
        );
    }

    #[test]
    fn an_unregistered_loaded_name_does_not_invent_a_catalog_entry() {
        // Monitoring a name that isn't registered must not fabricate a
        // source — the catalog is grounded in the registry.
        let loaded = [LoadedRef::new(
            "definitely-not-a-registered-source",
            Staging::Async,
            1,
            1,
        )];
        let catalog = source_catalog(&loaded);
        assert!(
            catalog
                .iter()
                .all(|s| s.availability == SourceAvailability::Available)
        );
    }

    #[test]
    fn availability_labels_are_stable() {
        assert_eq!(SourceAvailability::Loaded.label(), "loaded");
        assert_eq!(SourceAvailability::Available.label(), "available");
        assert!(SourceAvailability::Loaded.is_loaded());
        assert!(!SourceAvailability::Available.is_loaded());
    }

    #[test]
    fn staging_labels_are_stable() {
        assert_eq!(staging_label(Staging::Embedded), "embedded");
        assert_eq!(staging_label(Staging::Async), "async");
        assert_eq!(staging_label(Staging::Mmap), "mmap");
        assert_eq!(staging_label(Staging::Composed), "composed");
    }
}
