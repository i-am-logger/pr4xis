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
//! FUNCTOR carried AS `.prx` DATA — the committed
//! `data/projections/usc_functor.prx`, loaded fail-closed against its baked root
//! and interpreted by the one runtime primitive [`apply`](pr4xis_runtime::apply::apply).
//! [`usc_runtime_ontology`](crate::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology)
//! is the whole pipeline (`project → apply → materialize`),
//! the verbatim shape of `owl_runtime_ontology` (the pattern the English
//! bridge pioneered; English itself now proves its functor archive-level,
//! without materializing). Parthood is a canonically
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
use alloc::vec::Vec;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::Connection;
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::ontology::{MaterializeError, RuntimeOntology, materialize};

use super::UsCode;
use super::section_aux::UscSubdivision;
use crate::cognitive::linguistics::english::bridge::form_atom;

/// The RAW USLM node tag of a section in the SOURCE archive — the `<section>`
/// element name the projector emits, before the committed `usc_functor.prx`
/// relabels it to [`SECTION_KIND`]. (Subdivisions already carry their raw USLM tag via
/// `sub.kind.tag()`, so only the section root needed a name.)
pub const SECTION_TAG: &str = "section";

/// The praxis node kind a section relabels to — appears ONLY in the functor DATA,
/// never baked into the structural projection.
pub const SECTION_KIND: &str = "Section";

/// The RAW USLM relation of the Composes hierarchy in the SOURCE archive — a
/// subdivision Composes INTO its parent. The schema generator the committed
/// `usc_functor.prx` maps to [`PARTHOOD_REL`]; emitted raw so the
/// mereological reading is loaded data, not baked into the projector.
pub const COMPOSES_REL: &str = "Composes";

/// The praxis relation kind a Composes edge relabels to — Parthood (Casati &
/// Varzi 1999), one of the canonically transitive kinds
/// [`materialize`](pr4xis_runtime::ontology) folds, so the projection's mereology
/// is a real closure (a clause is transitively part-of its section). Appears ONLY
/// in the functor DATA.
pub const PARTHOOD_REL: &str = "Parthood";

/// The URN of the United States Code ROOT node the projection emits — the single
/// corpus-level node standing for the whole codification (1 U.S.C. § 204). It is
/// the anchor a Title/Code TYPE grounding attaches to (the sections are the
/// `Statute` instances; this root is the `Code`), so a loaded USC reaches
/// `legal_sources:Code` — not only `legal_sources:Statute` at the section level.
pub const CODE_ROOT_URN: &str = "/us/usc";

/// The RAW node kind of the [`CODE_ROOT_URN`] node — the United States Code as a
/// whole. It is NOT relabeled by `usc_functor.prx` (which only maps `section`), so
/// it reaches the grounding lens verbatim; the `usc_legal_sources_functor.prx`
/// object map keys on this kind (`code ↦ Code`). Emitted raw so the Code reading
/// is loaded functor DATA, never baked into the structural projection.
pub const CODE_ROOT_TAG: &str = "code";

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
/// structural transcription only (the praxis relabel is the committed
/// `usc_functor.prx`, loaded fail-closed).
///
/// Each section → a [`Definition`] `{kind: `[`SECTION_TAG`]`, name: urn, lexical:
/// heading}`; each subdivision → `{kind: its raw USLM tag (`subsection` /
/// `paragraph` / …), name: urn, lexical: heading∣chapeau∣content, edges:
/// [(`[`COMPOSES_REL`]`, parent_urn)]}`. Every Composes target is a declared
/// section/subdivision node, so the archive is referentially closed and (after
/// the functor relabels Composes→Parthood)
/// [`materialize`]s into a real mereology.
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
    // The corpus-level ROOT node: the United States Code itself (1 U.S.C. § 204).
    // Its RAW `code` kind reaches the grounding lens unrelabeled, where the
    // committed `usc_legal_sources_functor.prx` maps it to `legal_sources:Code`.
    // It carries no Composes edge (a Title/section→Code mereology is a separate,
    // heavier projection); its role here is the anchor the Code TYPE grounding
    // rides. Kept referentially closed (no outgoing edges).
    nodes.push(Definition {
        kind: CODE_ROOT_TAG.to_string(),
        name: CODE_ROOT_URN.to_string(),
        edges: Vec::new(),
        axioms: Vec::new(),
        lexical: Some("United States Code".to_string()),
    });
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

/// The committed USC → praxis projection — the `.prx` bytes the functor LIVES in
/// (Track C #203), embedded at build time. NOT a Rust literal: a connections-only
/// [`Archive`] carrying one [`Connection`] whose
/// [`Functor`](pr4xis_runtime::connection::GeneratorAction::Functor) action maps
/// `section ↦ Section`, `Composes ↦ Parthood` (Casati & Varzi 1999 — a
/// subdivision is PART-OF its parent), `heading ↦ canonicalForm`,
/// `citation ↦ otherForm`. Re-emitting it (say `Composes ↦ Containment`) re-aims
/// the projection without touching code — and without even recompiling.
const USC_FUNCTOR_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/projections/usc_functor.prx"
));

/// The trusted Merkle root of [`USC_FUNCTOR_PRX`] — the integrity pin the
/// fail-closed load checks against (file ⇔ pin coherence is asserted in tests).
const USC_FUNCTOR_ROOT_HEX: &str =
    "ec20f202804d684bc2443d1a55451eb6b1e623ee25c65925614b9eef65f5445a";

/// Load the USC → praxis functor from its committed `.prx` ([`USC_FUNCTOR_PRX`]) —
/// FAIL-CLOSED: the embedded bytes are admitted only if they re-derive to
/// [`USC_FUNCTOR_ROOT_HEX`], so a tampered or stale projection is refused, never
/// silently mis-applied. Reuses the kernel [`load`](pr4xis_runtime::load::load);
/// no new runtime API. A functor's whole content is its finite action on the
/// schema's generators (Fong & Spivak *Seven Sketches* Ch. 3), interpreted by
/// [`apply`](pr4xis_runtime::apply::apply) over a raw [`project_archive`] source. A load failure here is a
/// build-time invariant violation (the bytes ship embedded in the binary).
fn usc_functor() -> Connection {
    let root = ContentAddress::from_hex(USC_FUNCTOR_ROOT_HEX)
        .expect("USC_FUNCTOR_ROOT_HEX is valid 64-hex");
    let archive = pr4xis_runtime::load::load(USC_FUNCTOR_PRX, root)
        .expect("committed usc_functor.prx must load against its baked root");
    archive
        .connections
        .into_iter()
        .next()
        .expect("usc_functor.prx carries exactly one Connection")
}

/// The committed USC → LegalSources TYPE-grounding functor — the `.prx` bytes a
/// grounding functor LIVES in, the TWIN of [`USC_FUNCTOR_PRX`]. A
/// connections-only [`Archive`] carrying one [`Connection`] whose
/// [`Functor`](GeneratorAction::Functor) action maps `Section ↦ Statute` and the
/// Code root `code ↦ Code` (`map_object`) and the typing relation
/// `instantiates ↦ Subsumption` (`map_morphism`, the reachability kind the typing
/// edge asserts). It is NOT applied by [`apply`] (that relabels a node's own
/// kind); it is APPENDED to the USC archive as data ([`usc_archive`]) and read by
/// the general [`ground_declared`](crate::formal::meta::grounding::ground_declared)
/// step, which MINTS a cross-ontology
/// [`EdgeTarget::Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded)
/// edge per typed node into the LOADED LegalSources peer's atom. Re-emitting it
/// (say `Section ↦ Regulation`) re-aims the grounding without touching code.
///
/// Grounding: LKIF-Core (Hoekstra et al. 2007) — `lkif:Statute` (a section bears
/// enacted norms), `lkif:Code` (the USC is a compilation of legislation); Salmond
/// (enacted law); 1 U.S.C. § 204.
const USC_LEGAL_SOURCES_FUNCTOR_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/projections/usc_legal_sources_functor.prx"
));

/// The trusted Merkle root of [`USC_LEGAL_SOURCES_FUNCTOR_PRX`] — the fail-closed
/// integrity pin (file ⇔ pin coherence asserted in tests; regenerate with the
/// `--ignored regenerate_usc_legal_sources_functor_prx` test and bake the printed
/// root here).
const USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX: &str =
    "65dc0286ef18d3620a167eaec7ab58f0a79beda042b337d1cc2f7c1fbace4e54";

/// Load the USC → LegalSources type-grounding functor from its committed `.prx`
/// ([`USC_LEGAL_SOURCES_FUNCTOR_PRX`]) — FAIL-CLOSED against
/// [`USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX`], the verbatim shape of [`usc_functor`].
fn usc_legal_sources_functor() -> Connection {
    let root = ContentAddress::from_hex(USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX)
        .expect("USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX is valid 64-hex");
    let archive = pr4xis_runtime::load::load(USC_LEGAL_SOURCES_FUNCTOR_PRX, root)
        .expect("committed usc_legal_sources_functor.prx must load against its baked root");
    archive
        .connections
        .into_iter()
        .next()
        .expect("usc_legal_sources_functor.prx carries exactly one Connection")
}

/// Project a loaded [`UsCode`] into the generic runtime [`Archive`], CARRYING its
/// USC→LegalSources grounding functor as DATA — the schema-relabeled provisions
/// PLUS the committed `usc_legal_sources_functor` appended as a
/// [`Connection`].
///
/// [`project_archive`] → [`apply`]`(usc_functor)` (relabel raw USLM kinds to
/// praxis kinds) → append the grounding [`Connection`]. The archive now DECLARES
/// its cross-ontology typing (`Section ↦ legal_sources:Statute`, `code ↦ Code`) as
/// data the general loader step
/// [`ground_declared`](crate::formal::meta::grounding::ground_declared) reads and
/// mints from — the SAME path any `.prx` carrying an instance functor takes. No
/// USC-specific grounding code and no `emit::<LegalSourcesCategory>()` hardcode:
/// the target atoms come from the LOADED `LegalSources` peer at grounding time.
pub fn usc_archive(usc: &UsCode) -> Archive {
    let raw = project_archive(usc);
    let mut praxis = apply(&usc_functor().action, &raw)
        .expect("the loaded usc_functor is always a Functor action, which apply interprets");
    // APPEND the grounding functor as DATA — not applied here (that would relabel a
    // node's own kind); the general `ground_declared` step interprets it against
    // the loaded LegalSources peer, minting the type edges. Re-emitting the
    // committed `.prx` (say `Section ↦ Regulation`) re-aims the grounding.
    praxis.connections.push(usc_legal_sources_functor());
    praxis
}

/// Bridge a loaded [`UsCode`] into a generic [`RuntimeOntology`] — [`usc_archive`]
/// (project → apply → append the grounding functor as data) →
/// [`materialize`].
///
/// The materialized ontology CARRIES its grounding `Connection`, but its cross-
/// ontology type edges are NOT minted here — that is the loader's general
/// [`ground_loaded_set`](crate::formal::meta::grounding::ground_loaded_set) step,
/// which grounds this ontology against the LOADED `LegalSources` peer (the same
/// step that grounds any instance-functor `.prx`). This is what makes USC a plain
/// special case of the general grounding mechanism rather than a hardcoded path.
///
/// `apply` cannot fail here (the loaded `usc_functor` is always a `Functor`
/// action); materialization can still fail closed (a codec error on the root),
/// propagated typed.
pub fn usc_runtime_ontology(
    usc: &UsCode,
    name: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    materialize(usc_archive(usc), name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::bridge::FORM_KIND;
    use alloc::collections::BTreeSet;
    use pr4xis_runtime::connection::GeneratorAction;

    #[pr4xis::praxis_value(Verifiable)]
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
        // CONCEPTS (non-Form nodes) — one per section + per subdivision, PLUS the
        // single corpus-level Code root node ([`CODE_ROOT_URN`]); the §9 Form atoms
        // (heading / citation surfaces) are excluded.
        let concepts = archive.nodes.iter().filter(|n| n.kind != FORM_KIND).count();
        assert_eq!(
            concepts,
            expected + 1,
            "one concept node per section + subdivision, plus the Code root"
        );
        // The Code root node is present, carrying the raw `code` kind (the grounding
        // functor maps it to legal_sources:Code; the structural projection emits it
        // raw).
        assert!(
            archive
                .nodes
                .iter()
                .any(|n| n.kind == CODE_ROOT_TAG && n.name == CODE_ROOT_URN),
            "the projection emits the United States Code root node"
        );
        assert!(archive.connections.is_empty());
        // The structural projection carries the RAW USLM section tag — never the
        // praxis "Section" kind (that relabel is the functor's job).
        assert!(archive.nodes.iter().any(|n| n.kind == SECTION_TAG));
        assert!(archive.nodes.iter().all(|n| !n.name.is_empty()));
    }

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable, Extensible)]
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
        let praxis = apply(&usc_functor().action, &raw).expect("Functor applies");
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

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn the_functor_loads_from_its_committed_prx_fail_closed() {
        // The projection LIVES in `usc_functor.prx` (Track C #203): the loader
        // admits the committed bytes ONLY against the baked root and yields a
        // Functor action with non-empty relabel tables. The exact rows are NOT
        // re-asserted here — that would re-smuggle the map back into code; the
        // relabel BEHAVIOR is proven by `the_functor_relabels...` above.
        let GeneratorAction::Functor {
            map_object,
            map_morphism,
        } = &usc_functor().action
        else {
            panic!("the loaded projection is a Functor action");
        };
        assert!(
            !map_object.is_empty() && !map_morphism.is_empty(),
            "the loaded functor carries non-empty relabel tables"
        );
        // File ⇔ pin coherence + fail-closed: the committed bytes re-derive to the
        // baked root, and a WRONG root is refused (no drift test needed — the pin
        // IS the integrity, there is no Rust source to drift from).
        let pin = ContentAddress::from_hex(USC_FUNCTOR_ROOT_HEX).unwrap();
        assert_eq!(
            pr4xis_runtime::load::load(USC_FUNCTOR_PRX, pin)
                .unwrap()
                .root()
                .unwrap(),
            pin,
            "the committed .prx re-derives to its baked root"
        );
        assert!(
            pr4xis_runtime::load::load(USC_FUNCTOR_PRX, ContentAddress::of(b"wrong")).is_err(),
            "a wrong root is refused — the load is fail-closed"
        );
    }

    #[pr4xis::praxis_value(Extensible)]
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

    // ---------------------------------------------------------------------------
    // The USC → LegalSources TYPE-grounding functor (committed `.prx`, twin of
    // usc_functor.prx).
    // ---------------------------------------------------------------------------

    /// The committed grounding-functor `.prx` path.
    fn committed_usc_legal_sources_functor_prx_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/projections/usc_legal_sources_functor.prx")
    }

    /// The USC → LegalSources grounding functor as a connections-only [`Archive`]
    /// — the SOURCE OF TRUTH the committed `.prx` must equal. Built from code ONLY
    /// here (the regenerate + drift guard); the runtime loads it from the committed
    /// bytes fail-closed. `map_object` types the two USC node kinds; `map_morphism`
    /// declares the reachability kind the typing edge asserts.
    fn usc_legal_sources_functor_archive() -> Archive {
        let conn = Connection {
            // The grounding-functor kind is the META-ONTOLOGY concept the loader's
            // discriminator (`is_grounding_functor_kind`) reaches to `InstanceFunctor`
            // (Spivak 2012 FDM §3) — NOT an ad-hoc "TypeGrounding" string. This is
            // what makes the general `ground_declared` step recognise it as a
            // grounding functor to mint type edges from.
            kind: "InstanceFunctor".to_string(),
            source: "us_code".to_string(),
            target: "LegalSources".to_string(),
            action: GeneratorAction::Functor {
                map_object: vec![
                    (SECTION_KIND.to_string(), "Statute".to_string()),
                    (CODE_ROOT_TAG.to_string(), "Code".to_string()),
                ],
                map_morphism: vec![("instantiates".to_string(), "Subsumption".to_string())],
            },
            laws: vec!["PreservesTyping".to_string()],
        };
        Archive {
            nodes: Vec::new(),
            connections: vec![conn],
        }
    }

    /// REGENERATE PATH (`--ignored`, WRITES): re-emit the committed
    /// `usc_legal_sources_functor.prx` from [`usc_legal_sources_functor_archive`],
    /// then PRINT the root to bake into [`USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX`].
    /// Mirrors `regenerate_praxis_registry_prx`. Run:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_usc_legal_sources_functor_prx`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_usc_legal_sources_functor_prx() {
        let archive = usc_legal_sources_functor_archive();
        let bytes = pr4xis_runtime::load::emit(&archive).expect("encode grounding functor .prx");
        let out = committed_usc_legal_sources_functor_prx_path();
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).expect("create data/projections/");
        }
        std::fs::write(&out, &bytes).expect("write usc_legal_sources_functor.prx");
        let root = archive.root().expect("root").to_hex();
        eprintln!("wrote {} ({} bytes)", out.display(), bytes.len());
        println!("USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX = {root}");
    }

    /// STALENESS GUARD (normal suite): the committed `.prx` must be a FRESH emit of
    /// the source-of-truth archive, and its baked root must match. HARD-FAILS if
    /// the functor definition drifted from the committed bytes/pin without
    /// regenerating.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn committed_usc_legal_sources_functor_prx_matches_source() {
        let archive = usc_legal_sources_functor_archive();
        let fresh = pr4xis_runtime::load::emit(&archive).expect("encode");
        let committed = std::fs::read(committed_usc_legal_sources_functor_prx_path())
            .expect("read committed usc_legal_sources_functor.prx");
        assert_eq!(
            fresh, committed,
            "committed usc_legal_sources_functor.prx is STALE — regenerate with \
             `cargo test -p pr4xis-domains -- --ignored regenerate_usc_legal_sources_functor_prx` \
             and bake the printed USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX"
        );
        assert_eq!(
            archive.root().unwrap().to_hex(),
            USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX,
            "USC_LEGAL_SOURCES_FUNCTOR_ROOT_HEX is STALE vs the committed functor"
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn the_grounding_functor_loads_from_its_committed_prx_fail_closed() {
        // The grounding projection LIVES in `usc_legal_sources_functor.prx`: the
        // loader admits it ONLY against the baked root and yields a Functor action
        // with non-empty type + morphism tables.
        let GeneratorAction::Functor {
            map_object,
            map_morphism,
        } = &usc_legal_sources_functor().action
        else {
            panic!("the loaded grounding projection is a Functor action");
        };
        assert!(
            !map_object.is_empty() && !map_morphism.is_empty(),
            "the grounding functor carries non-empty type + morphism tables"
        );
        assert!(
            map_object
                .iter()
                .any(|(k, v)| k == SECTION_KIND && v == "Statute"),
            "the functor types a Section as a legal_sources:Statute"
        );
        // Fail-closed: a WRONG root is refused.
        assert!(
            pr4xis_runtime::load::load(USC_LEGAL_SOURCES_FUNCTOR_PRX, ContentAddress::of(b"wrong"))
                .is_err(),
            "a wrong root is refused — the grounding-functor load is fail-closed"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_pipeline_mints_type_grounding_edges_into_legal_sources() {
        use crate::formal::meta::grounding::ground_declared;
        use crate::social::judicial::legal_sources::ontology::LegalSourcesCategory;
        use pr4xis_runtime::definition::EdgeTarget;
        use pr4xis_runtime::emit::emit;
        // The USC archive CARRIES its grounding functor as data (a Connection); the
        // GENERAL `ground_declared` step mints the cross-ontology Grounded edges —
        // a section into legal_sources:Statute, the Code root into Code — against
        // the LOADED LegalSources peer. (The raw `project_archive` has NO such edge;
        // the with/without-grounding contrast.)
        let raw = project_archive(&UsCode::sample());
        assert!(
            raw.nodes.iter().all(|n| n
                .edges
                .iter()
                .all(|(_, t)| matches!(t, EdgeTarget::Local(_)))),
            "the RAW structural projection carries no cross-ontology grounded edge"
        );

        // The USC archive-with-grounding-functor-as-data, and the LegalSources peer
        // (built here from the same projection the runtime loads as the base, so the
        // atoms agree by content address).
        let usc = usc_archive(&UsCode::sample());
        assert!(
            usc.connections.iter().any(|c| c.kind == "InstanceFunctor"),
            "usc_archive carries its grounding functor as a Connection (data)"
        );
        let legal = emit::<LegalSourcesCategory>();
        let mut peers = alloc::collections::BTreeMap::new();
        peers.insert("LegalSources".to_string(), legal.clone());
        let grounded = ground_declared(&usc, &peers).expect("USC grounds into LegalSources");

        let statute_atom = legal
            .nodes
            .iter()
            .find(|n| n.name == "Statute")
            .unwrap()
            .address()
            .unwrap();
        // Some section grounds into the Statute atom of LegalSources.
        let grounds_statute = grounded.nodes.iter().any(|n| {
            n.kind.as_str() == SECTION_KIND
                && n.edges.iter().any(|(_, t)| {
                    matches!(t, EdgeTarget::Grounded { ontology, atom }
                        if ontology == "LegalSources" && *atom == statute_atom)
                })
        });
        assert!(
            grounds_statute,
            "ground_declared mints a section→legal_sources:Statute edge"
        );

        // The Code root grounds into legal_sources:Code.
        let code_atom = legal
            .nodes
            .iter()
            .find(|n| n.name == "Code")
            .unwrap()
            .address()
            .unwrap();
        let code_node = grounded
            .nodes
            .iter()
            .find(|n| n.name == CODE_ROOT_URN)
            .expect("the Code root node survives grounding");
        assert!(
            code_node.edges.iter().any(|(_, t)| {
                matches!(t, EdgeTarget::Grounded { ontology, atom }
                    if ontology == "LegalSources" && *atom == code_atom)
            }),
            "ground_declared mints a Code-root→legal_sources:Code edge"
        );
    }
}
