//! Definition provenance for WN-LMF lexicon sources — the `dc:source` join.
//!
//! ## The gap this closes
//!
//! An authored domain lexicon glosses each term from a real document: the
//! caregiving lexicon's `respite care` gloss is 42 U.S.C. § 300ii(7)'s own
//! words. Until this module existed that citation rode INSIDE the gloss string
//! (`"… of that child or adult — 42 USC 300ii(7)"`), which made it invisible to
//! praxis's own reasoning: nothing could inspect it, tell it apart from prose
//! that happens to contain digits, verify it against the loaded corpus, or
//! distinguish it from the ontologies a query actually reasoned OVER. A reader
//! saw a statute-looking answer whose provenance panel named only a lexicon —
//! literally correct and thoroughly misleading.
//!
//! WN-LMF already has the right slot. `<!ATTLIST Synset dc:source CDATA
//! #IMPLIED>` (WN-LMF DTD line 113) is parsed into
//! [`Synset::dc_source`](crate::social::software::markup::xml::lmf::ontology::Synset::dc_source)
//! and written back out by the LMF writer, so the format needs nothing new.
//! What was missing is a READ: a path from that field to an answer. This module
//! is that path.
//!
//! ## The join, and why it is on the synset id
//!
//! [`English::from_wordnet`](crate::cognitive::linguistics::english::English::from_wordnet)
//! projects a `Synset` to a `Concept` carrying `original_id = synset.id`, and
//! [`project_lexicon_archive`] names each archive node by that same
//! `original_id`. The synset id is therefore carried UNCHANGED from the XML
//! attribute to the loaded `.prx` node name — and `apply` copies a node's name
//! verbatim through the functor. So the LMF synset id is a key that already
//! spans the whole pipeline, and the provenance can be joined onto the projected
//! archive without widening `Concept` or `English` at all.
//!
//! That matters beyond taste: `Concept`, `English::from_wordnet` and
//! `project_lexicon_archive` are all members of the defines-overlay code
//! dependency closure
//! ([`DEFINES_GRAMMAR_CLOSURE_FILES`](super::defines_grammar_signature::DEFINES_GRAMMAR_CLOSURE_FILES)),
//! whose fingerprint gates a ~3.3-hour corpus rebake. Joining on the id — rather
//! than threading a field through them — reaches the same place for free, by
//! CALLING that machinery instead of editing it.
//!
//! ## The wire shape
//!
//! One `dcterms:BibliographicResource` atom
//! ([`SOURCE_KIND`](pr4xis_runtime::definition::SOURCE_KIND)) per DISTINCT
//! citation, plus one
//! `dcterms:source` edge ([`DEFINITION_SOURCE_REL`]) from each citing synset
//! node to it. A graph, not a delimited string: a definition authored from two
//! statutes is two edges, and a citation shared by forty terms is one atom.
//! Both constants are wire DATA in `pr4xis-runtime` beside `ontolex:Form`, for
//! the same reason — producer (here) and consumer (the composed reasoner's
//! [`ConceptView::Loaded`](crate::cognitive::linguistics::english::ConceptView)
//! provenance channel) must agree on the exact strings.
//!
//! ## Literature
//!
//! - **DCMI Metadata Terms** (2020-01-20) — `dcterms:source`, "a related
//!   resource from which the described resource is derived", and
//!   `dcterms:BibliographicResource`, "a book, article, or other documentary
//!   resource". The relation and the node kind are that vocabulary's, not
//!   invented here; the same standard-vocabulary-as-relation-kind-authority
//!   discipline `dcterms:replaces` already follows in
//!   [`Relations`](crate::formal::relations::ontology).
//! - **Global WordNet Association, WN-LMF DTD 1.1/1.3** — `dc:source` on
//!   `Synset` (line 113), the authored slot this reads.
//! - **The Bluebook: A Uniform System of Citation, 21st ed. (2020) §1.4** —
//!   string citations are separated by a semicolon, which is why
//!   [`CITATION_SEPARATOR`] is `"; "`: the authored data already writes multiple
//!   authorities that way, so the on-disk convention is the citation manual's,
//!   not a new one minted for this field.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use alloc::collections::{BTreeMap, BTreeSet};

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::{DEFINITION_SOURCE_REL, EdgeTarget, source_atom};
use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::english::bridge::{
    LexiconBridgeError, english_functor, project_lexicon_archive,
};
use crate::social::software::markup::xml::lmf::WordNet;

/// The separator between several authorities in ONE `dc:source` attribute —
/// Bluebook §1.4's string-citation semicolon, with its following space.
///
/// The space is load-bearing and deliberate: a citation may itself contain a
/// bare semicolon (`Psychopharmacology Bulletin 1988;24(4):653-659`), so
/// splitting on `';'` alone would shred a single authority into fragments.
/// `"; "` is the form the authored lexicons already use between authorities.
pub const CITATION_SEPARATOR: &str = "; ";

/// The two separators the shipped lexicons used to set a provenance citation
/// off from its definition before this module existed — an em-dash (U+2014) and
/// an ASCII double hyphen, each spaced. Named here, and ONLY here, because the
/// ratchet
/// (`no_registered_lexicon_reintroduces_an_inline_citation`) has to name the
/// retired shape in order to refuse it. Nothing on the read path consults them.
pub const RETIRED_CITATION_SEPARATORS: [&str; 2] = [" \u{2014} ", " -- "];

/// The authorities one `dc:source` attribute names, in authored order, with
/// empty segments dropped.
///
/// This is a DECODE of a declared attribute format, not a parse of natural
/// language: the value is authored data in a documented convention
/// ([`CITATION_SEPARATOR`]), the same way `Synset.members` is a space-separated
/// id list. Nothing here recognizes what a citation looks like.
///
/// The split is PARENTHESIS-DEPTH aware, and that is not a refinement — it is
/// what makes the convention lossless. A single authority routinely carries a
/// parenthetical gloss containing its own semicolon (`CRS Report R44948
/// (Sept. 14, 2017; updated May 16, 2018)`, and 10 more like it across the two
/// shipped lexicons). A depth-blind split would shred those into fragments, and
/// the alternative — editing the punctuation of authored citations so a naive
/// splitter copes — would be the tool dictating the record. Depth-0 splitting
/// keeps every citation byte-exact as its author wrote it.
pub fn citations_of(dc_source: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let bytes = dc_source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b';' if depth == 0 && dc_source[i..].starts_with(CITATION_SEPARATOR) => {
                out.push(dc_source[start..i].trim());
                i += CITATION_SEPARATOR.len();
                start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(dc_source[start..].trim());
    out.retain(|c: &&str| !c.is_empty());
    out
}

/// Join every synset's `dc:source` onto the projected lexicon `archive`, in
/// place — the whole of this module's mechanism.
///
/// For each synset that declares a source, the archive node NAMED BY THAT
/// SYNSET'S ID gains one [`DEFINITION_SOURCE_REL`] edge per cited authority,
/// and each distinct authority is appended once as a
/// [`SOURCE_KIND`](pr4xis_runtime::definition::SOURCE_KIND) atom (so the
/// archive stays referentially closed — `materialize` rejects an edge whose
/// local target is not a declared node).
///
/// A synset with no `dc:source` is untouched: its node's edges — and therefore
/// its content address — are byte-identical to the provenance-free projection.
/// This is the same additive discipline the antonym edges follow in
/// [`synset_definition`](crate::cognitive::linguistics::english::bridge::synset_definition),
/// and it is what makes "a definition without a source gains no spurious one" a
/// structural property rather than a test that happens to pass.
pub fn attach_definition_provenance(archive: &mut Archive, wn: &WordNet) {
    // name → node position. `project_lexicon_archive` emits one node per
    // concept, named by its `original_id` — which IS the synset id, carried
    // through `English::from_wordnet` unchanged. The join key is therefore the
    // id both sides already agree on, never a re-derived string.
    // Resolved in a scope that ENDS before the archive is written to: the walk
    // below only needs `(node position, citation)` pairs, and the citations are
    // borrowed from `wn`, not from the archive.
    let cited: Vec<(usize, &str)> = {
        let position: BTreeMap<&str, usize> = archive
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.name.as_str(), i))
            .collect();
        wn.synsets
            .iter()
            .filter_map(|synset| {
                let dc_source = synset.dc_source.as_deref()?;
                // A `dc:source` on a synset the projection dropped (a synset
                // with no concept) has nothing to hang on. Skipping is the
                // honest open-world reading: no node claims that provenance.
                let &pos = position.get(synset.id.as_str())?;
                Some(citations_of(dc_source).into_iter().map(move |c| (pos, c)))
            })
            .flatten()
            .collect()
    };

    // The distinct authorities, minted as atoms AFTER the edge walk so an
    // appended atom cannot shadow a concept node in the join above. Ordered
    // (BTreeSet) so the emitted archive — and hence its Merkle root — does not
    // depend on synset iteration order.
    let mut atoms: BTreeSet<&str> = BTreeSet::new();
    for (pos, citation) in cited {
        atoms.insert(citation);
        let edge = (
            DEFINITION_SOURCE_REL.to_string(),
            EdgeTarget::Local(citation.to_string()),
        );
        // A repeated authority within one attribute adds no second edge:
        // `Definition::address` dedups edge rows anyway, so keeping the vector
        // deduped here just keeps the in-memory shape honest.
        if !archive.nodes[pos].edges.contains(&edge) {
            archive.nodes[pos].edges.push(edge);
        }
    }

    for citation in atoms {
        archive.nodes.push(source_atom(citation));
    }
}

/// Bridge WN-LMF lexicon TEXT into a chat-composable [`RuntimeOntology`] that
/// CARRIES ITS DEFINITION PROVENANCE — the provenance-aware peer of
/// [`lexicon_runtime_ontology_from_lmf`](crate::cognitive::linguistics::english::bridge::lexicon_runtime_ontology_from_lmf):
///
/// ```text
/// read_wordnet → English::from_wordnet → project_lexicon_archive
///              → attach_definition_provenance   ← the one added step
///              → apply(english_functor) → materialize(name)
/// ```
///
/// The added step sits between projection and relabeling on purpose. `apply`
/// carries an unmapped generator through under its own name (the open-world
/// stance its own doc states), so the `dcterms:source` edges and
/// `dcterms:BibliographicResource` atoms survive the functor unchanged, exactly
/// as the `ontolex:Form` lexicalization channel already does.
///
/// Every registered definitional lexicon takes THIS loader; the bridge's
/// provenance-free peer remains the generic WN-LMF core it is built from.
pub fn lexicon_runtime_ontology_from_lmf(
    lmf_xml: &str,
    name: OntologyName,
) -> Result<RuntimeOntology, LexiconBridgeError> {
    let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(lmf_xml)
        .map_err(LexiconBridgeError::Lmf)?;
    let english = English::from_wordnet(&wn);
    let mut source = project_lexicon_archive(&english);
    attach_definition_provenance(&mut source, &wn);
    let praxis = apply(&english_functor().action, &source)
        .expect("the loaded english_functor is always a Functor action, which apply interprets");
    materialize(praxis, name).map_err(LexiconBridgeError::Materialize)
}

/// Bridge a REGISTERED, EMBEDDED WN-LMF lexicon source into a provenance-
/// carrying [`RuntimeOntology`] named after the source — the loader every
/// registered lexicon calls with its own `(name, version, committed .prx
/// bytes)`. Zero per-domain code: the source entry IS the parameterization.
///
/// The source text is admitted through the fail-closed
/// [`raw_source_text_embedded`](super::raw_source_prx::raw_source_text_embedded)
/// gate (praxis.lock `[compact_archive_signatures]` pin), then takes
/// [`lexicon_runtime_ontology_from_lmf`].
pub fn lexicon_runtime_ontology(
    name: &str,
    version: &str,
    embedded_prx: &[u8],
) -> Result<RuntimeOntology, LexiconBridgeError> {
    let xml = super::raw_source_prx::raw_source_text_embedded(name, version, embedded_prx);
    lexicon_runtime_ontology_from_lmf(&xml, OntologyName::new(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::bridge::SYNSET_KIND;
    use pr4xis_runtime::definition::SOURCE_KIND;

    /// A two-synset lexicon: one term glossed FROM a cited statute, one glossed
    /// with no source at all — the both-directions fixture every test here uses.
    const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" email="" license="CC0-1.0" version="1" url="">
    <LexicalEntry id="e-r"><Lemma writtenForm="respite care" partOfSpeech="n"/><Sense id="e-r-1" synset="s-respite"/></LexicalEntry>
    <LexicalEntry id="e-k"><Lemma writtenForm="kinship care" partOfSpeech="n"/><Sense id="e-k-1" synset="s-kinship"/></LexicalEntry>
    <Synset id="s-respite" partOfSpeech="n" dc:source="42 USC 300ii(7); 42 CFR 441.302">
      <Definition>Planned or emergency care to relieve a family caregiver.</Definition>
    </Synset>
    <Synset id="s-kinship" partOfSpeech="n">
      <Definition>Care provided by a relative — grandparent, aunt or uncle — to a child.</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

    /// The declared multi-authority convention decodes to its authorities and
    /// leaves a semicolon INSIDE one authority alone (the journal-volume case
    /// `1988;24(4)`, real in the caregiving lexicon).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn citations_decode_on_the_declared_separator_only() {
        assert_eq!(
            citations_of("42 USC 300ii(7); 42 CFR 441.302"),
            vec!["42 USC 300ii(7)", "42 CFR 441.302"]
        );
        // A bare semicolon inside one authority (a journal volume) is not a
        // separator — the separator carries a following space.
        assert_eq!(
            citations_of("Psychopharmacology Bulletin 1988;24(4):653-659"),
            vec!["Psychopharmacology Bulletin 1988;24(4):653-659"]
        );
        // …nor is a separator-shaped run INSIDE a parenthetical gloss: this is
        // `hc-social-security-disability-insurance`'s real third authority.
        assert_eq!(
            citations_of(
                "42 USC §§ 423(a)(1); CRS Report R44948 (Sept. 14, 2017; updated May 16, 2018)"
            ),
            vec![
                "42 USC §§ 423(a)(1)",
                "CRS Report R44948 (Sept. 14, 2017; updated May 16, 2018)"
            ]
        );
        assert!(citations_of("").is_empty());
    }

    /// A synset WITH `dc:source` gains one edge per authority and the atoms
    /// they target; a synset WITHOUT one gains nothing — the second half is the
    /// no-spurious-provenance direction, and it is checked on a definition whose
    /// prose contains em-dashes, the shape a dash-sniffing implementation would
    /// have mistaken for a citation.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn provenance_attaches_only_where_the_lexicon_declares_it() {
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(LMF)
            .expect("the fixture is well-formed WN-LMF");
        let english = English::from_wordnet(&wn);
        let mut archive = project_lexicon_archive(&english);
        let before = archive.nodes.len();
        attach_definition_provenance(&mut archive, &wn);

        let sourced = archive
            .nodes
            .iter()
            .find(|n| n.name == "s-respite")
            .expect("the cited synset projects a node");
        let cited: Vec<&str> = sourced
            .edges
            .iter()
            .filter(|(kind, _)| kind == DEFINITION_SOURCE_REL)
            .filter_map(|(_, t)| t.local_name())
            .collect();
        assert_eq!(cited, vec!["42 USC 300ii(7)", "42 CFR 441.302"]);
        assert_eq!(sourced.kind, SYNSET_KIND, "the node is still a synset");

        let unsourced = archive
            .nodes
            .iter()
            .find(|n| n.name == "s-kinship")
            .expect("the uncited synset projects a node");
        assert!(
            !unsourced
                .edges
                .iter()
                .any(|(kind, _)| kind == DEFINITION_SOURCE_REL),
            "a definition the lexicon gives no source for must gain none, even \
             though its prose carries em-dashes"
        );

        // Exactly the two distinct authorities were appended, as source atoms.
        let atoms: Vec<&str> = archive
            .nodes
            .iter()
            .filter(|n| n.kind == SOURCE_KIND)
            .map(|n| n.name.as_str())
            .collect();
        assert_eq!(atoms, vec!["42 CFR 441.302", "42 USC 300ii(7)"]);
        assert_eq!(archive.nodes.len(), before + 2);
    }

    /// THE RATCHET, both halves, over every REGISTERED definitional chat
    /// lexicon (registry-driven — a lexicon registered tomorrow is gated the
    /// day it is registered, with no edit here).
    ///
    /// 1. **Declared.** Every definition-bearing concept declares at least one
    ///    `dcterms:source`. A lexicon whose glosses are authored FROM documents
    ///    must say which; an entry added with no source fails here.
    /// 2. **Not re-inlined as provenance.** No definition may END with an
    ///    authority it declares, nor carry one immediately after a
    ///    [`RETIRED_CITATION_SEPARATORS`] separator. Those two positions ARE the
    ///    retired shape; occupying either puts the citation back where a reader
    ///    reads it as the answer's own authority.
    ///
    /// Together they make the old shape unreachable: re-inline the citation and
    /// (2) fires; drop it from `dc:source` to dodge (2) and (1) fires.
    ///
    /// The gate is exact in a way no dash-shaped rule can be, because it
    /// compares the text against THIS concept's own DECLARED authorities. It
    /// cannot fire on prose that merely uses an em-dash (`— thinking,
    /// remembering, and reasoning —`), cannot be fooled by a citation whose own
    /// punctuation contains a dash (Iowa's `former r. 441—77.37`), and needs no
    /// notion of what a citation "looks like".
    ///
    /// It is deliberately POSITIONAL rather than a bare `contains`: a definition
    /// may legitimately name a statute mid-sentence when the statute is its
    /// SUBJECT — `hc-medicaid-waiver` explains what a §1115 demonstration waiver
    /// is and must be free to say `(Social Security Act § 1115, 42 USC 1315)`.
    /// That is content. A trailing `— 42 USC 1315` is provenance wearing
    /// content's clothes, and only position tells them apart.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_registered_lexicon_reintroduces_an_inline_citation() {
        use crate::applied::data_provisioning::chat_lexicons::definitional_lexicon_runtime_ontology;
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::cognitive::linguistics::english::bridge::FORM_KIND;
        use pr4xis_runtime::lens::archive_lens::archived_local_name;

        let mut gated = 0usize;
        for entry in data_sources() {
            let Some(result) = definitional_lexicon_runtime_ontology(entry) else {
                continue;
            };
            let onto = result.unwrap_or_else(|e| panic!("{} materializes: {e}", entry.name));
            for node in onto.archive().nodes.iter() {
                // A Form atom is a surface and a BibliographicResource atom IS a
                // citation — neither is a definition-bearing concept.
                if node.kind == FORM_KIND || node.kind == SOURCE_KIND {
                    continue;
                }
                let Some(gloss) = node.lexical.as_ref() else {
                    continue;
                };
                let cited: Vec<&str> = node
                    .edges
                    .iter()
                    .filter(|e| e.0 == DEFINITION_SOURCE_REL)
                    .filter_map(|e| archived_local_name(&e.1))
                    .collect();
                assert!(
                    !cited.is_empty(),
                    "{}: `{}` glosses a term but declares no dc:source — an \
                     authored lexicon definition must name the document it was \
                     written from",
                    entry.name,
                    node.name.as_str(),
                );
                for citation in cited {
                    assert!(
                        !gloss.as_str().ends_with(citation),
                        "{}: `{}`'s definition ENDS with its own citation \
                         `{citation}`. A trailing citation reads as the \
                         answer's authority while nothing has read that \
                         document; it belongs in dc:source alone.",
                        entry.name,
                        node.name.as_str(),
                    );
                    for sep in RETIRED_CITATION_SEPARATORS {
                        for (at, _) in gloss.as_str().match_indices(sep) {
                            assert!(
                                !gloss.as_str()[at + sep.len()..].starts_with(citation),
                                "{}: `{}` re-introduces `{sep}{citation}` — the \
                                 exact inline-citation shape dc:source replaced.",
                                entry.name,
                                node.name.as_str(),
                            );
                        }
                    }
                }
                gated += 1;
            }
        }
        assert!(
            gated > 0,
            "the registry must carry at least one definitional chat lexicon for \
             this ratchet to mean anything"
        );
    }

    /// The provenance survives the functor and materialization, and the
    /// resulting ontology is referentially closed (a dangling `dcterms:source`
    /// target would make `materialize` fail, so reaching a `RuntimeOntology` at
    /// all IS the closure proof).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn provenance_survives_the_functor_into_a_materialized_ontology() {
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("t"))
            .expect("the fixture materializes");
        let node = onto
            .archive()
            .nodes
            .iter()
            .find(|n| n.name == "s-respite")
            .expect("the cited synset is present after the functor");
        let cited: Vec<&str> = node
            .edges
            .iter()
            .filter(|e| e.0 == DEFINITION_SOURCE_REL)
            .filter_map(|e| pr4xis_runtime::lens::archive_lens::archived_local_name(&e.1))
            .collect();
        assert_eq!(cited, vec!["42 USC 300ii(7)", "42 CFR 441.302"]);
        assert!(
            onto.archive()
                .nodes
                .iter()
                .any(|n| n.kind == SOURCE_KIND && n.name == "42 USC 300ii(7)"),
            "the cited authority is a declared BibliographicResource atom"
        );
    }
}
