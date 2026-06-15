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
//! English-specific. USC titles are externally provisioned (`pr4xis update`), so a
//! plain checkout skips gracefully.

use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::english::bridge::{
    ENGLISH_ONTOLOGY, FORM_KIND, project_archive_with_forms,
};
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::judicial::statute_structure::grounding::denotes_lens;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::{
    COMPOSES_REL, project_archive, usc_runtime_ontology,
};
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use pr4xis_runtime::definition::EdgeTarget;
use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};
use pr4xis_runtime::ontology::relations_kind;
use praxis_corpus_tests::workspace_root;

/// Load the first provisioned USC title as a `UsCode`, or `None` on a fresh
/// checkout (skip gracefully).
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
    let Some(usc) = first_provisioned_title() else {
        eprintln!("SKIP: no USC title provisioned");
        return;
    };
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
    let Some(usc) = first_provisioned_title() else {
        eprintln!("SKIP: no USC title provisioned");
        return;
    };
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
        let node = onto.concept(n.name.clone());
        onto.reachable_from(&node, parthood.clone()).len() > n.edges.len()
    });
    assert!(
        has_transitive_mereology,
        "the functor relabels Composes→Parthood and materialize folds it into a \
         transitive mereology (a nested provision is part-of its section transitively)"
    );
    eprintln!("USC PARTHOOD: {composes} raw Composes edges → a transitive Parthood mereology");
}
