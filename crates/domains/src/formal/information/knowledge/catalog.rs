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

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::ontology::Staging;

use crate::applied::data_provisioning::registry::data_sources;
use crate::applied::data_provisioning::source_role::functor::{is_chat_loadable, source_role};
use crate::applied::data_provisioning::source_role::ontology::{SourceRoleConcept, role_name};
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
        ResidentSource,
        EagerSource,
        ElectiveSource,
        DerivedSource,
    ],

    labels: {
        KnowledgeBoundary: ("en", "Knowledge boundary", "The line between what the system has materialized and what it has only registered — its known unknowns (Nelson & Narens 1990 meta-level model of the object-level)."),
        LoadedSource: ("en", "Loaded source", "A registered source the runtime has materialized into a live ontology (object-level, instantiated)."),
        AvailableSource: ("en", "Available source", "A registered source not yet materialized — a known unknown the meta-level can choose to load."),
        Monitoring: ("en", "Monitoring", "The object→meta information flow: the meta-level observes which sources are loaded (Nelson & Narens 1990)."),
        Control: ("en", "Control", "The meta→object information flow: the meta-level loads an available source (Nelson & Narens 1990). The 'load' action."),
        ResidentSource: ("en", "Resident source", "A loaded source the deployment ships as part of what it IS — installed at construction, never obtained by a control act, and so not releasable by one."),
        EagerSource: ("en", "Eager source", "A loaded source this deployment declares it fetches at startup rather than waiting to be asked for — obtained without a control act, but releasable by one."),
        ElectiveSource: ("en", "Elective source", "A loaded source present because the reader invoked control to obtain it, and which the same control may release."),
        DerivedSource: ("en", "Derived source", "A loaded source the runtime COMPOSED from sources already present — it was never obtained, so no control act releases it; releasing its basis removes it."),
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
        // Residency refines the loaded set by HOW each member arrived — the
        // three ways a source can come to be loaded.
        (ResidentSource, LoadedSource, SpecializesAsResidency),
        (EagerSource, LoadedSource, SpecializesAsResidency),
        (ElectiveSource, LoadedSource, SpecializesAsResidency),
        (DerivedSource, LoadedSource, SpecializesAsResidency),
        // …and that is what decides whether control can reverse the arrival.
        // Nelson & Narens' control flow is not total over the loaded set: a
        // resident source was never acquired by a control act, so there is no
        // control act that releases it. Modelling this is what keeps a UI from
        // offering an unload that cannot be undone.
        //
        // A DERIVED source is outside the control flow for the same reason and
        // one more: it is a FUNCTION of the loaded set, so "release it" is not
        // a state the object level can occupy while its basis is present — the
        // next monitoring pass would re-derive it. Control releases the basis,
        // and the derivative follows.
        (Control, EagerSource, Releases),
        (Control, ElectiveSource, Releases),
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

/// **How a loaded source came to be in the working set** — and therefore
/// whether the reader may put it back down.
///
/// This is a different question from [`Staging`], which reports how a source
/// *arrived* (baked into the binary vs. fetched at runtime). The two are
/// independent: an ontology whose bytes ship inside the binary may still be
/// there because the reader asked for it. Reading `Staging` as a proxy for
/// releasability conflates the medium with the choice, and the failure mode is
/// concrete — a resident base tagged as though it had been fetched gets offered
/// an unload control that no load control can reverse, because the reader never
/// invoked one to obtain it.
///
/// The partition is the deployment's, declared as data — the embedded
/// manifest's `default_loaded` flag and the eager-residency list — rather than
/// inferred from what a source is called. The one exception is
/// [`Residency::Derived`], which a host recognises by the canonical name its own
/// producer mints from; that is identity equality, not a guess about a name's
/// shape, but it is an exception and this comment used to deny there were any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Residency {
    /// Installed at construction — part of what the deployment *is*.
    /// [`SourceAvailability::Loaded`] before the reader does anything.
    Resident,
    /// Fetched at startup because this deployment declares it eager: obtained
    /// without a control act, but releasable by one.
    Eager,
    /// Loaded because the reader invoked control to obtain it.
    Elective,
    /// Composed by the runtime from sources already loaded — obtained by no
    /// control act, and not releasable by one. It is a FUNCTION of the loaded
    /// set: releasing what it was derived FROM is what removes it.
    Derived,
}

impl Residency {
    /// Lowercase wire label used by the self-model JSON surface.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Resident => "resident",
            Self::Eager => "eager",
            Self::Elective => "elective",
            Self::Derived => "derived",
        }
    }

    /// Whether a control act can *release* this source — the question a host
    /// asks before offering an unload affordance.
    ///
    /// False for [`Residency::Resident`]: control never acquired it, so no
    /// control releases it. An [`Residency::Eager`] source is releasable even
    /// though the reader did not ask for it — the deployment fetched it on
    /// their behalf, and a reader who does not want a large corpus resident
    /// must be able to say so.
    ///
    /// Also false for [`Residency::Derived`], on a stronger argument: it is not
    /// merely un-acquired but not independently *held*. It is recomputed from
    /// the loaded set, so an unload control over it would either be undone at
    /// once by the next derivation or, worse, appear to succeed while the
    /// knowledge it summarizes is still loaded — an affordance that misreports
    /// the system's own state.
    pub fn is_releasable(&self) -> bool {
        !matches!(self, Self::Resident | Self::Derived)
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
    /// How this source came to be loaded — see [`Residency`]. Carried
    /// separately from `staging` because releasability follows from the choice,
    /// not from the medium.
    pub residency: Residency,
    pub concepts: usize,
    pub morphisms: usize,
}

impl LoadedRef {
    pub fn new(
        name: impl Into<String>,
        staging: Staging,
        residency: Residency,
        concepts: usize,
        morphisms: usize,
    ) -> Self {
        Self {
            name: name.into(),
            staging,
            residency,
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
    /// The source's `prov:Role` (the `SourceRole` concept name) — the
    /// activity that consumes it. Every catalog row is `ChatKnowledge`
    /// (that is the filter the catalog applies); the field is carried so
    /// the self-model surface can name each row's role explicitly rather
    /// than leaving it implicit.
    pub role: String,
    /// Provenance line for display (description, falling back to URL).
    pub citation: String,
    pub availability: SourceAvailability,
    /// The staging a loaded source arrived through; `None` while available.
    pub staging: Option<Staging>,
    /// How a loaded source came to be in the working set, which is what decides
    /// whether a host may offer to release it; `None` while available (nothing
    /// to release). See [`Residency`].
    pub residency: Option<Residency>,
    /// Concept / morphism counts when loaded; `0` while available.
    pub concepts: usize,
    pub morphisms: usize,
}

/// Build the catalog: every **chat-loadable** registered source tagged
/// Loaded/Available by joining [`data_sources`] (the registry) against the
/// runtime's reported loaded set (the *monitoring* input), PLUS every loaded
/// ontology that is NOT a registered source (an embedded or uploaded `.prx`) —
/// by its OntologyName (doc §3). So the catalog reflects EVERY loaded source,
/// not only the registry: a load the registry never heard of is still part of
/// the live knowledge boundary, not dropped silently.
///
/// **The knowledge boundary is the `ChatKnowledge` role, not the whole
/// registry.** The registry also hash-pins `DecoderInput` sources (schemas,
/// glyph lists, test suites, the language-engine substrate) and
/// `NotYetLoadable` sources (PDF legal corpora with no decoder yet). Those are
/// legitimate managed sources but a chat user can never load them AS
/// knowledge, so counting them inflated the "N of M" denominator (the review
/// doc §2.4 bug: 2/47 with an unreachable 47). The catalog filters through
/// [`is_chat_loadable`] — the ontology query `IsChatLoadable ∘
/// SourceKindToRole` — so the denominator is the reasoner-reachable set. The
/// excluded roles stay in the provisioning registry, just not on the
/// knowledge boundary.
///
/// Generic over source kind — it reasons only about registry identity, role
/// (via the functor), and load membership, never about what a particular
/// source *is* by name.
pub fn source_catalog(loaded: &[LoadedRef]) -> Vec<SourceStatus> {
    let registered: BTreeSet<&str> = data_sources().iter().map(|e| e.name.as_str()).collect();

    let mut catalog: Vec<SourceStatus> = data_sources()
        .iter()
        .filter(|entry| is_chat_loadable(entry))
        .map(|entry| {
            let hit = loaded.iter().find(|l| l.name == entry.name);
            let (availability, staging, residency, concepts, morphisms) = match hit {
                Some(l) => (
                    SourceAvailability::Loaded,
                    Some(l.staging),
                    Some(l.residency),
                    l.concepts,
                    l.morphisms,
                ),
                None => (SourceAvailability::Available, None, None, 0, 0),
            };
            SourceStatus {
                name: entry.name.clone(),
                version: entry.version.clone(),
                kind: concept_name(entry.kind).to_string(),
                role: role_name(source_role(entry.kind)).to_string(),
                citation: entry
                    .description
                    .clone()
                    .unwrap_or_else(|| entry.url.clone()),
                availability,
                staging,
                residency,
                concepts,
                morphisms,
            }
        })
        .collect();

    // §3: loaded-but-UNREGISTERED ontologies — included by OntologyName so the
    // sources panel is complete. Always `Loaded` (they exist only because loaded);
    // no version/citation/kind from a registry it isn't in.
    for l in loaded {
        if !registered.contains(l.name.as_str()) {
            catalog.push(SourceStatus {
                name: l.name.clone(),
                version: String::new(),
                kind: "Loaded ontology".to_string(),
                // A materialized-and-loaded ontology is, by definition, chat
                // knowledge — the reasoner already holds it.
                role: role_name(SourceRoleConcept::ChatKnowledge).to_string(),
                citation: "Loaded at runtime — content-addressed .prx".to_string(),
                availability: SourceAvailability::Loaded,
                staging: Some(l.staging),
                residency: Some(l.residency),
                concepts: l.concepts,
                morphisms: l.morphisms,
            });
        }
    }
    catalog
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn catalog_covers_every_chat_loadable_source() {
        // The knowledge boundary is the ChatKnowledge role, not the whole
        // registry (review doc §2.4). With nothing loaded, the catalog must
        // enumerate exactly the chat-loadable registered sources — the
        // reasoner-reachable denominator, NOT all 47 registered sources
        // (which include DecoderInput schemas/glyph-lists and NotYetLoadable
        // PDF legal corpora).
        let catalog = source_catalog(&[]);
        let chat_loadable = data_sources()
            .iter()
            .filter(|e| is_chat_loadable(e))
            .count();
        assert_eq!(
            catalog.len(),
            chat_loadable,
            "catalog must enumerate every chat-loadable source"
        );
        // And it is a proper subset — the decoder/pending sources are excluded.
        assert!(
            chat_loadable < data_sources().len(),
            "the registry carries non-chat-loadable sources that must be excluded"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_reported_source_is_marked_loaded() {
        // Pick a real CHAT-LOADABLE registry name to report as loaded — a
        // DecoderInput/NotYetLoadable source is (correctly) absent from the
        // catalog, so it could not be found below.
        let some = data_sources()
            .iter()
            .find(|e| is_chat_loadable(e))
            .expect("registry has a chat-loadable source")
            .name
            .clone();
        let loaded = [LoadedRef::new(
            some.clone(),
            Staging::Embedded,
            Residency::Resident,
            42,
            7,
        )];
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

    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn residency_not_staging_decides_what_control_can_release() {
        // THE property that governs an unload affordance. Staging and residency
        // are independent, and a host that reads the first for the second gets
        // it wrong in exactly one direction: a resident base whose bytes ship
        // embedded looks identical to an embedded-but-elective load, so it is
        // offered a release that no acquisition can undo — the reader never
        // performed one.
        //
        // The pairs below are all reachable in the shipped deployment: the
        // caregiving bases (embedded + resident), the one-click demo (embedded
        // + elective, because the reader clicked), a prefetched title (async +
        // eager) and a reader-loaded title (async + elective).
        for (staging, residency, releasable) in [
            (Staging::Embedded, Residency::Resident, false),
            (Staging::Embedded, Residency::Elective, true),
            (Staging::Async, Residency::Eager, true),
            (Staging::Async, Residency::Elective, true),
            // …and the derived title lexicon, composed in-process from the
            // titles already loaded: never acquired, so never released.
            (Staging::Composed, Residency::Derived, false),
        ] {
            assert_eq!(
                residency.is_releasable(),
                releasable,
                "{} + {} must be {}releasable — releasability follows residency, \
                 never staging",
                staging_label(staging),
                residency.label(),
                if releasable { "" } else { "un" },
            );
        }

        // Stated as the invariant rather than a table of cases: staging alone
        // cannot decide it, because both staging values appear on both sides.
        assert!(
            !Residency::Resident.is_releasable(),
            "a resident source is what the deployment IS; control never \
             acquired it, so no control releases it"
        );
        assert!(
            Residency::Eager.is_releasable(),
            "a reader who does not want a prefetched corpus resident must be \
             able to say so — the deployment fetched it on their behalf, which \
             is not the same as their having asked for it"
        );
        assert!(
            !Residency::Derived.is_releasable(),
            "a derived source is a FUNCTION of the loaded set — an unload that \
             the next derivation immediately undoes would misreport the \
             system's own state; control releases the basis instead"
        );
    }

    /// The DECLARED edges decide releasability — `is_releasable` may not drift
    /// from them.
    ///
    /// The ontology above declares `(Control, EagerSource, Releases)` and
    /// `(Control, ElectiveSource, Releases)` and deliberately declares no such
    /// edge for `ResidentSource` or `DerivedSource`. Until this existed, that
    /// declaration was inert: adding or deleting a `Releases` edge changed no
    /// behaviour and failed no test, so the Rust `matches!` was the real rule
    /// and the ontology was commentary. This makes them one statement —
    /// disagree, and the disagreement is what fails.
    ///
    /// Cross-checked rather than derived: `is_releasable` stays a cheap `const`
    /// -shaped match on the hot path, and this proves the match and the edges
    /// say the same thing. That is an independent second statement, not the
    /// tautology of asserting a function equals itself.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn the_declared_releases_edges_decide_releasability() {
        use pr4xis::category::Category;

        let releasable_by_declaration = |c: KnowledgeBoundaryConcept| {
            KnowledgeBoundaryCategory::morphisms().iter().any(|m| {
                m.from == KnowledgeBoundaryConcept::Control
                    && m.to == c
                    && m.kind == KnowledgeBoundaryRelationKind::Releases
            })
        };

        for (residency, concept) in [
            (
                Residency::Resident,
                KnowledgeBoundaryConcept::ResidentSource,
            ),
            (Residency::Eager, KnowledgeBoundaryConcept::EagerSource),
            (
                Residency::Elective,
                KnowledgeBoundaryConcept::ElectiveSource,
            ),
            (Residency::Derived, KnowledgeBoundaryConcept::DerivedSource),
        ] {
            assert_eq!(
                residency.is_releasable(),
                releasable_by_declaration(concept),
                "{} disagrees with the ontology: is_releasable() says {}, but a \
                 `Control --Releases--> {concept:?}` edge is {}declared",
                residency.label(),
                residency.is_releasable(),
                if releasable_by_declaration(concept) {
                    ""
                } else {
                    "not "
                },
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn residency_labels_are_stable() {
        // Wire labels the host reads; renaming one silently changes an
        // affordance, so they are pinned here alongside `staging_label`'s.
        assert_eq!(Residency::Resident.label(), "resident");
        assert_eq!(Residency::Eager.label(), "eager");
        assert_eq!(Residency::Elective.label(), "elective");
        assert_eq!(Residency::Derived.label(), "derived");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_unregistered_loaded_ontology_appears_in_the_catalog() {
        // §3: a LOADED ontology the registry never heard of (an embedded or
        // uploaded `.prx`) is INCLUDED by its OntologyName — the catalog reflects
        // every loaded source, not only registered ones (it was silently dropped
        // before). Its presence is grounded in the LOAD, not a fabricated source.
        let loaded = [LoadedRef::new(
            "an-uploaded-prx",
            Staging::Async,
            Residency::Elective,
            5,
            3,
        )];
        let catalog = source_catalog(&loaded);

        let entry = catalog
            .iter()
            .find(|s| s.name == "an-uploaded-prx")
            .expect("the unregistered loaded ontology appears in the catalog");
        assert_eq!(entry.availability, SourceAvailability::Loaded);
        assert_eq!(entry.concepts, 5);
        assert_eq!(entry.morphisms, 3);

        // The registered sources (none loaded here) stay Available — the
        // unregistered load does not pollute or mislabel them.
        assert!(
            catalog
                .iter()
                .filter(|s| s.name != "an-uploaded-prx")
                .all(|s| s.availability == SourceAvailability::Available),
            "registered sources are unaffected"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn availability_labels_are_stable() {
        assert_eq!(SourceAvailability::Loaded.label(), "loaded");
        assert_eq!(SourceAvailability::Available.label(), "available");
        assert!(SourceAvailability::Loaded.is_loaded());
        assert!(!SourceAvailability::Available.is_loaded());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn staging_labels_are_stable() {
        assert_eq!(staging_label(Staging::Embedded), "embedded");
        assert_eq!(staging_label(Staging::Async), "async");
        assert_eq!(staging_label(Staging::Mmap), "mmap");
        assert_eq!(staging_label(Staging::Composed), "composed");
    }
}
