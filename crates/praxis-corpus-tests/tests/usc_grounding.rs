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

use pr4xis::ontology::Axiom;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;
use pr4xis_domains::cognitive::linguistics::english::bridge::{
    ENGLISH_ONTOLOGY, FORM_KIND, project_archive_with_forms,
};
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_domains::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
use pr4xis_domains::formal::meta::grounding_laws::LoadedUscSectionGroundsToLawByComposition;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::judicial::statute_structure::grounding::{
    CITES_REL, DEFINES_REL, cites_lens, defines_lens, denotes_lens,
};
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::{
    COMPOSES_REL, citation_index, project_archive, usc_runtime_ontology,
};
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use pr4xis_runtime::definition::EdgeTarget;
use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology, ground};
use pr4xis_runtime::ontology::relations_kind;
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

/// Load a specific provisioned USC title by number (e.g. `15` →
/// `usc_title_15`), or `None` if that title isn't provisioned on this
/// checkout — routed through [`require`] by callers (tests do not skip).
fn provisioned_title(number: u32) -> Option<UsCode> {
    let root = workspace_root();
    let name = format!("usc_title_{number}");
    let entry = data_sources().iter().find(|e| e.name == name)?;
    let source = std::fs::read(root.join(entry.local_path())).ok()?;
    let text = core::str::from_utf8(&source).expect("USLM source is UTF-8");
    let title = read_uslm_title(text).expect("parse title");
    Some(UsCode::from_uslm_titles_owned(vec![title]))
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
                .expect("the denotes floor never fails closed")
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

/// THE CITES GATE: a real cross-title citation in a provisioned USC title
/// resolves — over the generic substrate — into the real cited provision in a
/// SEPARATE provisioned title's archive, via `cites_lens` + the generic
/// `AtomResolver`. Titles 15 and 5 cross-cite heavily in the real corpus (Title
/// 15 alone carries several hundred `href="/us/usc/t5/..."` citations), so this
/// searches title 15's real citations for the first one whose target is ALSO a
/// node title 5's own real projection declares — proving the whole pipeline
/// (citation_index → cites_lens → ground → AtomResolver) end to end on real
/// data, not a synthetic fixture.
#[test]
fn a_real_cross_title_citation_resolves_into_the_cited_titles_archive() {
    let citing = require(provisioned_title(15), "usc_title_15");
    let cited_title = require(provisioned_title(5), "usc_title_5");

    let citing_archive = project_archive(&citing);
    let cited_archive = project_archive(&cited_title);
    let cited_names: std::collections::BTreeSet<&str> = cited_archive
        .nodes
        .iter()
        .map(|n| n.name.as_str())
        .collect();

    let refs_by_urn = citation_index(&citing);
    // A real citation from title 15 into title 5 whose target is a node title
    // 5's own projection actually declares (some hrefs name a title/part/
    // chapter grouping node `project_archive` never projects — this finds a
    // real HIT among the real citations, not merely a real citation).
    let (citing_urn, target_href) = refs_by_urn
        .iter()
        .flat_map(|(urn, refs)| refs.iter().map(move |r| (urn.as_str(), r.href.as_str())))
        .find(|(_, href)| href.starts_with("/us/usc/t5/") && cited_names.contains(href))
        .expect("title 15 has at least one real citation into a title-5 node its own projection declares");

    let own_names: std::collections::BTreeSet<String> = citing_archive
        .nodes
        .iter()
        .map(|n| n.name.clone())
        .collect();
    let mut peers = std::collections::BTreeMap::new();
    let cited_root = cited_archive.root().unwrap();
    peers.insert("usc_title_5".to_string(), cited_archive);

    let grounded = ground(
        &citing_archive,
        cites_lens(&refs_by_urn, &own_names, &peers),
    )
    .expect("the cites lens grounds the real title");
    let citing_node = grounded
        .nodes
        .iter()
        .find(|n| n.name == citing_urn)
        .expect("the citing node survives grounding");
    let cites_edge = citing_node
        .edges
        .iter()
        .find(|(k, t)| {
            k == CITES_REL
                && matches!(t, EdgeTarget::Grounded { ontology, .. } if ontology == "usc_title_5")
        })
        .map(|(_, t)| t.clone())
        .expect("the real cross-title citation grounded into usc_title_5");

    let manifest = ConnectedOntologies(vec![ConnectedOntology {
        name: "usc_title_5".to_string(),
        root: cited_root,
        role: "cites".to_string(),
    }]);
    let resolver = AtomResolver::new(&manifest, &peers).expect("usc_title_5 pin agrees");
    let resolved = resolver
        .resolve(&cites_edge)
        .expect("the real minted Cites edge resolves by content address");
    assert_eq!(resolved.name, target_href);
    eprintln!(
        "USC CITES GATE: {citing_urn} cites {target_href} (real title 15 -> title 5 cross-reference, over the generic substrate)"
    );
}

/// THE DEFINES GATE: a real "the term 'X' means Y" declarative in a
/// provisioned USC title grounds — over the generic substrate — into the
/// definiendum's real English `ontolex:Form` atom, via `defines_lens` + the
/// generic `AtomResolver`. Targets the specific provision
/// `/us/usc/t15/s6603/h/6/A`, whose real prose ("The term "consumer" means a
/// natural person.") is byte-verified against
/// `usc_title_15-pl-119-90.xml` and is simple enough for the shipped grammar
/// (a single-word definiendum, close apposition, a determiner+adjective+noun
/// object — no coordination, no PP chain, no parenthetical adjunct) to fully
/// derive. Isolates that ONE real node rather than scanning the whole title
/// (title 15 has tens of thousands of provisions; most of their prose is far
/// outside this grammar's coverage and would each pay a full chart-parse
/// before failing) — the node itself is still the real, unmodified
/// projection of the real corpus, not a synthetic fixture.
#[test]
fn a_real_the_term_x_means_y_provision_grounds_a_defines_pointer() {
    let usc = require(provisioned_title(15), "usc_title_15");
    let english = english_loaded();
    let verbnet = verbnet_classes_loaded();

    let archive = project_archive(&usc);
    let provision = archive
        .nodes
        .iter()
        .find(|n| n.name == "/us/usc/t15/s6603/h/6/A")
        .expect("the real title-15 consumer-definition provision projects");
    let lexical = provision
        .lexical
        .as_deref()
        .expect("the provision carries prose");
    eprintln!("USC DEFINES GATE source prose: {lexical:?}");
    assert!(
        lexical.contains("consumer") && lexical.contains("means"),
        "sanity: the targeted node is really the consumer definition; got {lexical:?}"
    );

    let single_node_archive = pr4xis_runtime::archive::Archive {
        nodes: vec![provision.clone()],
        connections: vec![],
    };
    let mint_domain = OntologyName::new_static("usc_t15_coinages");
    let grounded = ground(
        &single_node_archive,
        defines_lens(
            english,
            english,
            verbnet,
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &mint_domain,
        ),
    )
    .expect("the defines lens grounds the real provision");
    let (kind, target) = grounded.nodes[0]
        .edges
        .iter()
        .find(|(k, _)| k == DEFINES_REL)
        .expect("the real provision's definiendum grounded");
    assert_eq!(kind, DEFINES_REL);

    let english_archive = project_archive_with_forms(english);
    let english_root = english_archive.root().unwrap();
    let mut peers = std::collections::BTreeMap::new();
    peers.insert(ENGLISH_ONTOLOGY.to_string(), english_archive);
    let manifest = ConnectedOntologies(vec![ConnectedOntology {
        name: ENGLISH_ONTOLOGY.to_string(),
        root: english_root,
        role: "defines".to_string(),
    }]);
    let resolver = AtomResolver::new(&manifest, &peers).expect("english pin agrees");
    let resolved = resolver
        .resolve(target)
        .expect("the real minted defines edge resolves by content address");
    assert_eq!(resolved.kind, FORM_KIND);
    assert_eq!(resolved.name, "consumer");
    eprintln!(
        "USC DEFINES GATE: /us/usc/t15/s6603/h/6/A defines the English Form {:?} (over the generic substrate)",
        resolved.name
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
/// TYPE grounding, but NOT `Precedent` ("case law") — crediting the grounding
/// functor + the loaded LegalSources closure, never a hardcode, for BOTH load
/// orders. This is the `#[test]` driver for the registered, cited
/// [`LoadedUscSectionGroundsToLawByComposition`] axiom (the raw differential now
/// lives behind its `verify()`). `require()`-gates on a provisioned USC title,
/// so an unprovisioned checkout hard-fails with the `pr4xis update usc` hint —
/// tests do not skip.
#[test]
fn a_loaded_usc_section_reaches_statute_and_law_by_composition() {
    require(first_provisioned_title(), "usc");
    assert!(LoadedUscSectionGroundsToLawByComposition.verify().is_ok());
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

    // Compose the USC ALONE (no LegalSources). The general grounding pass DEFERS —
    // with no LegalSources peer there is nothing to mint the type edge against, so
    // the section carries no cross-ontology typing (never a silent wrong bind).
    let mut set = vec![Rc::new(usc_onto)];
    pr4xis_domains::formal::meta::grounding::ground_loaded_set(&mut set, English::sample_static())
        .expect("USC-alone grounding defers cleanly (no LegalSources peer, no loud fault)");
    let composed = ComposedReasoner::new(English::sample_static(), set);

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
