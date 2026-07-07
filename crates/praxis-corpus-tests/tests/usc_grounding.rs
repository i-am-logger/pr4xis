//! USC grounding over the GENERIC substrate, on the real corpus — the heavy-lane
//! gate that ties the two bridges:
//!
//!   real USLM title → `UsCode`
//!     → `uslm::corpus::bridge::project_archive`  (statute provisions as Definition nodes)
//!     → `grounding::ground(denotes_lens(english_loaded()))`  (typed Grounded edges)
//!     → generic `AtomResolver`  (resolve a provision's denotes edge)
//!     → an English `ontolex:Form` atom.
//!
//! Nothing here is a bespoke string side-channel and nothing outside the lens is
//! English-specific. USC titles are externally provisioned (`pr4xis update`); CI
//! provisions them, so a plain checkout HARD-FAILS via `require` naming
//! `pr4xis update usc` — tests do not skip.

use std::rc::Rc;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;
use pr4xis_domains::cognitive::linguistics::english::bridge::{
    ENGLISH_ONTOLOGY, FORM_KIND, project_archive_with_forms,
};
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::judicial::legal_sources::ontology::LegalSourcesCategory;
use pr4xis_domains::social::judicial::statute_structure::grounding::denotes_lens;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::{
    COMPOSES_REL, project_archive, usc_runtime_ontology,
};
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use pr4xis_runtime::definition::EdgeTarget;
use pr4xis_runtime::emit::emit;
use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};
use pr4xis_runtime::ontology::{materialize, relations_kind, subsumption_kind};
use praxis_corpus_tests::{require, workspace_root};

/// Load the first provisioned USC title as a `UsCode`, or `None` on a fresh
/// checkout — callers route the `None` through [`require`] to HARD-FAIL (tests
/// do not skip).
fn first_provisioned_title() -> Option<UsCode> {
    let root = workspace_root();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Ok(source) = std::fs::read(root.join(entry.local_path())) else {
            continue;
        };
        let text = core::str::from_utf8(&source).expect("USLM source is UTF-8");
        let title = read_uslm_title(text).expect("parse title");
        return Some(UsCode::from_uslm_titles_owned(vec![title]));
    }
    None
}

/// THE GATE: a real statute provision's prose grounds — over the generic substrate
/// — into a real English `ontolex:Form` atom, resolved by the generic resolver.
#[test]
fn a_real_statute_provision_grounds_into_an_english_form_atom() {
    let usc = require(first_provisioned_title(), "usc");
    let english = english_loaded();

    // 1. Project the title into the generic Archive (provisions as Definition nodes).
    let archive = project_archive(&usc);
    assert!(
        archive.nodes.len() > 1,
        "the title projects sections + subdivisions as nodes"
    );

    // 2. Ground lazily with the lexical denotes lens — find the FIRST provision
    //    whose prose grounds a word English knows (typed Grounded edge).
    let lens = denotes_lens(english);
    let (provision_name, denotes_edge) = archive
        .nodes
        .iter()
        .find_map(|n| {
            lens(n)
                .into_iter()
                .next()
                .map(|(_, target)| (n.name.clone(), target))
        })
        .expect("some statute provision grounds a known English word");
    assert!(
        matches!(&denotes_edge, EdgeTarget::Grounded { .. }),
        "the grounding edge is a typed foreign-atom edge"
    );

    // 3. Resolve it through the GENERIC resolver over the real English archive.
    let english_archive = project_archive_with_forms(english);
    let english_root = english_archive.root().unwrap();
    let mut peers = std::collections::BTreeMap::new();
    peers.insert(ENGLISH_ONTOLOGY.to_string(), english_archive);
    let manifest = ConnectedOntologies(vec![ConnectedOntology {
        name: ENGLISH_ONTOLOGY.to_string(),
        root: english_root,
        role: "denotes".to_string(),
    }]);
    let resolver = AtomResolver::new(&manifest, &peers).expect("english pin agrees");

    let form = resolver
        .resolve(&denotes_edge)
        .expect("the statute provision's denotes edge resolves by content address");
    assert_eq!(
        form.kind, FORM_KIND,
        "a statute provision grounds into an ontolex:Form, never a sense"
    );
    eprintln!(
        "USC GROUNDING GATE: {provision_name} denotes the English Form {:?} (over the generic substrate)",
        form.name
    );
}

/// The Composes hierarchy projects as a real, referentially-closed Parthood
/// mereology over a nested title (the unit test only had the flat `UsCode::sample`).
#[test]
fn the_real_title_projects_a_parthood_mereology() {
    let usc = require(first_provisioned_title(), "usc");
    // (1) The RAW structural projection: the Composes mereology edges are
    // referentially closed (the §9 heading/citation lexicalization edges ride
    // alongside; the praxis relabel is the functor's job, below).
    let archive = project_archive(&usc);
    let declared: std::collections::BTreeSet<&str> =
        archive.nodes.iter().map(|n| n.name.as_str()).collect();

    let mut composes = 0usize;
    for n in &archive.nodes {
        for (kind, target) in &n.edges {
            if kind != COMPOSES_REL {
                continue; // a §9 heading/citation lexicalization edge, not mereology
            }
            let parent = target.local_name().expect("Composes is a local edge");
            assert!(
                declared.contains(parent),
                "{}--composes-->{parent} names an undeclared node",
                n.name
            );
            composes += 1;
        }
    }
    assert!(
        composes > 0,
        "a real title has a subdivision hierarchy → Composes edges"
    );

    // (2) The full functor-as-data pipeline relabels Composes→Parthood (DATA) and
    // materialize folds it into a TRANSITIVE mereology — a deeply-nested provision
    // is part-of its section transitively, not just its direct parent.
    let onto = usc_runtime_ontology(
        &usc,
        pr4xis::ontology::meta::OntologyName::new_static("us_code"),
    )
    .expect("the USC pipeline materializes");
    let parthood = relations_kind("Parthood");
    let has_transitive_mereology = onto.archive().nodes.iter().any(|n| {
        let node = onto.concept(n.name.to_string());
        onto.reachable_from(&node, parthood.clone()).len() > n.edges.len()
    });
    assert!(
        has_transitive_mereology,
        "the functor relabels Composes→Parthood and materialize folds it into a \
         transitive mereology (a nested provision is part-of its section transitively)"
    );
    eprintln!("USC PARTHOOD: {composes} raw Composes edges → a transitive Parthood mereology");
}

/// THE COMPLAINT'S GATE: a LOADED USC section reaches `legal_sources:Statute` and
/// transitively `legal_sources:LegalSource` ("law") THROUGH the cross-ontology
/// TYPE grounding — the same materialized closure "is a statute a law" reads —
/// resolved by the generic `AtomResolver`. The conceptual layer ("is a statute a
/// law") answered before; this ties a LOADED statute (a specific section) to it.
///
/// It CREDITS THE GROUNDING FUNCTOR, not a hardcode, three ways:
///   1. the section reaches Statute → LegalSource ONLY because the USC→LegalSources
///      functor minted the typing edge AND LegalSources is loaded to resolve it;
///   2. the SAME section does NOT reach `Precedent` ("case law") — a sibling under
///      LegalSource that Statute does not subsume — so the answer reads the real
///      LegalSources closure, never a blanket yes;
///   3. the base "is a statute a law" still answers (the conceptual layer intact).
#[test]
fn a_loaded_usc_section_reaches_statute_and_law_by_composition() {
    let usc = require(first_provisioned_title(), "usc");

    // The grounded USC (project → usc_functor → type-grounding → materialize) and
    // the always-loaded LegalSources base (the CLI/wasm path: default lexicalizing
    // emit, materialized under "LegalSources").
    let usc_onto = usc_runtime_ontology(&usc, OntologyName::new_static("usc"))
        .expect("the USC pipeline materializes");
    let legal = materialize(
        emit::<LegalSourcesCategory>(),
        OntologyName::new_static("LegalSources"),
    )
    .expect("LegalSources materializes");

    let composed = ComposedReasoner::new(
        English::sample_static(),
        vec![Rc::new(legal), Rc::new(usc_onto)],
    );
    let subsumption = subsumption_kind();

    // The conceptual layer still answers: statute ⊑ … ⊑ law inside LegalSources.
    let statute = composed.lookup("statute").to_vec();
    let law = composed.lookup("law").to_vec();
    assert!(
        !statute.is_empty() && !law.is_empty(),
        "the LegalSources surfaces 'statute' and 'law' resolve"
    );
    assert!(
        statute
            .iter()
            .any(|&s| law.iter().any(|&l| composed.reaches(s, l, &subsumption))),
        "the base taxonomy holds: a statute is a law"
    );

    // A LOADED section — addressed by its URN surface (no hardcoded section number;
    // the first provisioned section of the first title).
    let section_urn = usc.all_sections()[0].urn.value().to_lowercase();
    let section = composed.lookup(&section_urn).to_vec();
    assert!(
        !section.is_empty(),
        "the loaded section {section_urn} resolves by its URN surface"
    );

    // THE FIX: the loaded section reaches legal_sources:Statute (its typing) …
    let reaches_statute = section.iter().any(|&sec| {
        statute
            .iter()
            .any(|&st| composed.reaches(sec, st, &subsumption))
    });
    assert!(
        reaches_statute,
        "a LOADED USC section reaches legal_sources:Statute through the type-grounding functor \
         (this is the complaint's fix — the instance→type link)"
    );

    // … and transitively legal_sources:LegalSource ("law"), the cross-ontology fold
    // (section --instantiates--> Statute ⊑ LegalDocument ⊑ LegalSource).
    let reaches_law = section
        .iter()
        .any(|&sec| law.iter().any(|&l| composed.reaches(sec, l, &subsumption)));
    assert!(
        reaches_law,
        "a LOADED USC section reaches legal_sources:LegalSource ('law') by composition"
    );

    // NOT a blanket yes: the section does NOT reach `Precedent` ("case law"), a
    // sibling Statute does not subsume — the cross-ontology reaches reads the REAL
    // LegalSources closure, crediting the functor + closure, not a hardcode.
    let case_law = composed.lookup("case law").to_vec();
    assert!(
        !case_law.is_empty(),
        "the LegalSources surface 'case law' (Precedent) resolves"
    );
    let reaches_precedent = section.iter().any(|&sec| {
        case_law
            .iter()
            .any(|&p| composed.reaches(sec, p, &subsumption))
    });
    assert!(
        !reaches_precedent,
        "a section is NOT case law — the type grounding is discriminating (reads the closure), \
         not a blanket cross-ontology yes"
    );

    eprintln!(
        "USC TYPE-GROUNDING GATE: loaded section {section_urn} reaches legal_sources:Statute → \
         LegalSource ('law'), but NOT Precedent ('case law') — cross-ontology composition."
    );
}

/// The RESOLVE-SIDE contrast: the SAME grounded USC, composed WITHOUT the
/// LegalSources peer, does NOT resolve the section's type — the cross-ontology
/// edge is fail-closed (no peer to resolve the foreign atom). This isolates the
/// resolution to the LOADED target, proving it is composition (both the functor's
/// edge AND the loaded target), never a property baked into the USC ontology.
#[test]
fn without_the_legal_sources_peer_the_type_link_is_fail_closed() {
    let usc = require(first_provisioned_title(), "usc");
    let usc_onto = usc_runtime_ontology(&usc, OntologyName::new_static("usc"))
        .expect("the USC pipeline materializes");

    // Compose the grounded USC ALONE (no LegalSources). The section still carries
    // its `Grounded` typing edge, but there is no peer to resolve it against.
    let composed = ComposedReasoner::new(English::sample_static(), vec![Rc::new(usc_onto)]);

    // With LegalSources absent, 'statute'/'law' do not resolve to any concept —
    // there is nothing for the section to be tied to, so the question abstains
    // rather than answering by a hardcoded property of the USC nodes.
    assert!(
        composed.lookup("statute").is_empty() && composed.lookup("law").is_empty(),
        "without the LegalSources peer, the LegalSources surfaces do not resolve — the \
         type link exists only when the target ontology is ALSO loaded (composition)"
    );

    // And the raw structural projection carries no cross-ontology edge at all — the
    // functor is what mints it (the produce-side with/without contrast).
    let raw = project_archive(&usc);
    let raw_grounded = raw.nodes.iter().any(|n| {
        n.edges
            .iter()
            .any(|(_, t)| !matches!(t, EdgeTarget::Local(_)))
    });
    assert!(
        !raw_grounded,
        "the RAW USC projection has no type grounding — the usc_legal_sources_functor mints it"
    );
}
