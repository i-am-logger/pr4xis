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
//! into a connected ontology (the lexical `denotes` floor, and `cites` —
//! [`citation_index`](crate::social::software::markup::xml::uslm::corpus::bridge::citation_index)
//! + [`cites_lens`](crate::social::judicial::statute_structure::grounding::cites_lens);
//! `defines` remains open follow-up work), resolved by the generic
//! `AtomResolver` — never a bespoke string side-channel.

use alloc::collections::{BTreeMap, BTreeSet};
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
use super::runtime_types::UsCodeRef;
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
    fn project_subdivision(sub: &UscSubdivision, parent_urn: &str, nodes: &mut Vec<Definition>) {
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
                .as_deref()
                .or(sub.chapeau.as_deref())
                .or(sub.content.as_deref())
                .map(ToString::to_string),
        });
        for child in &sub.children {
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
        for top in &section.subdivisions {
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

/// The `refs` side-table [`project_archive`] deliberately does NOT carry onto its
/// [`Definition`] nodes — URN → this provision's OWN [`UsCodeRef`] citations
/// (section refs at the section's URN, subdivision refs at the subdivision's
/// URN, exactly the same per-node scoping `UscSection::refs` /
/// `UscSubdivision::refs` document).
///
/// [`Definition`] has no generic field for structured pointer data (only
/// free-text `lexical`), and a raw href is USUALLY not a node `project_archive`
/// declares (most citations point outside whatever title is loaded) — so
/// threading `refs` onto the `Definition` as unconditional `Local` edges would
/// make `materialize`'s referential-closure check
/// ([`MaterializeError::DanglingEdge`]) reject nearly every real title. This
/// walk is the side-channel a `cites` lens closes over instead: same shape as
/// [`project_archive`]'s own tree walk (kept in lockstep deliberately — both
/// read `usc.all_sections()` and the same subdivision tree), but returning the
/// raw citation data by URN rather than folding it into the archive.
pub fn citation_index(usc: &UsCode) -> BTreeMap<String, Vec<UsCodeRef>> {
    fn index_subdivision(sub: &UscSubdivision, out: &mut BTreeMap<String, Vec<UsCodeRef>>) {
        if !sub.refs.is_empty() {
            out.insert(sub.urn.value().to_string(), sub.refs.clone());
        }
        for child in &sub.children {
            index_subdivision(child, out);
        }
    }

    let mut out = BTreeMap::new();
    for section in usc.all_sections() {
        if !section.refs.is_empty() {
            out.insert(section.urn.value().to_string(), section.refs.clone());
        }
        for top in &section.subdivisions {
            index_subdivision(top, &mut out);
        }
    }
    out
}

/// The provision PROSE a heading shadows — the side-table [`project_archive`]
/// deliberately does NOT carry onto its projected [`Definition`] node, exactly
/// mirroring the shape [`citation_index`] already established for `refs` (same
/// per-URN keying, same "extra side parameter instead of a `Definition`
/// field" consumption shape `cites_lens` already takes `refs_by_urn` in).
///
/// Indexed at BOTH levels of the USLM containment hierarchy (LRC *USLM XML
/// User Guide* §V), because [`project_archive`] shadows prose at both — by
/// two DIFFERENT mechanisms, with two different scoping conditions:
///
/// # Subdivision level — heading-FIRST shadowing
///
/// `project_subdivision`'s `lexical` selection is
/// `heading.or(chapeau).or(content)`: a subdivision that carries a `<heading>`
/// exposes ONLY that heading text to any lens reading `Definition.lexical` —
/// even when its `chapeau`/`content` carries the actual "the term X means Y"
/// declarative underneath. Measured against the real Title 42
/// caregiving-critical definition sections (300ii, 15002, the lettered 1395x
/// subsections, 1396d(a), …): 88 of 226 candidate definition texts are
/// shadowed this way (2026-07-15 `defines_coverage_probe` run, the
/// defines-lens gap backlog's finding S1) — the projection never even OFFERS
/// the defining sentence to [`defines_lens`](crate::social::judicial::statute_structure::grounding::defines_lens),
/// independent of whatever the grammar itself can or can't parse once it gets
/// there.
///
/// # Section ROOT — heading-ONLY shadowing (the backlog's L1)
///
/// A section root is NOT heading-first, it is heading-**only**: the section
/// `Definition` this module projects sets `lexical:
/// Some(section.heading.clone())` UNCONDITIONALLY, with no
/// `.or(chapeau).or(content)` fallthrough at all — so unlike a subdivision, a
/// section root is shadowed whether or not its heading exists, and no lens
/// reading `lexical` can EVER see a section's operative text. For a section
/// with enumerated subdivisions this costs nothing (its prose lives on the
/// subdivision nodes, which are projected and scanned under their own URNs),
/// but a section with NO enumerated subdivisions — USLM's ordinary shape for a
/// short prose-only section, which carries its whole operative text directly
/// in its own `<chapeau>`/`<content>` — has its entire body outside every scan
/// path. Measured on Title 1 (2026-07-25, `t1probe sections usc_title_1`): 32
/// of 39 sections are subdivision-less, 14,933 characters of operative text,
/// scanned nowhere; ten of the title's thirty definienda live there, including
/// the § 1 rules of construction and § 2 "county" / § 3 "vessel" / § 4
/// "vehicle".
///
/// The section-root entry is scoped to `subdivisions.is_empty()` for an
/// ATTRIBUTION reason, not a shadowing one. [`UscSection::text`](super::UscSection::text)
/// is a MEREOLOGICAL CLOSURE, not the section's own prose: `section_body_text`
/// (`super`) concatenates the section's own `<chapeau>`/`<content>` with every
/// descendant's, at any depth. Indexing that closure at the section URN would
/// attribute a subdivision's definition to its ANCESTOR — the exact
/// misattribution this whole side-table exists to avoid, and the discipline
/// [`citation_index`] already states ("section refs at the section's URN,
/// subdivision refs at the subdivision's URN … not aggregated"): containment
/// is Parthood (Casati & Varzi 1999, *Parts and Places*), and authorship of a
/// definition does not flow up a Parthood edge. `text` equals the section's
/// OWN prose exactly when the section has no parts — so that, and only that,
/// is when it may be keyed to the section's own URN. Measured on Title 1: the
/// unscoped form mints three new ancestor-misattributions (`state` at § 7 —
/// really § 7(b); `secretary` at § 112b — really § 112b(k)(6); `code` at
/// § 204), the scoped form mints none.
///
/// This is a fix scoped to `defines` alone — a second field bolted onto the
/// shared runtime [`Definition`] (or a heading/prose-carrying enum replacing
/// `lexical`) would change every node's content address AND every OTHER
/// `lexical` consumer (`denotes_lens`, a chat "what is X" gloss, the
/// corpus-pin address) for a gap that only `defines_lens` has. So, same as
/// `citation_index`, the prose lives in this side-table instead: keyed by the
/// shadowed provision's own URN, consulted ADDITIONALLY to (never instead
/// of) `Definition.lexical` by [`defines_lens`](crate::social::judicial::statute_structure::grounding::defines_lens),
/// so heading-first `lexical` is completely unchanged for every other lens
/// and caller.
///
/// Only a GENUINELY shadowed provision gets an entry: a subdivision with no
/// heading already exposes its prose via `lexical` (an entry here would be a
/// pure duplicate), and one with a heading but no body prose (a pure grouping
/// container — e.g. a chapeau-less numbered placeholder whose content lives
/// entirely in its children) has nothing to add; likewise a section root with
/// empty `text` (a `[Reserved]` placeholder). Chapeau is preferred over
/// content when a subdivision somehow carries both, matching
/// `project_subdivision`'s own preference order.
///
/// # Known residual — a section-level chapeau ABOVE its subdivisions
///
/// A section that has BOTH its own `<chapeau>`/`<content>` and enumerated
/// subdivisions still exposes that top-level prose nowhere: it cannot be
/// recovered from [`UscSection::text`](super::UscSection::text) without
/// dragging every descendant's text along with it, because
/// [`UscSection`](super::UscSection) — unlike [`UscSubdivision`] — carries no
/// separate `chapeau`/`content` fields, only the flattened closure. Closing
/// that half is a `UscSection`/`UscSectionAux` shape change, which moves the
/// archived `[compact_archive_signatures]` bytes, so it is a corpus-artifact
/// regeneration, not a lens change.
pub fn defines_prose_index(usc: &UsCode) -> BTreeMap<String, String> {
    fn index_subdivision(sub: &UscSubdivision, out: &mut BTreeMap<String, String>) {
        if sub.heading.is_some()
            && let Some(prose) = sub.chapeau.as_deref().or(sub.content.as_deref())
        {
            out.insert(sub.urn.value().to_string(), prose.to_string());
        }
        for child in &sub.children {
            index_subdivision(child, out);
        }
    }

    let mut out = BTreeMap::new();
    for section in usc.all_sections() {
        // A subdivision-less section: `text` IS this section's own
        // `<chapeau>`/`<content>` and nothing else (no parts to close over),
        // so it may be keyed to the section's own URN without attributing a
        // descendant's words to it. See this function's own doc.
        if section.subdivisions.is_empty() && !section.text.is_empty() {
            out.insert(section.urn.value().to_string(), section.text.clone());
        }
        for top in &section.subdivisions {
            index_subdivision(top, &mut out);
        }
    }
    out
}

/// The RE-ASSEMBLED virtual-declarative-sentence candidates for every
/// DANGLING-CHAPEAU subdivision — the side-table fix for the defines-lens
/// gap backlog's G3 ("dangling-chapeau enumerated definiens", the USLM
/// "sandwich": `<chapeau>` ends its sentence in an em dash or colon, with
/// the definiens continuing in enumerated `children`) and S2 ("split
/// definition", e.g. 42 U.S.C. § 1396n(c)(5) "habilitation services": the
/// chapeau carries only the term + em dash, and the "means"/"includes" verb
/// itself lives on a CHILD subparagraph — no single node ever carries a
/// parseable declarative).
///
/// Structurally, G3 and S2 are the SAME shape (a chapeau ending in a loaded
/// [`WritingSystem::is_enumeration_introducer`](crate::cognitive::linguistics::orthography::WritingSystem::is_enumeration_introducer)
/// mark, followed by `children` in [`UscSubdivision::children`]) — S2 only LOOKS different
/// because its children happen to each open with their own definitional
/// verb rather than continuing one shared relative clause. So this walk
/// does not classify which shape a node is; for every dangling chapeau with
/// at least one non-empty child, it mints TWO kinds of virtual-sentence
/// candidate, uniformly:
///
/// 1. **ALL-joined** — `chapeau` followed by EVERY child's own prose
///    (`chapeau.or(content)`, in document order) — the direct reading of
///    "reassemble it with its enumerated children into one virtual
///    declarative sentence" for G3's enumerated-relative-clause shape (e.g.
///    developmental disability, 42 U.S.C. § 15002(8)(A): "...a severe,
///    chronic disability of an individual that— (i) is attributable to...
///    (v) reflects...").
/// 2. **PER-CHILD** — `chapeau` followed by JUST that one child's prose —
///    the shape S2 needs: habilitation's chapeau ("...the term
///    "habilitation services"—") paired with EACH sibling subparagraph
///    separately ("means services designed to..." / "includes
///    (except...) prevocational..."), since each is its own independent
///    assertion (Dickerson's means/includes distinction,
///    [`crate::social::judicial::statute_structure::grounding::defines_pointers`]'s
///    own `is_partial_definition_verb` doc) rather than a joint compound
///    clause.
///
/// Trying both uniformly (never guessing which shape applies) keeps this
/// walk purely STRUCTURAL: whichever candidate(s) the FULL existing
/// tokenize/chart/Montague/VerbNet pipeline can actually reduce is what
/// [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
/// decides, downstream — the same
/// "no coverage → no pointer" discipline every other lens in this codebase
/// already establishes. Every candidate is attributed to the ORIGINAL
/// dangling node's OWN URN (never a synthetic node) — the same per-URN
/// side-channel shape [`defines_prose_index`] and [`citation_index`]
/// already establish, consulted ADDITIONALLY by
/// [`grounding::defines_lens`](crate::social::judicial::statute_structure::grounding::defines_lens),
/// never instead of `Definition.lexical` or `shadowed_prose`.
///
/// Recurses into EVERY child regardless of whether the parent itself
/// qualifies, so a nested dangling chapeau (e.g. § 15002(8)(A)(iv), whose
/// OWN chapeau "results in substantial functional limitations ... :" ends
/// in a colon and introduces its own `(I)`–`(VII)` subclauses) is indexed
/// under its own URN too — independent of, and un-shadowed by, its
/// ancestor's entry.
pub fn dangling_chapeau_reassembly_index(usc: &UsCode) -> BTreeMap<String, Vec<String>> {
    use crate::cognitive::linguistics::orthography::english_writing_system;

    fn child_prose(child: &UscSubdivision) -> Option<&str> {
        child.chapeau.as_deref().or(child.content.as_deref())
    }

    fn index_subdivision(
        sub: &UscSubdivision,
        ws: &crate::cognitive::linguistics::orthography::WritingSystem,
        out: &mut BTreeMap<String, Vec<String>>,
    ) {
        if let Some(chapeau) = sub.chapeau.as_deref()
            && chapeau
                .trim_end()
                .chars()
                .next_back()
                .is_some_and(|c| ws.is_enumeration_introducer(c))
        {
            let child_texts: Vec<&str> = sub.children.iter().filter_map(child_prose).collect();
            if !child_texts.is_empty() {
                let mut candidates: Vec<String> = Vec::new();
                candidates.push(format!("{chapeau} {}", child_texts.join(" ")));
                for text in &child_texts {
                    let per_child = format!("{chapeau} {text}");
                    if !candidates.contains(&per_child) {
                        candidates.push(per_child);
                    }
                }
                out.insert(sub.urn.value().to_string(), candidates);
            }
        }
        for child in &sub.children {
            index_subdivision(child, ws, out);
        }
    }

    let ws = english_writing_system();
    let mut out = BTreeMap::new();
    for section in usc.all_sections() {
        for top in &section.subdivisions {
            index_subdivision(top, &ws, &mut out);
        }
    }
    out
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

/// [`usc_runtime_ontology`], additionally grounding each provision's
/// statutory `defines` edges (task #6 — "the term X means Y" extraction,
/// [`defines_lens`](crate::social::judicial::statute_structure::grounding::defines_lens))
/// before materializing.
///
/// Unlike the TYPE grounding ([`usc_archive`]'s appended `Connection`,
/// resolved later by the loader's general `ground_declared`/
/// `ground_loaded_set` pass against the loaded peer set), `defines` is a
/// LEXICAL lens: it runs the real tokenize/chart/Montague pipeline over
/// each provision's prose, so it needs `lang`/`reasoner`/`verbnet` at
/// grounding time rather than deferring to a later declared-functor pass.
/// `lang` doubles as `reasoner` (`English` itself implements
/// `LexicalReasoner`) — no COMPOSED reasoner exists yet at USC-load time
/// (this ontology has not been materialized, let alone joined into one),
/// which is exactly why `defines_lens` is written against the SAME typed
/// interface either can satisfy, mirroring its own test suite's usage.
///
/// `defines_lens` never fails closed (every node yields `Ok(..)` — see its
/// own doc), so the `expect` here asserts a real, documented invariant, not
/// an unchecked shortcut.
///
/// # NOT a live-startup call — measured cost, not a guess
///
/// `defines_pointers` runs the FULL tokenize/chart/Montague pipeline per
/// node (~26–40ms/node measured against real loaded prose, `scratch_probe.rs`'s
/// `probe_defines_pointers_timing`/`probe_usc_provision_count_and_sample_timing`).
/// The full loaded USC corpus (all registered titles) carries 295,830
/// lexical-prose nodes — a ~3.3 HOUR projected wall-clock cost; even Title 42
/// alone carries 138,956 (~1.5 hours). This makes calling this function from
/// `pr4xis-cli`/`pr4xis-wasm` process STARTUP (where `usc_runtime_ontology`
/// runs today) completely non-viable — it would turn an instant CLI/chat
/// load into an hours-long hang. This function is deliberately NOT wired
/// into that live startup path. It is a correct, tested building block for
/// a future BUILD-TIME grounding pass (a `pr4xis compile`-style one-time
/// step producing a cached, committed pre-grounded archive/`.prx`, mirroring
/// this codebase's existing "parse-once fast load" precedent for other
/// expensive one-time computations) — not yet built, tracked separately.
///
/// `mint_domain` is the defines-lens gap backlog's G7 fix's statute-local
/// minting namespace: a definiendum no tier of `is_known_written_form`
/// recognizes is a genuine statutory coinage, minted into `mint_domain`
/// rather than dropped (`defines_pointers`'s own doc, point 4, has the full
/// mechanism) — pointing a `Grounded` edge at a term that resolves once a
/// caller composes the SAME `(mint_domain, term)` mint into a
/// [`ComposedReasoner`](crate::cognitive::linguistics::composed::ComposedReasoner)'s
/// `loaded` set (not yet wired here, matching G6's `renvoi_lens`'s own
/// "built, tested, not yet composed into the live pipeline" precedent).
pub fn usc_runtime_ontology_with_defines(
    usc: &UsCode,
    name: OntologyName,
    lang: &crate::cognitive::linguistics::english::English,
    verbnet: &crate::cognitive::linguistics::verbnet::ontology::VerbNet,
    mint_domain: &OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    use crate::social::judicial::statute_structure::grounding::defines_lens;
    // S1: recover the prose a heading shadows (see [`defines_prose_index`]'s
    // own doc) BEFORE grounding, so `defines_lens` sees it alongside every
    // node's `lexical`.
    let shadowed_prose = defines_prose_index(usc);
    // G3+S2: reassemble every dangling-chapeau node's virtual-declarative
    // candidates (see [`dangling_chapeau_reassembly_index`]'s own doc)
    // BEFORE grounding, so `defines_lens` runs them through the same
    // pipeline alongside `lexical`/`shadowed_prose`.
    let reassembled = dangling_chapeau_reassembly_index(usc);
    let grounded = pr4xis_runtime::grounding::ground(
        &usc_archive(usc),
        defines_lens(
            lang,
            lang,
            verbnet,
            &shadowed_prose,
            &reassembled,
            mint_domain,
        ),
    )
    .expect("defines_lens never fails closed (see its own doc)");
    materialize(grounded, name)
}

/// The sparse DEFINES-edge overlay [`usc_runtime_ontology_with_defines`]
/// grounds — `(provision URN, definiendum term)` pairs, one per
/// [`DefinesPointer`](crate::social::judicial::statute_structure::grounding::DefinesPointer)
/// `defines_pointers` finds. This is the cacheable UNIT (task #6's build-
/// time grounding pass, `uslm::corpus::prx::emit_all_compact_usc_defines_prx_gz`):
/// re-serializing the whole tens-of-thousands-of-node structural archive for
/// a result that touches a tiny fraction of it would be wasteful, and
/// conflating the two artifacts would force every title's EXISTING
/// `[compact_archive_signatures]` pin to shift whenever the defines grammar
/// changes something structurally unrelated. The full grounded archive is
/// reconstructed at load time by [`apply_defines_overlay`], which re-derives
/// each pair's `EdgeTarget::Grounded` from the bare term string — cheap (a
/// hash), never a re-parse.
///
/// EXPENSIVE — see [`usc_runtime_ontology_with_defines`]'s own doc for the
/// measured per-node cost. Build-time-only; never called from a live
/// process startup.
///
/// Parallelized across `std::thread::available_parallelism()` worker
/// threads (native `std::thread::scope`, no new dependency): each lexical-
/// prose node's `defines_pointers` call is independent — no shared mutable
/// state, `English`/`VerbNet` are immutable zero-copy archives (`Sync`) — so
/// this is an embarrassingly parallel per-node sweep. Splitting titles
/// across threads instead (rather than splitting one title's nodes) would
/// NOT help the dominant cost: the largest title's wall-clock still gates
/// the whole run regardless of how fast the smaller titles finish alongside
/// it. Splitting within a title is what actually shortens that title's own
/// wall-clock.
///
/// `mint_domain` is passed straight through to `defines_pointers` (the
/// defines-lens gap backlog's G7 fix — see its own doc), but a MINTED
/// pointer is deliberately NOT cached here: [`apply_defines_overlay`]
/// reconstructs every cached pair's `EdgeTarget::Grounded` against
/// `english_wordnet` (its own doc: "re-derives ... from the bare term
/// string"), a hardcoded, English-only assumption this sparse `(urn, term)`
/// pair format has no room to carry a per-term target ontology in without a
/// real cache-format migration (a separate, `.prx.gz`-regeneration-scale
/// task — see the module-level `defines` doc's own "not yet built" note).
/// Caching a minted pointer under this format would silently reconstruct
/// the WRONG target (English's, not the mint domain's), so only pointers
/// [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
/// grounds into `english_wordnet` are kept; a minted
/// pointer is still correctly computed (and still reachable via the
/// UNCACHED [`usc_runtime_ontology_with_defines`] path), just not persisted
/// by this cache producer.
pub fn compute_defines_overlay(
    usc: &UsCode,
    lang: &crate::cognitive::linguistics::english::English,
    verbnet: &crate::cognitive::linguistics::verbnet::ontology::VerbNet,
    mint_domain: &OntologyName,
) -> Vec<(String, String)> {
    use crate::cognitive::linguistics::english::bridge::ENGLISH_ONTOLOGY;
    use crate::social::judicial::statute_structure::grounding::defines_pointers;

    let archive = usc_archive(usc);
    // S1: a heading-shadowed subdivision's real prose never reaches
    // `lexical` (see [`defines_prose_index`]'s own doc) — scanned here under
    // the SAME URN as its node's `lexical` entry, so the cached overlay
    // carries whatever the shadowed prose alone can ground, matching
    // `usc_runtime_ontology_with_defines`'s (uncached) `defines_lens`
    // behavior exactly.
    let shadowed_prose = defines_prose_index(usc);
    // G3+S2: reassemble every dangling-chapeau node's virtual-declarative
    // candidates (see [`dangling_chapeau_reassembly_index`]'s own doc) —
    // scanned here under the SAME URN as its node's `lexical` entry, so the
    // cached overlay matches `usc_runtime_ontology_with_defines`'s
    // (uncached) `defines_lens` behavior exactly.
    let reassembled = dangling_chapeau_reassembly_index(usc);
    let mut lexical_nodes: Vec<(&str, &str)> = archive
        .nodes
        .iter()
        .filter_map(|node| {
            node.lexical
                .as_deref()
                .map(|text| (node.name.as_str(), text))
        })
        .collect();
    lexical_nodes.extend(
        shadowed_prose
            .iter()
            .map(|(urn, prose)| (urn.as_str(), prose.as_str())),
    );
    lexical_nodes.extend(
        reassembled.iter().flat_map(|(urn, candidates)| {
            candidates.iter().map(move |c| (urn.as_str(), c.as_str()))
        }),
    );

    let process_one = |(name, text): &(&str, &str)| -> Vec<(String, String)> {
        defines_pointers(text, lang, lang, verbnet, mint_domain)
            .into_iter()
            // Only an ENGLISH-grounded pointer is cacheable in this format —
            // see this function's own doc for why a MINTED (G7) pointer is
            // deliberately excluded here.
            .filter(|p| {
                matches!(&p.target, EdgeTarget::Grounded { ontology, .. } if ontology == ENGLISH_ONTOLOGY)
            })
            .map(|p| ((*name).to_string(), p.term))
            .collect()
    };

    let raw: Vec<(String, String)> = if lexical_nodes.len() <= 1 {
        lexical_nodes.iter().flat_map(&process_one).collect()
    } else {
        // A BOUNDED pool of `available_parallelism()` worker threads — the
        // runtime environment's OWN reported execution-unit count, never a
        // chosen constant — each pulling the NEXT unclaimed node off a
        // shared atomic cursor until the slice is exhausted. This is TRUE
        // dynamic load-balancing (a worker that finishes a cheap node
        // immediately claims the next one, so no static up-front split can
        // strand one thread with a disproportionately expensive share while
        // others sit idle — the flaw a fixed-chunk-count split had), without
        // paying for one OS thread per node: a large title's provision
        // count can run into the thousands, and spawning that many
        // concurrent threads when only a handful of execution units exist
        // to run them pays real context-switch and stack-allocation
        // overhead for zero additional parallelism beyond the hardware's
        // own capacity — confirmed by direct measurement (spawning ~2000
        // threads against `available_parallelism() == 8` showed exactly
        // this: `nlwp` climbing into the thousands while wall-clock
        // improvement over a bounded pool would be nil-to-negative).
        let worker_count = std::thread::available_parallelism()
            .map(core::num::NonZeroUsize::get)
            .unwrap_or(1)
            .min(lexical_nodes.len());
        let cursor = core::sync::atomic::AtomicUsize::new(0);
        // Each worker carries the CLAIM INDEX back with its output, and the
        // joined results are re-sorted by it before flattening.
        //
        // Without that, this function is NONDETERMINISTIC and the artifact it
        // feeds is content-addressed, so the address itself became a coin
        // flip: workers claim items out of a racing `AtomicUsize` but their
        // `out` vectors were concatenated in WORKER-INDEX order, so which
        // worker happened to win which item decided the pair order.
        // `defines_overlay_to_succinct` (`prx.rs`) then writes the pairs in
        // exactly that order with no sort, and the byte layout IS the address.
        // Measured before this fix: 12 emits of `usc_title_1` — same binary,
        // same input, 32 threads — produced 6 DISTINCT content addresses (the
        // 6 permutations of its 3 pairs); `usc_title_28` gave 4 distinct
        // addresses over 4 emits with gzip lengths 700/698/705/694. Pinned to
        // one core the pool collapses to a single worker and the address was
        // stable. So every `[compact_defines_signatures]` pin was one
        // arbitrary permutation that no independent rebuild could reproduce,
        // and the dedup below could not repair it — it preserves
        // first-occurrence of whatever scrambled list it is handed.
        //
        // Sorting by claim index reproduces the single-worker order exactly,
        // which is document order, which `apply_defines_overlay` then replays
        // into `node.edges`. The atomic cursor is KEPT rather than replaced by
        // a static split because per-item cost spans four orders of magnitude
        // and dynamic balancing earns its keep; the sort is milliseconds
        // against minutes.
        std::thread::scope(|scope| {
            let mut claimed: Vec<(usize, Vec<(String, String)>)> = (0..worker_count)
                .map(|_| {
                    let cursor = &cursor;
                    let lexical_nodes = &lexical_nodes;
                    let process_one = &process_one;
                    scope.spawn(move || {
                        let mut out: Vec<(usize, Vec<(String, String)>)> = Vec::new();
                        loop {
                            let i = cursor.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                            let Some(item) = lexical_nodes.get(i) else {
                                break;
                            };
                            out.push((i, process_one(item)));
                        }
                        out
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|handle| {
                    handle
                        .join()
                        .expect("defines_pointers worker thread panicked")
                })
                .collect();
            claimed.sort_unstable_by_key(|(i, _)| *i);
            claimed.into_iter().flat_map(|(_, pairs)| pairs).collect()
        })
    };

    // A dangling node offers MULTIPLE reassembled candidates (the ALL-joined
    // form plus one PER-CHILD, see `dangling_chapeau_reassembly_index`'s own
    // doc) that routinely name the SAME definiendum once more than one
    // candidate happens to parse — dedup by (urn, term), first-occurrence
    // order preserved, so the cached overlay never carries a redundant
    // duplicate edge for a case `lexical`/`shadowed_prose` alone never could.
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut deduped = Vec::with_capacity(raw.len());
    for (urn, term) in &raw {
        if seen.insert((urn.as_str(), term.as_str())) {
            deduped.push((urn.clone(), term.clone()));
        }
    }
    deduped
}

/// Apply a [`compute_defines_overlay`]-shaped pair list onto
/// [`usc_archive`]'s output, then materialize — the FAST path equivalent of
/// [`usc_runtime_ontology_with_defines`], using a CACHED overlay instead of
/// re-running `defines_lens` (the ~30ms/node cost this makes disappear
/// entirely: an O(matched-nodes) URN lookup plus one
/// `form_atom(term).address()` hash per pair, not a re-parse). A pair whose
/// URN no longer names a node in `usc`'s current archive (a stale overlay
/// against edited source text) is silently skipped — the SAME "declared but
/// not present" tolerance `type_lens`'s sibling grounding passes apply,
/// never a panic over a cache that has drifted from its source; the
/// content-address gate on the overlay ARTIFACT itself
/// (`load_compact_usc_defines_prx_gz_gated`) is what actually prevents a
/// stale overlay from installing silently — this merge step's own job is
/// just to apply whatever pairs it is handed.
pub fn usc_runtime_ontology_from_cached_defines(
    usc: &UsCode,
    name: OntologyName,
    overlay: &[(String, String)],
) -> Result<RuntimeOntology, MaterializeError> {
    materialize(apply_defines_overlay(usc_archive(usc), overlay), name)
}

/// Push one `(defines, EdgeTarget::Grounded)` edge per overlay pair onto its
/// named node — the reconstruction half of [`compute_defines_overlay`]'s
/// cache/replay split. See [`usc_runtime_ontology_from_cached_defines`] for
/// the stale-URN tolerance.
///
/// `pub`, not just the private half of `usc_runtime_ontology_from_cached_
/// defines`: `crates/wasm/build.rs`'s `stage_usc_archives` calls this
/// DIRECTLY (not that function) because it needs the merged `Archive`
/// itself to serialize via `ArchiveLens::put_aligned` — `usc_runtime_
/// ontology_from_cached_defines` already materializes past that point,
/// into a `RuntimeOntology` the wasm build-time staging step has no use
/// for (the browser loads pre-staged `.rprx` bytes, never runs
/// `materialize` itself for USC titles — see `Encoding::RkyvArchive`'s own
/// doc in `wasm/src/lib.rs`).
pub fn apply_defines_overlay(mut archive: Archive, overlay: &[(String, String)]) -> Archive {
    use crate::cognitive::linguistics::english::bridge::ENGLISH_ONTOLOGY;
    use crate::social::judicial::statute_structure::grounding::DEFINES_REL;
    for (urn, term) in overlay {
        let Ok(atom) = form_atom(term).address() else {
            continue;
        };
        if let Some(node) = archive.nodes.iter_mut().find(|n| &n.name == urn) {
            node.edges.push((
                DEFINES_REL.to_string(),
                EdgeTarget::Grounded {
                    ontology: ENGLISH_ONTOLOGY.to_string(),
                    atom,
                },
            ));
        }
    }
    archive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::bridge::FORM_KIND;
    use alloc::collections::BTreeSet;
    use pr4xis_runtime::connection::GeneratorAction;

    /// The G7 statute-local minting namespace this file's own `defines`
    /// tests use — see `usc_runtime_ontology_with_defines`'s own doc.
    fn mint_domain() -> OntologyName {
        OntologyName::new_static("usc_t42_coinages")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn projects_every_section_and_subdivision_as_a_node() {
        let usc = UsCode::sample();
        let archive = project_archive(&usc);
        let expected: usize = usc.section_count().value as usize
            + usc
                .all_sections()
                .iter()
                .map(|s| s.subdivision_count().value as usize)
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

    /// `citation_index` buckets each provision's `refs` by its OWN URN — section
    /// refs at the section's URN, subdivision refs at the subdivision's URN —
    /// mirroring exactly the per-node scoping `UscSection::refs` /
    /// `UscSubdivision::refs` already document and
    /// `refs_survive_parse_and_compact_round_trip` (prx.rs) already proves for
    /// the parse/persist path. A node with no citations gets no entry (not an
    /// empty `Vec`) — the index is sparse.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn citation_index_buckets_refs_by_the_citing_node_urn() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const SAMPLE_WITH_REFS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 18</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t18">
      <num value="18">Title 18</num>
      <heading>CRIMES AND CRIMINAL PROCEDURE</heading>
      <section identifier="/us/usc/t18/s1513">
        <num value="1513">§ 1513.</num>
        <heading>Retaliating against a witness, victim, or an informant</heading>
        <chapeau>Whoever knowingly takes any action harmful to any person, in violation of <ref href="/us/usc/t18/s1512">section 1512 of this title</ref>, shall—</chapeau>
        <subsection identifier="/us/usc/t18/s1513/a">
          <num value="a">(a)</num>
          <content>be subject to the penalties described in <ref href="/us/usc/t15/s78j">section 78j of title 15</ref>.</content>
        </subsection>
      </section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(SAMPLE_WITH_REFS).expect("parse title with refs");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        let index = citation_index(&usc);
        assert_eq!(
            index.len(),
            2,
            "both the section and the subsection cite something"
        );

        let section_refs = index.get("/us/usc/t18/s1513").expect("section-scoped ref");
        assert_eq!(section_refs.len(), 1);
        assert_eq!(section_refs[0].href, "/us/usc/t18/s1512");

        let sub_refs = index
            .get("/us/usc/t18/s1513/a")
            .expect("subsection-scoped ref");
        assert_eq!(sub_refs.len(), 1);
        assert_eq!(sub_refs[0].href, "/us/usc/t15/s78j");

        // The un-cited Code root and any provision that cites nothing simply has
        // no entry — never a spurious empty bucket.
        assert!(!index.contains_key(CODE_ROOT_URN));
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

    /// Task #6, layer 1: [`usc_runtime_ontology_with_defines`] actually grounds
    /// a real "the term X means Y" provision — a custom `UsCode` (same
    /// `CodegenData` shape [`UsCode::sample`] itself uses) with one section
    /// whose prose is the exact grammar shape `defines_lens`'s own real-
    /// statute-prose test (`grounding.rs::recognizes_the_term_x_means_y_
    /// from_real_statute_prose`) is verified against, so this test proves
    /// the COMPOSITION (ground → materialize), not a re-derivation of
    /// `defines_pointers`'s own grammar coverage.
    ///
    /// TWO edges are expected here, not one — a fixture artifact, not a
    /// production duplication risk: this minimal fixture has no `aux`
    /// subdivision table, so the ONLY way to get the defining sentence into
    /// ANY node's `lexical` field is via `entity_labels` (a SECTION's
    /// `lexical` is its heading — confirmed by direct probe of
    /// `project_archive`, which projects `entity_defs`/body text only into
    /// SUBDIVISION nodes). That same heading string also becomes an
    /// `ontolex:Form` atom (`project_archive` mints one per heading/citation
    /// surface), and a Form atom's `lexical` is its own surface text
    /// verbatim — so the identical sentence is scanned twice, once as the
    /// Section's heading and once as its own Form atom, and the grammar
    /// matches both. This can NOT happen with real USC data: a real
    /// section's heading is a short title ("Definitions"), never a full
    /// defining sentence, and the actual body prose only ever reaches a
    /// SUBDIVISION node's `lexical` — never also becomes a Form-atom surface
    /// (only headings/citations do).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn usc_runtime_ontology_with_defines_grounds_a_real_definition() {
        use crate::cognitive::linguistics::english::english_loaded;
        use crate::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
        use crate::social::judicial::statute_structure::grounding::DEFINES_REL;
        use pr4xis::codegen_data::CodegenData;

        // A SECTION-level node's `lexical` field is populated from
        // `entity_labels` (the heading), NOT `entity_defs` (`project_archive`
        // only projects `entity_defs`/body text into SUBDIVISION nodes, none
        // of which this minimal fixture has an `aux` table for) — confirmed
        // by direct probe, not assumed.
        static DEFINES_SAMPLE: CodegenData<UsCode> = CodegenData {
            entity_count: 1,
            entity_ids: &["/us/usc/t15/s6603"],
            entity_kind: &["section"],
            entity_labels: &["The term \u{201C}consumer\u{201D} means a natural person."],
            entity_defs: &[""],
            word_index: &[],
            taxonomy: &[],
            mereology: &[],
            opposition: &[],
            equivalence: &[],
            causation: &[],
            references: &[],
        };
        let usc = UsCode::from_codegen(&DEFINES_SAMPLE);
        let lang = english_loaded();
        let verbnet = verbnet_classes_loaded();
        let onto = usc_runtime_ontology_with_defines(
            &usc,
            OntologyName::new_static("usc_defines"),
            lang,
            verbnet,
            &mint_domain(),
        )
        .expect("the defines-grounded USC pipeline materializes");
        let archive = onto.archive();
        let defines_edges: usize = archive
            .nodes
            .iter()
            .map(|n| {
                n.edges
                    .iter()
                    .filter(|e| e.0.as_str() == DEFINES_REL)
                    .count()
            })
            .sum();
        assert_eq!(
            defines_edges, 2,
            "expected the DEFINES edge on both the Section node and its own \
             Form-atom node (this fixture's known duplication, see this \
             test's own doc) for the \"consumer\" definition"
        );
    }

    // ---------------------------------------------------------------------------
    // S1: `defines_prose_index` recovers the prose a heading shadows.
    // ---------------------------------------------------------------------------

    /// `defines_prose_index` recovers a heading-shadowed subdivision's REAL
    /// prose, and `project_archive`'s `lexical` for that SAME node is
    /// unchanged (still heading-only) — proving the fix is additive, not a
    /// change to the existing heading-first projection every other `lexical`
    /// consumer relies on.
    ///
    /// The fixture is 42 U.S.C. § 6291(55)(B) — the "BR30" lamp-ballast
    /// definition — byte-verified against
    /// `usc_title_42-pl-119-90.xml`: `<heading> BR30.—</heading><content>The
    /// term "BR30" means a BR incandescent reflector lamp with a diameter of
    /// 30/8ths of an inch.</content>`, real text this codebase's own S1 gap
    /// measurement flagged as shadowed. (Its own definiens is separately
    /// blocked by the still-open G4 PP-chain and G7 unknown-acronym gaps —
    /// this test targets ONLY the projection-level fix, not full grounding;
    /// see `usc_runtime_ontology_with_defines_grounds_a_heading_shadowed_subdivision`
    /// below for the full-pipeline proof, isolated from those other gaps.)
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn defines_prose_index_recovers_real_title_42_prose_a_heading_shadows() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const BR30_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 42</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t42">
      <num value="42">Title 42</num>
      <heading>THE PUBLIC HEALTH AND WELFARE</heading>
      <section identifier="/us/usc/t42/s6291">
        <num value="6291">§ 6291.</num>
        <heading> Definitions</heading>
        <paragraph identifier="/us/usc/t42/s6291/55/B">
          <num value="B">(B)</num>
          <heading> BR30.—</heading>
          <content>The term “BR30” means a BR incandescent reflector lamp with a diameter of 30/8ths of an inch.</content>
        </paragraph>
      </section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(BR30_SAMPLE).expect("parse the real BR30 fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        // The projection's `lexical` is STILL heading-only for this node — S1
        // does not touch `Definition.lexical` itself.
        let raw = project_archive(&usc);
        let shadowed_node = raw
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t42/s6291/55/B")
            .expect("the BR30 subdivision projects");
        assert_eq!(
            shadowed_node.lexical.as_deref(),
            Some("BR30.—"),
            "the projected node's lexical is unchanged — still heading-only"
        );

        // The prose the heading shadowed is recovered, verbatim, by URN.
        let prose = defines_prose_index(&usc);
        assert_eq!(
            prose.get("/us/usc/t42/s6291/55/B").map(String::as_str),
            Some(
                "The term \u{201C}BR30\u{201D} means a BR incandescent reflector \
                 lamp with a diameter of 30/8ths of an inch."
            ),
            "the real body prose the heading shadowed is recovered verbatim"
        );
        // Sparse, and NON-AGGREGATING: this section HAS a subdivision, so its
        // `text` is a mereological closure over that subdivision's prose —
        // keying it at the section URN would attribute the (B) definition to
        // its ancestor. The section root is deliberately excluded (see
        // `defines_prose_index`'s own doc; the subdivision-LESS case is
        // covered by
        // `defines_prose_index_recovers_a_real_subdivision_less_section_body`
        // below).
        assert!(
            !prose.contains_key("/us/usc/t42/s6291"),
            "a section WITH subdivisions is never keyed — its `text` is a \
             closure over its parts, and a part's definition is not its \
             ancestor's"
        );
        assert_eq!(prose.len(), 1, "exactly the one genuinely shadowed node");
    }

    /// L1: `defines_prose_index` recovers a SUBDIVISION-LESS section's own
    /// body — the operative text `project_archive` shadows behind an
    /// unconditional heading-only `lexical` at the section root — and does
    /// NOT key a section that HAS subdivisions, whose `text` is a
    /// mereological closure over its parts.
    ///
    /// Both fixtures are REAL, byte-verified against
    /// `usc_title_1-pl-119-90.xml`:
    ///
    /// * 1 U.S.C. § 2 — a subdivision-less section (`<content>` directly
    ///   under `<section>`, USLM's ordinary shape for a short prose-only
    ///   section). Its definiendum "county" is one of the ten Title 1
    ///   definienda that lived in text no scan path reached.
    /// * 1 U.S.C. § 7 — three `<subsection>`s, the middle of which
    ///   (§ 7(b)) is where "State" is really defined. The section root must
    ///   stay unkeyed: § 7's `text` contains (b)'s sentence, so keying it
    ///   would produce a `(/us/usc/t1/s7, "state")` pair for a provision
    ///   that defines nothing — measured as one of exactly three
    ///   ancestor-misattributions the unscoped form mints on this title.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn defines_prose_index_recovers_a_real_subdivision_less_section_body() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const TITLE_1_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 1</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t1">
      <num value="1">Title 1</num>
      <heading>GENERAL PROVISIONS</heading>
      <section identifier="/us/usc/t1/s2"><num value="2">§ 2.</num><heading> “County” as including “parish”, and so forth</heading><content>
<p class="indent0">The word “county” includes a parish, or any other equivalent subdivision of a State or Territory of the United States.</p>
</content></section>
      <section identifier="/us/usc/t1/s7"><num value="7">§ 7.</num><heading> Marriage</heading><subsection identifier="/us/usc/t1/s7/b"><num value="b">(b)</num><content> In this section, the term “State” means a State, the District of Columbia, the Commonwealth of Puerto Rico, or any other territory or possession of the United States.</content>
</subsection></section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(TITLE_1_SAMPLE).expect("parse the real Title 1 fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        // The projection's `lexical` is STILL heading-only at BOTH section
        // roots — L1 does not touch `Definition.lexical` itself.
        let raw = project_archive(&usc);
        for (urn, heading) in [
            (
                "/us/usc/t1/s2",
                "\u{201C}County\u{201D} as including \u{201C}parish\u{201D}, and so forth",
            ),
            ("/us/usc/t1/s7", "Marriage"),
        ] {
            let node = raw
                .nodes
                .iter()
                .find(|n| n.name == urn)
                .expect("the section projects");
            assert_eq!(
                node.lexical.as_deref(),
                Some(heading),
                "the projected section node's lexical is unchanged — still \
                 heading-only"
            );
        }

        let prose = defines_prose_index(&usc);

        // The subdivision-less section's own operative text is recovered
        // verbatim, keyed to the section's OWN URN.
        assert_eq!(
            prose.get("/us/usc/t1/s2").map(String::as_str),
            Some(
                "The word \u{201C}county\u{201D} includes a parish, or any \
                 other equivalent subdivision of a State or Territory of the \
                 United States."
            ),
            "a subdivision-less section's body — scanned by no path before — \
             is recovered under its own URN"
        );

        // The section WITH parts is not keyed: its own `text` closes over
        // (b)'s sentence, which is already scanned at (b)'s own URN.
        assert!(
            !prose.contains_key("/us/usc/t1/s7"),
            "a section with subdivisions is never keyed — that would attribute \
             § 7(b)'s definition of \u{201C}State\u{201D} to § 7 itself"
        );
        // And (b) needs no entry either: it carries no `<heading>`, so its
        // own `lexical` already exposes the sentence.
        assert!(
            !prose.contains_key("/us/usc/t1/s7/b"),
            "a heading-less subdivision already exposes its prose via lexical"
        );
        assert_eq!(prose.len(), 1, "exactly the one genuinely unscanned body");
    }

    /// L1, END-TO-END: [`usc_runtime_ontology_with_defines`] grounds a
    /// `defines` edge on a SUBDIVISION-LESS section root whose own `lexical`
    /// is nothing but its heading.
    ///
    /// The fixture is 1 U.S.C. § 2 verbatim (byte-verified against
    /// `usc_title_1-pl-119-90.xml`) — a real, unmodified statutory
    /// definition the existing grammar already reduces ("The word "county"
    /// includes a parish, …": Dickerson's PARTIAL-definition `includes`
    /// frame, which
    /// [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)'s
    /// `is_partial_definition_verb` already recognizes). So this test
    /// isolates the PROJECTION-level L1 fix: nothing about the grammar
    /// changed, only whether the sentence was ever offered to it.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn usc_runtime_ontology_with_defines_grounds_a_subdivision_less_section() {
        use crate::cognitive::linguistics::english::english_loaded;
        use crate::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
        use crate::social::judicial::statute_structure::grounding::DEFINES_REL;
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const COUNTY_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 1</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t1">
      <num value="1">Title 1</num>
      <heading>GENERAL PROVISIONS</heading>
      <section identifier="/us/usc/t1/s2"><num value="2">§ 2.</num><heading> “County” as including “parish”, and so forth</heading><content>
<p class="indent0">The word “county” includes a parish, or any other equivalent subdivision of a State or Territory of the United States.</p>
</content></section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(COUNTY_SAMPLE).expect("parse the real § 2 fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        let lang = english_loaded();
        let verbnet = verbnet_classes_loaded();
        let onto = usc_runtime_ontology_with_defines(
            &usc,
            OntologyName::new_static("usc_flat_section_defines"),
            lang,
            verbnet,
            &mint_domain(),
        )
        .expect("the defines-grounded USC pipeline materializes");
        let section = onto
            .archive()
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t1/s2")
            .expect("the section survives grounding");
        assert!(
            section.edges.iter().any(|e| e.0.as_str() == DEFINES_REL),
            "the subdivision-less section grounds a `defines` edge via its \
             recovered body, even though its own lexical never carried it"
        );
    }

    /// AFTER the S1 fix, END-TO-END: [`usc_runtime_ontology_with_defines`]
    /// grounds a `defines` edge for a subdivision whose OWN `lexical` is
    /// nothing but its heading — the exact shadowing shape measured in the
    /// real corpus (a `<Term>.—` heading over a "The term … means …" body;
    /// the real recurring convention BR30/BR40/ER30/ER40 at 42 U.S.C. § 6291
    /// and CO/VOC/PM–10 at 42 U.S.C. § 7602 use, byte-verified against
    /// `usc_title_42-pl-119-90.xml`).
    ///
    /// The body text here is deliberately the SAME already-grammar-proven
    /// real sentence `usc_runtime_ontology_with_defines_grounds_a_real_definition`
    /// (above) already grounds (real text, `/us/usc/t15/s6603/h/6/A`), NOT
    /// BR30's own definiens — that isolates the PROJECTION/S1 fix under
    /// test from the SEPARATE, still-open grammar gaps (G4 PP-chains, G7
    /// unknown acronyms) that block the real BR30/CO/VOC sentences
    /// themselves (see `defines_prose_index_recovers_real_title_42_prose_a_heading_shadows`
    /// above for the real-sentence-shaped structural proof).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn usc_runtime_ontology_with_defines_grounds_a_heading_shadowed_subdivision() {
        use crate::cognitive::linguistics::english::english_loaded;
        use crate::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
        use crate::formal::meta::identifier_format::Identifier;
        use crate::social::judicial::statute_structure::grounding::DEFINES_REL;
        use pr4xis::codegen_data::CodegenData;

        use super::super::{SubdivisionKind, UscSectionAux};

        static SHADOWED_DEFINES_SAMPLE: CodegenData<UsCode> = CodegenData {
            entity_count: 1,
            entity_ids: &["/us/usc/t42/s6291"],
            entity_kind: &["section"],
            entity_labels: &["Definitions"],
            entity_defs: &[""],
            word_index: &[],
            taxonomy: &[],
            mereology: &[],
            opposition: &[],
            equivalence: &[],
            causation: &[],
            references: &[],
        };

        let aux = alloc::vec![UscSectionAux {
            urn: "/us/usc/t42/s6291".to_string(),
            subdivisions: alloc::vec![UscSubdivision {
                urn: Identifier::uslm_urn("/us/usc/t42/s6291/55/B").expect("valid USLM URN"),
                kind: SubdivisionKind::Subparagraph,
                num: "B".to_string(),
                // The real `<Term>.—` heading convention (BR30.—, CO.—, VOC.—, …).
                heading: Some("Consumer.\u{2014}".to_string()),
                chapeau: None,
                // Deliberately the proven "consumer" sentence, not BR30's own
                // (separately G4/G7-blocked) definiens — see this test's doc.
                content: Some(
                    "The term \u{201C}consumer\u{201D} means a natural person.".to_string()
                ),
                children: Vec::new(),
                refs: Vec::new(),
            }],
            relations: Vec::new(),
            refs: Vec::new(),
        }];
        let usc = UsCode::from_codegen_with_aux(&SHADOWED_DEFINES_SAMPLE, &aux);

        // Confirm the shadowing is real before checking the pipeline.
        let raw = project_archive(&usc);
        let shadowed_node = raw
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t42/s6291/55/B")
            .expect("the shadowed subdivision projects");
        assert_eq!(
            shadowed_node.lexical.as_deref(),
            Some("Consumer.\u{2014}"),
            "the projected node's lexical is still heading-only"
        );

        let lang = english_loaded();
        let verbnet = verbnet_classes_loaded();
        let onto = usc_runtime_ontology_with_defines(
            &usc,
            OntologyName::new_static("usc_shadowed_defines"),
            lang,
            verbnet,
            &mint_domain(),
        )
        .expect("the defines-grounded USC pipeline materializes");
        let grounded_node = onto
            .archive()
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t42/s6291/55/B")
            .expect("the shadowed subdivision survives grounding");
        assert!(
            grounded_node
                .edges
                .iter()
                .any(|e| e.0.as_str() == DEFINES_REL),
            "the shadowed subdivision grounds a `defines` edge via the \
             recovered prose, even though its own lexical never carried it"
        );
    }

    // ---------------------------------------------------------------------------
    // G3+S2: `dangling_chapeau_reassembly_index` (the defines-lens gap
    // backlog's dangling-chapeau-enumeration + split-definition findings).
    // ---------------------------------------------------------------------------

    /// G3, the classic shape: a REAL flat 3-item enumeration — 42 U.S.C.
    /// § 300ii(1) "adult with a special need" (byte-verified against
    /// `usc_title_42-pl-119-90.xml`): chapeau "...requires care or
    /// supervision to—" (an em dash — an unresolved sentence), followed by
    /// three sibling subparagraphs (A)/(B)/(C). Asserts the index mints
    /// BOTH candidate kinds this module's own doc documents — the ALL-joined
    /// form and one PER-CHILD — attributed to the chapeau's OWN URN, exact
    /// text.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dangling_chapeau_reassembly_index_reassembles_a_real_flat_enumeration() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const ADULT_WITH_SPECIAL_NEED_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 42</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t42">
      <num value="42">Title 42</num>
      <heading>THE PUBLIC HEALTH AND WELFARE</heading>
      <section identifier="/us/usc/t42/s300ii">
        <num value="300ii">§ 300ii.</num>
        <heading>Definitions</heading>
        <paragraph identifier="/us/usc/t42/s300ii/1">
          <num value="1">(1)</num>
          <heading>Adult with a special need</heading>
          <chapeau>The term “adult with a special need” means a person 18 years of age or older who requires care or supervision to—</chapeau>
          <subparagraph identifier="/us/usc/t42/s300ii/1/A">
            <num value="A">(A)</num>
            <content>meet the person’s basic needs;</content>
          </subparagraph>
          <subparagraph identifier="/us/usc/t42/s300ii/1/B">
            <num value="B">(B)</num>
            <content>prevent physical self-injury or injury to others; or</content>
          </subparagraph>
          <subparagraph identifier="/us/usc/t42/s300ii/1/C">
            <num value="C">(C)</num>
            <content>avoid placement in an institutional facility.</content>
          </subparagraph>
        </paragraph>
      </section>
    </title>
  </main>
</uscDoc>
"##;
        let title =
            read_uslm_title(ADULT_WITH_SPECIAL_NEED_SAMPLE).expect("parse the real fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        let index = dangling_chapeau_reassembly_index(&usc);
        let candidates = index
            .get("/us/usc/t42/s300ii/1")
            .expect("the dangling chapeau is indexed under its OWN URN");
        assert_eq!(
            candidates.len(),
            4,
            "1 ALL-joined + 3 PER-CHILD candidates; got {candidates:?}"
        );
        assert_eq!(
            candidates[0],
            "The term \u{201C}adult with a special need\u{201D} means a person 18 years \
             of age or older who requires care or supervision to\u{2014} meet the \
             person\u{2019}s basic needs; prevent physical self-injury or injury to \
             others; or avoid placement in an institutional facility.",
            "the ALL-joined candidate is chapeau + every child, in document order"
        );
        assert_eq!(
            candidates[1],
            "The term \u{201C}adult with a special need\u{201D} means a person 18 years \
             of age or older who requires care or supervision to\u{2014} meet the \
             person\u{2019}s basic needs;",
            "the first PER-CHILD candidate is chapeau + subparagraph (A) alone"
        );
        assert_eq!(
            candidates[3],
            "The term \u{201C}adult with a special need\u{201D} means a person 18 years \
             of age or older who requires care or supervision to\u{2014} avoid placement \
             in an institutional facility.",
            "the last PER-CHILD candidate is chapeau + subparagraph (C) alone"
        );
        // The section root itself carries no chapeau — never indexed.
        assert!(!index.contains_key("/us/usc/t42/s300ii"));
    }

    /// G3, the nested shape: 42 U.S.C. § 15002(8)(A) "developmental
    /// disability" (byte-verified against `usc_title_42-pl-119-90.xml`) —
    /// its OWN chapeau ends "...an individual that—" over 5 clauses (i)–(v),
    /// and clause (iv) ITSELF carries a nested chapeau ("...major life
    /// activity:", a COLON, not a dash) over 7 subclauses (I)–(VII). Both
    /// URNs are indexed independently — the recursive walk does not stop at
    /// the first dangling ancestor.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dangling_chapeau_reassembly_index_recurses_into_a_real_nested_dangling_chapeau() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const DEVELOPMENTAL_DISABILITY_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 42</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t42">
      <num value="42">Title 42</num>
      <heading>THE PUBLIC HEALTH AND WELFARE</heading>
      <section identifier="/us/usc/t42/s15002">
        <num value="15002">§ 15002.</num>
        <heading>Definitions</heading>
        <paragraph identifier="/us/usc/t42/s15002/8">
          <num value="8">(8)</num>
          <heading>Developmental disability</heading>
          <subparagraph identifier="/us/usc/t42/s15002/8/A">
            <num value="A">(A)</num>
            <heading>In general</heading>
            <chapeau>The term “developmental disability” means a severe, chronic disability of an individual that—</chapeau>
            <clause identifier="/us/usc/t42/s15002/8/A/i">
              <num value="i">(i)</num>
              <content>is attributable to a mental or physical impairment or combination of mental and physical impairments;</content>
            </clause>
            <clause identifier="/us/usc/t42/s15002/8/A/ii">
              <num value="ii">(ii)</num>
              <content>is manifested before the individual attains age 22;</content>
            </clause>
            <clause identifier="/us/usc/t42/s15002/8/A/iii">
              <num value="iii">(iii)</num>
              <content>is likely to continue indefinitely;</content>
            </clause>
            <clause identifier="/us/usc/t42/s15002/8/A/iv">
              <num value="iv">(iv)</num>
              <chapeau>results in substantial functional limitations in 3 or more of the following areas of major life activity:</chapeau>
              <subclause identifier="/us/usc/t42/s15002/8/A/iv/I">
                <num value="I">(I)</num>
                <content>Self-care.</content>
              </subclause>
              <subclause identifier="/us/usc/t42/s15002/8/A/iv/II">
                <num value="II">(II)</num>
                <content>Receptive and expressive language.</content>
              </subclause>
            </clause>
            <clause identifier="/us/usc/t42/s15002/8/A/v">
              <num value="v">(v)</num>
              <content>reflects the individual’s need for a combination and sequence of special, interdisciplinary, or generic services, individualized supports, or other forms of assistance that are of lifelong or extended duration and are individually planned and coordinated.</content>
            </clause>
          </subparagraph>
        </paragraph>
      </section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(DEVELOPMENTAL_DISABILITY_SAMPLE)
            .expect("parse the real developmental-disability fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        let index = dangling_chapeau_reassembly_index(&usc);
        let outer = index
            .get("/us/usc/t42/s15002/8/A")
            .expect("the outer dangling chapeau (5 clauses) is indexed");
        assert_eq!(outer.len(), 6, "1 ALL-joined + 5 PER-CHILD; got {outer:?}");
        assert!(
            outer[0].starts_with(
                "The term \u{201C}developmental disability\u{201D} means a severe, \
                 chronic disability of an individual that\u{2014} is attributable to"
            ),
            "the ALL-joined candidate starts with the chapeau + first clause; got {:?}",
            outer[0]
        );

        let inner = index
            .get("/us/usc/t42/s15002/8/A/iv")
            .expect("the NESTED dangling chapeau (colon, 2 subclauses) is ALSO indexed");
        assert_eq!(inner.len(), 3, "1 ALL-joined + 2 PER-CHILD; got {inner:?}");
        assert_eq!(
            inner[0],
            "results in substantial functional limitations in 3 or more of the \
             following areas of major life activity: Self-care. Receptive and \
             expressive language.",
            "the nested chapeau reassembles independently of its ancestor"
        );
    }

    /// S2: the REAL split means/includes definition — 42 U.S.C.
    /// § 1396n(c)(5) "habilitation services" (byte-verified against
    /// `usc_title_42-pl-119-90.xml`), this task's own headline example: the
    /// chapeau carries ONLY the term + em dash (no verb at all — "For
    /// purposes of paragraph (4)(B), the term "habilitation services"—"),
    /// subparagraph (A) opens with "means", subparagraph (B) opens with
    /// "includes", and subparagraph (C) is ITSELF a further-nested dangling
    /// chapeau ("does not include—"). Structurally this is the exact same
    /// "chapeau + children" shape as G3 — asserts the PER-CHILD candidates
    /// recover BOTH the "means" and "includes" continuations under the
    /// chapeau's own URN, never a synthetic node.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dangling_chapeau_reassembly_index_handles_the_real_habilitation_services_split_definition() {
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;

        const HABILITATION_SERVICES_SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 42</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t42">
      <num value="42">Title 42</num>
      <heading>THE PUBLIC HEALTH AND WELFARE</heading>
      <section identifier="/us/usc/t42/s1396n">
        <num value="1396n">§ 1396n.</num>
        <heading>Waivers</heading>
        <subsection identifier="/us/usc/t42/s1396n/c">
          <num value="c">(c)</num>
          <heading>Waiver for home or community-based services</heading>
          <paragraph identifier="/us/usc/t42/s1396n/c/5">
            <num value="5">(5)</num>
            <chapeau>For purposes of paragraph (4)(B), the term “habilitation services”—</chapeau>
            <subparagraph identifier="/us/usc/t42/s1396n/c/5/A">
              <num value="A">(A)</num>
              <content>means services designed to assist individuals in acquiring, retaining, and improving the self-help, socialization, and adaptive skills necessary to reside successfully in home and community based settings; and</content>
            </subparagraph>
            <subparagraph identifier="/us/usc/t42/s1396n/c/5/B">
              <num value="B">(B)</num>
              <content>includes (except as provided in subparagraph (C)) prevocational, educational, and supported employment services; but</content>
            </subparagraph>
            <subparagraph identifier="/us/usc/t42/s1396n/c/5/C">
              <num value="C">(C)</num>
              <chapeau>does not include—</chapeau>
              <clause identifier="/us/usc/t42/s1396n/c/5/C/i">
                <num value="i">(i)</num>
                <content>special education and related services which otherwise are available to the individual through a local educational agency; and</content>
              </clause>
            </subparagraph>
          </paragraph>
        </subsection>
      </section>
    </title>
  </main>
</uscDoc>
"##;
        let title = read_uslm_title(HABILITATION_SERVICES_SAMPLE)
            .expect("parse the real habilitation-services fixture");
        let usc = UsCode::from_uslm_titles_owned(alloc::vec![title]);

        let index = dangling_chapeau_reassembly_index(&usc);
        let candidates = index.get("/us/usc/t42/s1396n/c/5").expect(
            "the split definition's chapeau is indexed under its OWN URN — never a synthetic node",
        );
        assert_eq!(
            candidates.len(),
            4,
            "1 ALL-joined + 3 PER-CHILD (A/B/C); got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .any(|c| c.contains("means services designed to assist")),
            "one PER-CHILD candidate recovers subparagraph (A)'s \"means\" continuation; got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .any(|c| c.contains("includes (except as provided in subparagraph (C))")),
            "another PER-CHILD candidate recovers subparagraph (B)'s \"includes\" continuation; got {candidates:?}"
        );
        // Subparagraph (C) is ITSELF a nested dangling chapeau ("does not
        // include—") — indexed independently under its OWN URN too.
        let nested = index
            .get("/us/usc/t42/s1396n/c/5/C")
            .expect("the nested \"does not include—\" chapeau is ALSO indexed");
        assert_eq!(
            nested.len(),
            1,
            "1 ALL-joined == 1 PER-CHILD (single clause); got {nested:?}"
        );
    }

    /// END-TO-END WIRING PROOF: reassembly + `defines_lens` genuinely
    /// composes, isolated from the SEPARATELY-scoped grammar gaps (G1
    /// fronted adjuncts, G4 relative clauses) that block every REAL
    /// dangling-chapeau candidate measured above — mirroring
    /// `usc_runtime_ontology_with_defines_grounds_a_heading_shadowed_subdivision`'s
    /// own established precedent exactly: split the ALREADY grammar-proven
    /// "consumer" sentence
    /// (`recognizes_the_term_x_means_y_from_real_statute_prose`) across a
    /// dangling chapeau + one child, rather than inventing new prose. This
    /// is real statutory text, merely re-split at the SAME em-dash boundary
    /// real USC drafting uses (see `dangling_chapeau_reassembly_index`'s own
    /// doc: "the term X—" / "means Y" is exactly S2's shape).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn usc_runtime_ontology_with_defines_grounds_a_definition_recovered_via_reassembly() {
        use crate::cognitive::linguistics::english::english_loaded;
        use crate::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
        use crate::formal::meta::identifier_format::Identifier;
        use crate::social::judicial::statute_structure::grounding::{
            DEFINES_REL, DefinitionExhaustiveness, defines_pointers,
        };
        use pr4xis::codegen_data::CodegenData;

        use super::super::{SubdivisionKind, UscSectionAux};

        static REASSEMBLY_SAMPLE: CodegenData<UsCode> = CodegenData {
            entity_count: 1,
            entity_ids: &["/us/usc/t15/s6603"],
            entity_kind: &["section"],
            entity_labels: &["Definitions"],
            entity_defs: &[""],
            word_index: &[],
            taxonomy: &[],
            mereology: &[],
            opposition: &[],
            equivalence: &[],
            causation: &[],
            references: &[],
        };
        let aux = alloc::vec![UscSectionAux {
            urn: "/us/usc/t15/s6603".to_string(),
            subdivisions: alloc::vec![UscSubdivision {
                urn: Identifier::uslm_urn("/us/usc/t15/s6603/h/6/A").expect("valid USLM URN"),
                kind: SubdivisionKind::Subparagraph,
                num: "A".to_string(),
                heading: None,
                // The dangling half — the SAME em-dash "sandwich" boundary
                // real USC drafting uses (see this test's own doc).
                chapeau: Some("The term \u{201C}consumer\u{201D}\u{2014}".to_string()),
                content: None,
                children: alloc::vec![UscSubdivision {
                    urn: Identifier::uslm_urn("/us/usc/t15/s6603/h/6/A/i").expect("valid USLM URN"),
                    kind: SubdivisionKind::Clause,
                    num: "i".to_string(),
                    heading: None,
                    chapeau: None,
                    content: Some("means a natural person.".to_string()),
                    children: Vec::new(),
                    refs: Vec::new(),
                }],
                refs: Vec::new(),
            }],
            relations: Vec::new(),
            refs: Vec::new(),
        }];
        let usc = UsCode::from_codegen_with_aux(&REASSEMBLY_SAMPLE, &aux);

        // Confirm the split is genuinely unparseable PER-NODE first — the
        // honest baseline this fix improves on.
        assert!(
            defines_pointers(
                "The term \u{201C}consumer\u{201D}\u{2014}",
                english_loaded(),
                english_loaded(),
                verbnet_classes_loaded(),
                &mint_domain(),
            )
            .is_empty(),
            "the dangling chapeau alone carries no verb — no per-node pointer"
        );

        let lang = english_loaded();
        let verbnet = verbnet_classes_loaded();
        let onto = usc_runtime_ontology_with_defines(
            &usc,
            OntologyName::new_static("usc_reassembled_defines"),
            lang,
            verbnet,
            &mint_domain(),
        )
        .expect("the defines-grounded USC pipeline materializes");
        let grounded_node = onto
            .archive()
            .nodes
            .iter()
            .find(|n| n.name == "/us/usc/t15/s6603/h/6/A")
            .expect("the dangling chapeau's OWN URN survives grounding");
        assert!(
            grounded_node
                .edges
                .iter()
                .any(|e| e.0.as_str() == DEFINES_REL),
            "the dangling chapeau grounds a `defines` edge via its reassembled \
             candidate, attributed to its OWN (not a synthetic) URN"
        );

        // The recovered pointer itself is Exhaustive ("means").
        let recovered = defines_pointers(
            "The term \u{201C}consumer\u{201D}\u{2014} means a natural person.",
            lang,
            lang,
            verbnet,
            &mint_domain(),
        );
        assert_eq!(recovered.len(), 1, "got {recovered:?}");
        assert_eq!(
            recovered[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
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
