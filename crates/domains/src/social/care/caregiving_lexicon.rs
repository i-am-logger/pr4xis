//! Family Caregiving Lexicon (`caregiving_lexicon@2026`) — runtime loader.
//!
//! Wraps the WN-LMF bundle at
//! `crates/domains/data/care/caregiving_lexicon.xml` (registered in
//! `praxis.toml` as `caregiving_lexicon@2026`, kind `CaregivingLexicon`,
//! pinned in `praxis.lock`) as a chat-composable
//! [`RuntimeOntology`](pr4xis_runtime::ontology::RuntimeOntology): one
//! praxis `Concept` per caregiving concept, glossed by ONE `Definition`
//! whose authorities ride the SEPARATE `dcterms:source` provenance channel
//! (never the gloss text), lexicalized by
//! `canonicalForm` edges to one `ontolex:Form` atom per written form
//! (canonical lemma AND every synonym surface), with the verified isA
//! taxonomy as `Subsumption` edges (respite care → home and
//! community-based services, ...).
//!
//! # Literature
//!
//! Per the bundled XML's header — each synset's own authorities are carried
//! STRUCTURALLY, in its WN-LMF `dc:source` attribute, and reach a reader
//! through the
//! [`lexicon_provenance`](crate::applied::data_provisioning::lexicon_provenance)
//! channel rather than as prose inside the gloss. The high-level sources are:
//!
//! - **Social Security Act Titles XIX / XVIII** (42 U.S.C. § 1396-1 / §
//!   1395c) — the bare program headwords "medicaid"/"medicare" (verified
//!   against Cornell LII 2026-07-20; not in the loaded
//!   `usc_title_42@pl-119-90` bundle, which carries only sections a
//!   specific Public Law amended).
//! - **Loaded U.S. Code Title 42** (`usc_title_42@pl-119-90`) — the
//!   statutorily-defined terms (42 USC 300ii, 1395i-3, 1395n, 1395x,
//!   1396n, 1396p, 1396r, 1396u-4, 3002, 3022, 3030s, 3030s-1, 11225,
//!   15002), each gloss the statute's own wording trimmed to a
//!   self-contained sentence, verified against the on-disk corpus XML.
//! - **42 CFR** 409, 418, 435, 438, 440, 441, 484; **45 CFR** 1321.3 —
//!   Medicaid HCBS waiver, Medicare home health / hospice, and OAA
//!   service definitions; CMS 1915(c) Waiver Technical Guide v3.6
//!   (Jan. 2019), Appendix C.
//! - **National Institute on Aging** health topic pages — dementia-care
//!   clinical vocabulary; CMS State Operations Manual App. PP F-689.
//! - **ACL, Alternatives to Guardianship** (acl.gov) — guardianship /
//!   supported decision-making.
//! - **26 USC 7702B(c)(2)** — functional-assessment terms (Title 26 is
//!   not a loaded USC title, hence carried here).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

/// The registered source name (`praxis.toml` `[sources.caregiving_lexicon]`).
pub const CAREGIVING_LEXICON_NAME: &str = "caregiving_lexicon";

/// The registered source version (`caregiving_lexicon@2026`, pinned in
/// `praxis.lock`; equals the bundle's `<Lexicon version>`).
pub const CAREGIVING_LEXICON_VERSION: &str = "2026";

/// The committed `.prx` — the content-addressed envelope carrying the
/// authored lexicon. The raw `.xml` is the git-tracked source-of-truth but
/// is EXCLUDED from the published crate; only this `.prx` ships,
/// materialized through the generalized feature-light
/// `[compact_archive_signatures]` gate — the `us_legal_lexicon` shape.
/// `pub` so the registry-driven residency dispatch
/// ([`chat_lexicons`](crate::applied::data_provisioning::chat_lexicons))
/// can map this source's taxonomy kind to its committed bundle.
pub const CAREGIVING_LEXICON_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/care/caregiving_lexicon.prx"
));

/// The registered `caregiving_lexicon@2026` source as a chat-composable
/// [`RuntimeOntology`](pr4xis_runtime::ontology::RuntimeOntology) — the
/// generic WN-LMF lexicon bridge
/// ([`lexicon_runtime_ontology`](crate::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology))
/// parameterized by THIS source's registered `(name, version, committed
/// .prx)`. No lexicon-specific projection code.
///
/// Composing the materialized ontology under the `ComposedReasoner` makes
/// every written form ("respite care", "adult day care", "sundowning") a
/// queryable surface resolving to its concept's cited gloss, and every
/// verified isA edge answerable as a loaded-parent ("is respite care a
/// home and community-based service").
#[cfg(feature = "std")]
pub fn caregiving_lexicon_runtime_ontology() -> Result<
    pr4xis_runtime::ontology::RuntimeOntology,
    crate::cognitive::linguistics::english::bridge::LexiconBridgeError,
> {
    crate::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology(
        CAREGIVING_LEXICON_NAME,
        CAREGIVING_LEXICON_VERSION,
        CAREGIVING_LEXICON_PRX,
    )
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::english::bridge::{FORM_KIND, project_lexicon_archive};
    use pr4xis_runtime::archive::Archive;
    use pr4xis_runtime::definition::SOURCE_KIND;

    /// The REAL registered lexicon, loaded once and projected once — the
    /// `(English, raw lexicalized Archive)` pair the structural tests read
    /// (parse-once, query-many; the `us_legal_lexicon` test shape).
    fn lexicon_projection() -> &'static (English, Archive) {
        static PROJ: OnceLock<(English, Archive)> = OnceLock::new();
        PROJ.get_or_init(|| {
            let xml = crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
                CAREGIVING_LEXICON_NAME,
                CAREGIVING_LEXICON_VERSION,
                CAREGIVING_LEXICON_PRX,
            );
            let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(&xml)
                .expect("the registered caregiving_lexicon parses as WN-LMF");
            let english = English::from_wordnet(&wn);
            let archive = project_lexicon_archive(&english);
            (english, archive)
        })
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_lexicon_projection_is_referentially_closed() {
        // Every projected edge — canonicalForm lexicalization AND the
        // hypernym taxonomy (this lexicon, unlike us_legal_lexicon, DOES
        // declare SynsetRelations) — names a DECLARED node: the closure
        // `materialize` will later enforce, asserted here directly on the
        // raw projection so a regression is caught at the projection.
        let (_, archive) = lexicon_projection();
        let declared: alloc::collections::BTreeSet<&str> =
            archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                let to = target
                    .local_name()
                    .expect("a projected lexicon edge is a same-archive local edge");
                assert!(
                    declared.contains(to),
                    "edge {}--{kind}-->{to} names an undeclared node",
                    n.name
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_written_form_resolves_and_mints_exactly_one_form_atom() {
        // The queryable surface of the materialized ontology is EXACTLY the
        // lexicon's written forms: every form (canonical lemma and synonym
        // surface alike) mints one `ontolex:Form` atom.
        let (english, archive) = lexicon_projection();
        let mut count = 0usize;
        for w in english.word_index.words() {
            assert!(
                !english.lookup(w).is_empty(),
                "written form {w:?} must resolve to at least one concept"
            );
            let atoms = archive
                .nodes
                .iter()
                .filter(|n| n.kind == FORM_KIND && n.name == w)
                .count();
            assert_eq!(
                atoms, 1,
                "written form {w:?} must mint exactly one Form atom"
            );
            count += 1;
        }
        assert!(
            count >= 100,
            "the authored lexicon carries 139 surfaces; loaded only {count}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_registered_lexicon_materializes_as_a_runtime_ontology() {
        let onto = caregiving_lexicon_runtime_ontology()
            .expect("the registered caregiving_lexicon materializes");
        assert_eq!(
            onto.id().as_str(),
            CAREGIVING_LEXICON_NAME,
            "the ontology is named after its registered source"
        );
        let archive = onto
            .to_owned_archive()
            .expect("the materialized archive round-trips");
        // The english_functor relabeled every raw Synset generator into the
        // praxis Concept kind; the two other kinds are the ontolex:Form surface
        // atom and the dcterms:BibliographicResource citation atom the
        // provenance join appends.
        for n in &archive.nodes {
            assert!(
                n.kind == "Concept" || n.kind == FORM_KIND || n.kind == SOURCE_KIND,
                "unexpected node kind {:?} on {:?}",
                n.kind,
                n.name
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn a_caregiving_terms_written_form_defines_through_the_composed_reasoner() {
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use alloc::rc::Rc;

        let onto = caregiving_lexicon_runtime_ontology()
            .expect("the registered caregiving_lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        // The define-style probe: a known term's written form — including a
        // SYNONYM surface sharing its canonical's synset — resolves through
        // the composed surface union to a LOADED concept. The gloss is the
        // exact text a caregiver is recited; the citation reaches the same
        // reader through the SEPARATE `definition_sources` channel, and the
        // two must not be confusable — so each assertion is made twice, once
        // per channel, in opposite directions.
        for (surface, cited_in) in [
            ("respite care", "42 USC 300ii(7)"),
            // synonym surface → SAME synset, SAME declared authority
            ("respite", "42 USC 300ii(7)"),
            (
                "adult day care",
                "CMS 1915(c) Waiver Technical Guide v3.6 (Jan. 2019), Appendix C (Adult Day Health) (elaborated definition)",
            ),
            ("family caregiver", "42 USC 3022(3)"),
            (
                "sundowning",
                "NIA, Coping With Agitation, Aggression, and Sundowning in Alzheimer's Disease",
            ),
        ] {
            let loaded = composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("{surface:?} must resolve to a loaded lexicon concept"));
            let concept = composed
                .concept(loaded)
                .expect("the loaded concept's view resolves");
            let gloss = concept
                .definitions()
                .next()
                .unwrap_or_else(|| panic!("{surface:?}'s definition must be reachable"));
            let sources: alloc::vec::Vec<&str> = concept.definition_sources().iter().collect();
            assert!(
                sources.contains(&cited_in),
                "{surface:?} must SURFACE its authority {cited_in:?} through the \
                 provenance channel; got {sources:?}"
            );
            assert!(
                !gloss.contains(cited_in),
                "{surface:?}'s recited gloss must NOT carry the citation as prose; \
                 got {gloss:?}"
            );
        }

        // The other direction: a WordNet substrate concept — one this lexicon
        // does not define — surfaces NO provenance. A lexicographic gloss is
        // derived from no document, and inventing one would be the fabricated
        // provenance this channel exists to prevent.
        let substrate = composed
            .lookup("dog")
            .iter()
            .copied()
            .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::English(_))))
            .expect("the English substrate resolves its own word");
        assert!(
            composed
                .concept(substrate)
                .expect("the substrate concept's view resolves")
                .definition_sources()
                .is_empty(),
            "a WordNet gloss cites no document and must claim none"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_verified_taxonomy_answers_is_a_through_loaded_parents() {
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use alloc::rc::Rc;

        let onto = caregiving_lexicon_runtime_ontology()
            .expect("the registered caregiving_lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        let loaded_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("{surface:?} must resolve to a loaded lexicon concept"))
        };

        // The verified isA edges (each citation-backed in the term data):
        // "is respite care a home and community-based service" and the
        // workforce taxonomy both answer from loaded parents.
        for (child, parent) in [
            ("respite care", "home and community-based services"),
            ("homemaker services", "home and community-based services"),
            ("personal care attendant", "direct care worker"),
            ("older relative caregiver", "caregiver"),
        ] {
            let c = loaded_id(child);
            let p = loaded_id(parent);
            assert!(
                composed.parents(c).contains(&p),
                "{child:?} must have {parent:?} among its loaded parents"
            );
        }
    }
}
