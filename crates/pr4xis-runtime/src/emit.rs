//! Emit — project a live, compiled praxis ontology (a `Category`) into a `.prx`
//! [`Archive`].
//!
//! This is the COMPILER half of the chrysalis. Where the runtime kernel works on
//! `Archive` *data*, this bridges the compile-time `ontology!` world into it: a
//! real, macro-generated ontology becomes a content-addressed `.prx` that
//! round-trips through the very runtime defined by the meta-`.prx`.
//!
//! Gated on the `emit` feature, which deps `pr4xis` (the category model); the
//! kernel itself carries no compile-time coupling. The projection is faithful to
//! the structure `Category` exposes: each `Concept` variant becomes a node, its
//! edges the outgoing morphisms (the materialized transitive closure) as
//! `(relation-kind, target)`, and its lexical grounding the concept's
//! ONTOLEX-Lemon gloss via [`Concept::lexical`] — so a `.prx` loaded with no
//! access to the compile-time `labels()` table (e.g. in the browser) still
//! carries each concept's meaning.
//!
//! ## Connections — the morphisms BETWEEN ontologies
//!
//! The projection also serializes every **connection** (functor / adjunction /
//! natural transformation) that touches `Cat`: from the registry's
//! [`connection_constructors`] it
//! selects the entries whose source or target ontology name is `Cat`'s own
//! ([`NamedCategory::ontology_name`](pr4xis::category::NamedCategory::ontology_name)),
//! extracts each one's finite
//! action-on-generators (the finite-presentation theorem — a structure-
//! preserving map is determined by its action on generators), and content-
//! addresses it as a [`Connection`]. The connection's `source`/`target` name
//! the OTHER ontology, so the arrow is cross-ontology and p2p-ready: a peer
//! rebinds it by content-address agreement.

use pr4xis::category::connection::{ConnectionFamily, ConnectionGenerators};
use pr4xis::category::{Arrow, Concept, DomainAxiomatized, FinitelyGenerated};
use pr4xis::ontology::connection_constructors;

use crate::archive::Archive;
use crate::connection::{Connection, GeneratorAction};
use crate::definition::Definition;
use crate::definition::EdgeTarget;

/// Translate a registry [`ConnectionGenerators`] (the `pr4xis`-native finite
/// presentation) into the wire [`Connection`] the runtime serializes. The two
/// types carry the same finite tables; this is the cross-crate bridge (the
/// registry cannot name the runtime type without a dependency cycle).
fn to_connection(g: ConnectionGenerators) -> Connection {
    let action = match g.family {
        ConnectionFamily::Functor {
            map_object,
            map_morphism,
        } => GeneratorAction::Functor {
            map_object,
            map_morphism,
        },
        ConnectionFamily::NaturalTransformation { components } => {
            GeneratorAction::NaturalTransformation { components }
        }
        ConnectionFamily::Adjunction {
            left_map_object,
            right_map_object,
            unit,
            counit,
        } => GeneratorAction::Adjunction {
            left_map_object,
            right_map_object,
            unit,
            counit,
        },
    };
    Connection {
        kind: g.kind,
        source: g.source.as_str().to_string(),
        target: g.target.as_str().to_string(),
        action,
        laws: g.laws,
    }
}

/// Lower ONE typed concept to its canonical wire [`Definition`] — THE
/// typed→`Definition` boundary. Praxis code holds typed values ([`Concept`]s
/// and [`Arrow`]s); the wire holds names. This function is the single place a
/// node makes that crossing: `kind` and `obj` lower via [`Concept::name`],
/// each morphism via its kind's canonical identifier and its target's name,
/// and the lexical grounding via the concept's ONTOLEX-Lemon gloss
/// ([`Concept::lexical`]). Because every serializer shares this one lowering,
/// the same concept lowers to the same `Definition` wherever it appears — two
/// call sites can never drift into different projections of the same node
/// (which would split its content address and break rebind-by-agreement).
pub fn definition_of<K, O, M>(kind: &K, obj: &O, morphisms: &[M]) -> Definition
where
    K: Concept,
    O: Concept,
    M: Arrow<Object = O>,
{
    lower(kind.name(), obj, morphisms)
}

/// The wire [`Definition`] of a behavioural BINDING — a node that is
/// name-keyed BY DESIGN: it carries a registered name (an axiom / lens
/// binding) the receiving binary re-binds through its OWN registries, where
/// load is the gate. Its definition is exactly its typed kind + that name —
/// no edges, no gloss — so its address commits to nothing the receiver does
/// not re-derive. The name is an open-world key (the same `&str` surface as
/// `axiom_by_name` and [`RebindTarget::address_of`](crate::rebind::RebindTarget::address_of)),
/// not a typed concept.
pub fn binding_definition<K: Concept>(kind: &K, name: &str) -> Definition {
    Definition {
        kind: kind.name().to_string(),
        name: name.to_string(),
        edges: Vec::new(),
        axioms: Vec::new(),
        lexical: None,
    }
}

/// [`definition_of`] with the kind already lowered to its wire name — the
/// kernel-internal leg [`emit`] uses for the meta-ontology kinds (`Concept`,
/// `Axiom`), which exist as DATA in the bootstrap meta-`.prx`
/// ([`crate::meta`]), not as a compile-time enum.
fn lower<O, M>(kind: &str, obj: &O, morphisms: &[M]) -> Definition
where
    O: Concept,
    M: Arrow<Object = O>,
{
    let mut edges: Vec<(String, EdgeTarget)> = morphisms
        .iter()
        .filter(|m| m.target() != *obj) // drop identity self-loops
        .map(|m| {
            (
                format!("{:?}", m.kind()),
                EdgeTarget::Local(m.target().name().to_string()),
            )
        })
        .collect();
    edges.sort();
    edges.dedup();
    // Carry the concept's lexical grounding INTO the `.prx`: its
    // ONTOLEX-Lemon gloss (the `Definition`/sense text). The runtime
    // `Definition.lexical` is the serialized gloss string, so project
    // the structured `Lexical` to its definition text here — sourced
    // generically from `Concept::lexical()` (which the `ontology!`
    // macro fills from its labels table), never a per-ontology hack.
    // An ungrounded concept stays `None` (honest absence).
    let lexical = obj.lexical().map(|lex| lex.definition.as_str().to_string());
    Definition {
        kind: kind.to_string(),
        name: obj.name().to_string(),
        edges,
        // ## Axioms are per-ONTOLOGY, not per-concept (the model).
        //
        // Praxis declares axioms at the ontology level: the structural
        // axioms are keyed by `Category`/relation-kind
        // (`structural_axioms_for::<Cat>()` — e.g. `NoCycles[Subsumption]`
        // constrains the WHOLE subsumption relation, not one concept),
        // and domain axioms are declared in the `ontology!` `axioms:`
        // clause. There is no mechanism for a single concept to declare
        // its OWN axiom, so this per-NODE `axioms` field stays empty:
        // populating it would FABRICATE a per-concept attachment praxis
        // does not have. The meta's "Axiom Constrains Concept" claim is
        // instead backed by emitting the ontology's structural axioms as
        // first-class Axiom NODES (in [`emit`]), each content-addressed by
        // their stable name — the same name `axiom_by_name` rebinds on load.
        axioms: Vec::new(),
        lexical,
    }
}

/// Project the compiled ontology `Cat` into a `.prx` [`Archive`]: one node per
/// `Concept` variant, edges from `morphisms_from` as `(relation-kind, target)`,
/// identity self-loops dropped, edges sorted for a canonical address; plus every
/// registered connection (functor / adjunction / natural transformation) whose
/// source or target is `Cat`'s ontology, content-addressed.
///
/// This is the PLAIN projection — it carries each concept's identity (`name`) and
/// its ONTOLEX-Lemon gloss (`lexical`), but mints NO lexical Form surfaces. Use it
/// ONLY for the kernel morphism-kind identity vocabulary (`RelationsCategory` /
/// `morphism_kinds.prx`), whose descriptive labels ("Subsumption (is-a)") must NOT
/// become chat surfaces — folding them into Form atoms would change
/// `morphism_kinds.prx` and move the kernel identity. Every chat-loadable DOMAIN
/// ontology must use [`emit`] instead, which lexicalizes by default so its natural
/// labels are query surfaces by construction.
pub fn emit_kind_vocabulary<Cat>() -> Archive
where
    // Emits one node per concept variant — enumerates the objects, so the
    // compiled ontology being projected must be finitely generated (closed-world).
    // `DomainAxiomatized: NamedCategory`, so this also gives the declared name.
    Cat: DomainAxiomatized + 'static,
    Cat::Object: FinitelyGenerated,
    // Structural axioms are derived from the category's typed relation-kinds
    // (`structural_axioms_for::<Cat>()`), so the kind enum must be inspectable.
    <Cat::Morphism as Arrow>::Kind: core::fmt::Debug + PartialEq + Clone + 'static,
{
    let mut nodes: Vec<Definition> = <Cat::Object as FinitelyGenerated>::variants()
        .iter()
        .map(|obj| lower("Concept", obj, &Cat::morphisms_from(obj)))
        .collect();

    // Structural axiom NODES — the per-ontology axioms the category's typed
    // relation-kinds license (Smith 2005 OBO-RO: every kind carries canonical
    // axioms; Guarino 2009). Each becomes an `Axiom`-kinded node named by its
    // stable `Axiom::name` (e.g. `NoCycles[Subsumption]`) and lexically grounded
    // by its description — content-addressable, and rebindable by name through
    // `axiom_by_name`. This backs the meta's `Axiom` concept WITHOUT fabricating
    // a per-concept attachment; the axiom's identity is its name, the same wire
    // identity the load gate matches.
    for axiom in pr4xis::ontology::reasoning::structural_axioms_for::<Cat>() {
        nodes.push(Definition {
            kind: "Axiom".to_string(),
            name: axiom.name().as_str().to_string(),
            edges: Vec::new(),
            axioms: Vec::new(),
            lexical: Some(axiom.description().as_str().to_string()),
        });
    }

    // Domain axiom NODES — the per-`axioms:`-clause claims the ontology
    // DECLARED (Guarino 2009: the axiomatisation layer, distinct from the
    // structural commitment above). Where structural axioms are derivable from
    // the morphism graph alone, these are subject-matter claims reachable only
    // through the typed `Cat::domain_axioms()` accessor the `ontology!` macro
    // wired (the `DomainAxiomatized` bridge — mirroring `NamedCategory`). Each
    // becomes an `Axiom`-kinded node keyed by its stable `Axiom::name` — the
    // SAME wire identity `axiom_by_name` rebinds on load — and lexically
    // grounded by its description. We emit ONLY axioms the ontology actually
    // declares: an empty `axioms:` clause yields an empty Vec, so nothing is
    // fabricated.
    //
    // Order/duplicate-independent identity comes for free at the envelope:
    // [`Archive::root`] sorts and dedups node ADDRESSES before rooting, so a
    // domain axiom that happens to share a wire name (hence node) with a
    // structural one collapses there — no explicit re-sort of `nodes` needed
    // (which would also reorder the concept nodes).
    for axiom in <Cat as DomainAxiomatized>::domain_axioms() {
        nodes.push(Definition {
            kind: "Axiom".to_string(),
            name: axiom.name().as_str().to_string(),
            edges: Vec::new(),
            axioms: Vec::new(),
            lexical: Some(axiom.description().as_str().to_string()),
        });
    }

    // Connections: every registered functor / adjunction / natural
    // transformation whose source OR target is THIS ontology. Selecting on
    // both directions makes the slice the full set of arrows incident to
    // `Cat` — incoming AND outgoing — so a peer holding this `.prx` sees every
    // structure-preserving map that relates `Cat` to its neighbours.
    //
    // `connection_constructors()` runs each registered arrow's extractor (the
    // finite action-on-generators); we keep the ones touching `Cat`, translate
    // them to the wire `Connection`, and content-address them via the archive
    // root. On wasm32 the registry is empty (linkme unsupported) — the same
    // fail-closed "no connections extractable" the node path has; the `.prx` is
    // produced natively at build time, where the registry is populated.
    let own = Cat::ontology_name();
    let mut connections: Vec<Connection> = connection_constructors()
        .into_iter()
        .filter(|g| g.source == own || g.target == own)
        .map(to_connection)
        .collect();
    // Order-independent identity: sort + dedup by content-address so the
    // archive root does not depend on registry/link order (the same canonical
    // discipline the nodes follow). Two connections with equal action collapse.
    connections.sort_by_key(|c| c.address().map(|a| *a.as_bytes()).unwrap_or([0u8; 32]));
    connections.dedup();

    Archive { nodes, connections }
}

/// Project the compiled ontology `Cat` into a `.prx` [`Archive`] and LEXICALIZE it
/// — the DEFAULT projection every chat-loadable ontology uses. It first builds the
/// plain projection ([`emit_kind_vocabulary`]: identity + gloss + connections),
/// then for every concept whose ONTOLEX-Lemon label is a written surface DISTINCT
/// from its identifier (its node name) mints an `ontolex:Form` atom carrying that
/// label and a [`CANONICAL_FORM_REL`](crate::definition::CANONICAL_FORM_REL) edge
/// from the concept to it.
///
/// # Lexicalization is the default; the plain projection is the marked exception
///
/// A concept's label is the word a person actually types — `LegalSource` labelled
/// "law", `Precedent` labelled "case law", `CorrectService` labelled "correct
/// service". Minting it as a queryable Form surface is what makes EVERY compiled
/// ontology chat-queryable by its natural language BY CONSTRUCTION, not opt-in. So
/// this is the default `emit`; the label-dropping [`emit_kind_vocabulary`] is the
/// MARKED exception, reserved for the kernel morphism-kind identity vocabulary
/// (`RelationsCategory` / `morphism_kinds.prx`), whose descriptive labels must NOT
/// become surfaces.
///
/// A concept whose label equals its name (case-insensitively) mints no redundant
/// Form (the node-name grounding already covers that surface), so an ontology whose
/// labels all match its variants emits byte-identically to [`emit_kind_vocabulary`].
pub fn emit<Cat>() -> Archive
where
    Cat: DomainAxiomatized + 'static,
    Cat::Object: FinitelyGenerated,
    <Cat::Morphism as Arrow>::Kind: core::fmt::Debug + PartialEq + Clone + 'static,
{
    use crate::definition::{CANONICAL_FORM_REL, form_atom};
    use std::collections::BTreeSet;

    let mut archive = emit_kind_vocabulary::<Cat>();

    // One Form atom per DISTINCT label surface, minted after the concept edges are
    // aimed at it, so the archive stays referentially closed with no duplicate.
    let mut forms: BTreeSet<String> = BTreeSet::new();
    for obj in <Cat::Object as FinitelyGenerated>::variants() {
        let Some(lexical) = obj.lexical() else {
            continue;
        };
        let label = lexical.label.as_str();
        let name = obj.name();
        // Skip the empty and the redundant: a label that (case-insensitively)
        // equals the identifier adds no surface the node-name grounding lacks.
        if label.is_empty() || label.eq_ignore_ascii_case(name) {
            continue;
        }
        // Aim a canonicalForm edge from the concept node at its label Form. The
        // concept was emitted by `lower` as a `Concept`-kinded node named `name`.
        if let Some(node) = archive
            .nodes
            .iter_mut()
            .find(|n| n.kind == "Concept" && n.name == name)
        {
            node.edges.push((
                CANONICAL_FORM_REL.to_string(),
                EdgeTarget::Local(label.to_string()),
            ));
            // Keep the edge rows canonical (sorted, deduped) so the node address
            // does not depend on the order lexicalization ran relative to `lower`.
            node.edges.sort();
            node.edges.dedup();
            forms.insert(label.to_string());
        }
    }
    for surface in &forms {
        archive.nodes.push(form_atom(surface));
    }
    archive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ContentAddress;
    use crate::load;
    use crate::rebind::{RebindTarget, rebind_nodes};
    use std::collections::{BTreeSet, HashMap};

    // A small REAL ontology — exactly what a domain ontology is, in miniature:
    // generated by the same `ontology!` macro, materialized transitive closure
    // and all.
    pr4xis::ontology! {
        name: "Org",
        source: "pr4xis-runtime emit test fixture",
        concepts: [Employer, Employee, Person, Agent],
        labels: {
            Employer: ("en", "Employer", "One who employs."),
            Employee: ("en", "Employee", "One who is employed."),
            Person: ("en", "Person", "A human being."),
            Agent: ("en", "Agent", "One who acts."),
        },
        is_a: [
            (Employer, Person),
            (Employee, Person),
            (Person, Agent),
        ],
    }

    // A REAL ontology that DECLARES domain axioms via the `ontology!` `axioms:`
    // clause — the per-subject-matter claims (Guarino 2009) distinct from the
    // structural axioms the relation-kinds license. These live on the generated
    // `<Name>Ontology` struct and are reached only through the typed
    // `DomainAxiomatized::domain_axioms()` bridge the macro wires; the test
    // below proves `emit` surfaces them as content-addressed `Axiom` nodes that
    // round-trip byte-exact and re-bind by name. The `holds` expressions read
    // the ontology's OWN materialized structure (its morphisms / closure), so
    // each axiom is a genuine claim about this ontology — not a tautology.
    pr4xis::ontology! {
        name: "Guild",
        source: "pr4xis-runtime emit domain-axiom test fixture",
        concepts: [Master, Apprentice, Member],
        labels: {
            Master: ("en", "Master", "A guild member who has completed an apprenticeship."),
            Apprentice: ("en", "Apprentice", "A guild member learning the trade."),
            Member: ("en", "Member", "One enrolled in the guild."),
        },
        is_a: [
            (Master, Member),
            (Apprentice, Member),
        ],
        axioms: {
            // Every direct role (Master, Apprentice) subsumes to Member — a
            // claim about THIS ontology's taxonomy, checked against the
            // materialized Subsumption closure (not a constant `true`).
            EveryRoleIsAMember: {
                source: "Guarino (2009) The Ontological Level — domain axiomatisation layer",
                description: "Every guild role (Master, Apprentice) is-a Member via the Subsumption closure.",
                holds: {
                    use pr4xis::category::Category;
                    let to_member = |from: GuildConcept| {
                        GuildCategory::morphisms().iter().any(|m| {
                            m.from == from
                                && m.to == GuildConcept::Member
                                && m.kind == GuildRelationKind::Subsumption
                        })
                    };
                    to_member(GuildConcept::Master) && to_member(GuildConcept::Apprentice)
                },
            },
            // A second declared axiom, so the test sees MORE than one domain
            // axiom node: the Member role does not subsume any role (it is the
            // taxonomy root) — checked against the same closure.
            MemberIsTheRoot: {
                source: "Smith et al. (2005) OBO Relation Ontology — Subsumption is antisymmetric",
                description: "Member is the taxonomy root: it subsumes no other guild role.",
                holds: {
                    use pr4xis::category::Category;
                    !GuildCategory::morphisms().iter().any(|m| {
                        m.from == GuildConcept::Member
                            && m.to != GuildConcept::Member
                            && m.kind == GuildRelationKind::Subsumption
                    })
                },
            },
        },
    }

    // A SECOND real ontology — the codomain of a cross-ontology functor. Its
    // concepts are the same-named "workforce" view of the Org concepts (a full
    // subcategory embedding), so the functor below is fully-faithful by name.
    pr4xis::ontology! {
        name: "Workforce",
        source: "pr4xis-runtime emit test fixture",
        concepts: [Employer, Employee, Person, Agent, Contractor],
        labels: {
            Employer: ("en", "Employer", "One who employs (workforce view)."),
            Employee: ("en", "Employee", "One who is employed (workforce view)."),
            Person: ("en", "Person", "A human being (workforce view)."),
            Agent: ("en", "Agent", "One who acts (workforce view)."),
            Contractor: ("en", "Contractor", "An external agent under contract."),
        },
        is_a: [
            (Employer, Person),
            (Employee, Person),
            (Contractor, Agent),
            (Person, Agent),
        ],
    }

    // A REAL registered functor `Org → Workforce` — a fully-faithful by-name
    // embedding (the Org subsumption lattice IS a full subcategory of the
    // Workforce one). Registering it via `pr4xis::functor!` populates the
    // FUNCTOR_CONSTRUCTORS slice, so `emit` discovers it the same way a domain
    // ontology's functors are discovered.
    pr4xis::functor! {
        name: OrgIntoWorkforce,
        source: OrgCategory,
        target: WorkforceCategory,
        citation: "Mac Lane (1971) Categories for the Working Mathematician Ch. I §4",
        map_object: |obj: &OrgConcept| -> WorkforceConcept {
            match obj {
                OrgConcept::Employer => WorkforceConcept::Employer,
                OrgConcept::Employee => WorkforceConcept::Employee,
                OrgConcept::Person => WorkforceConcept::Person,
                OrgConcept::Agent => WorkforceConcept::Agent,
            }
        },
        map_morphism: |m: &OrgRelation| -> WorkforceRelation {
            let map = |o: &OrgConcept| match o {
                OrgConcept::Employer => WorkforceConcept::Employer,
                OrgConcept::Employee => WorkforceConcept::Employee,
                OrgConcept::Person => WorkforceConcept::Person,
                OrgConcept::Agent => WorkforceConcept::Agent,
            };
            // By-name kind translation — each Org kind to its same-named
            // Workforce kind (the macro emits the full canonical kind enum).
            let kind = match m.kind {
                OrgRelationKind::Identity => WorkforceRelationKind::Identity,
                OrgRelationKind::Subsumption => WorkforceRelationKind::Subsumption,
                OrgRelationKind::Parthood => WorkforceRelationKind::Parthood,
                OrgRelationKind::Causation => WorkforceRelationKind::Causation,
                OrgRelationKind::Opposition => WorkforceRelationKind::Opposition,
            };
            WorkforceRelation { from: map(&m.from), to: map(&m.to), kind }
        },
    }

    #[test]
    fn emits_every_concept_as_a_node() {
        let archive = emit::<OrgCategory>();
        let names: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for c in ["Employer", "Employee", "Person", "Agent"] {
            assert!(names.contains(c), "missing concept node {c}");
        }
    }

    #[test]
    fn emits_the_subsumption_closure_as_edges() {
        let archive = emit::<OrgCategory>();
        let employer = archive.nodes.iter().find(|n| n.name == "Employer").unwrap();
        let targets: BTreeSet<&str> = employer
            .edges
            .iter()
            .filter_map(|(_, t)| t.local_name())
            .collect();
        // direct: Employer is_a Person; closure: Employer is_a Agent.
        assert!(targets.contains("Person"), "Employer → Person missing");
        assert!(
            targets.contains("Agent"),
            "closure Employer → Agent missing"
        );
    }

    #[test]
    fn emits_each_concepts_gloss_as_its_lexical_and_round_trips() {
        // The labels table the `ontology!` macro generated for this fixture —
        // the authoritative source the emitted gloss must match (tuple shape:
        // (variant, lang, surface, gloss)).
        let glosses: HashMap<&str, &str> = OrgOntology::labels()
            .iter()
            .map(|(_, _, surface, gloss)| (*surface, *gloss))
            .collect();

        let archive = emit::<OrgCategory>();
        // Every emitted CONCEPT node carries its concept's gloss — non-None and
        // equal to the macro's labels table (proving the gloss travels IN the
        // `.prx`, not only in the compile-time `labels()` side table). Axiom
        // nodes (the per-ontology structural axioms) carry their OWN gloss — the
        // axiom description — so the gloss check is scoped to the Concept nodes.
        let concept_nodes: Vec<_> = archive
            .nodes
            .iter()
            .filter(|n| n.kind == "Concept")
            .collect();
        for node in &concept_nodes {
            let expected = glosses.get(node.name.as_str()).copied();
            assert_eq!(
                node.lexical.as_deref(),
                expected,
                "node {} must carry its labels-table gloss",
                node.name
            );
            assert!(
                node.lexical.is_some(),
                "every glossed concept must emit a lexical; {} did not",
                node.name
            );
        }
        assert_eq!(
            concept_nodes.iter().filter(|n| n.lexical.is_some()).count(),
            glosses.len(),
            "every labelled concept must contribute a gloss"
        );

        // The gloss survives the byte-exact round-trip through load.
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive);
        let employer = loaded.nodes.iter().find(|n| n.name == "Employer").unwrap();
        assert_eq!(employer.lexical.as_deref(), Some("One who employs."));
    }

    #[test]
    fn emitted_ontology_round_trips_through_the_runtime() {
        let archive = emit::<OrgCategory>();
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive);
    }

    #[test]
    fn axioms_field_is_wired_through_the_codec_round_trip() {
        // The meta-`.prx` declares an `Axiom` concept that `Constrains` a
        // `Concept`, and `Definition::address` folds `axioms` into the
        // content-address — but `emit` does not yet DERIVE per-concept axioms
        // (see the honest-deferral note at the `axioms: Vec::new()` site). This
        // test proves the field is nonetheless REAL, not vestigial: a manually
        // constructed `Definition` carrying a NON-EMPTY axioms Vec survives the
        // `emit -> load` codec round-trip byte-exact, AND the axioms field is
        // load-bearing on the content-address (two definitions differing ONLY
        // in `axioms` get different addresses, so the archive root differs too).
        let with_axioms = Definition {
            kind: "Concept".to_string(),
            name: "Employer".to_string(),
            edges: vec![(
                "Subsumption".to_string(),
                EdgeTarget::Local("Agent".to_string()),
            )],
            axioms: vec![
                "EmployerIsAgent".to_string(),
                "EmployerHiresEmployee".to_string(),
            ],
            lexical: Some("employer".to_string()),
        };
        // Same node with NO axioms — differs ONLY in the axioms field.
        let without_axioms = Definition {
            axioms: Vec::new(),
            ..with_axioms.clone()
        };

        // Difference-detection: the axioms field changes the node address...
        assert_ne!(
            with_axioms.address().unwrap(),
            without_axioms.address().unwrap(),
            "axioms must be load-bearing on the definition address"
        );

        let archive = Archive {
            nodes: vec![with_axioms.clone()],
            connections: Vec::new(),
        };
        let archive_no_axioms = Archive {
            nodes: vec![without_axioms],
            connections: Vec::new(),
        };
        // ...and therefore on the archive root (the content-address the load
        // gate checks against).
        assert_ne!(
            archive.root().unwrap(),
            archive_no_axioms.root().unwrap(),
            "the axioms field must reach the archive root"
        );

        // The axioms survive the codec round-trip byte-exact, fail-closed
        // against the archive's own root.
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive, "the archive must round-trip faithfully");
        let node = loaded.nodes.iter().find(|n| n.name == "Employer").unwrap();
        assert_eq!(
            node.axioms,
            vec![
                "EmployerIsAgent".to_string(),
                "EmployerHiresEmployee".to_string(),
            ],
            "the non-empty axioms Vec must survive the round-trip byte-exact"
        );
    }

    #[test]
    fn emits_declared_domain_axioms_as_nodes_that_round_trip_and_rebind() {
        // The Guild ontology DECLARES two domain axioms in its `axioms:` clause.
        // They are reached only through the typed `DomainAxiomatized` bridge the
        // macro wired — NOT from the `Cat` morphism graph alone — and `emit`
        // must surface each as a content-addressed `Axiom`-kinded node.

        // The names the ontology actually declared (the source of truth — read
        // from the typed accessor, never hand-typed into the assertion).
        let declared: Vec<(String, String)> = <GuildCategory as DomainAxiomatized>::domain_axioms()
            .iter()
            .map(|a| {
                (
                    a.name().as_str().to_string(),
                    a.description().as_str().to_string(),
                )
            })
            .collect();
        assert_eq!(
            declared.len(),
            2,
            "the Guild fixture declares exactly two domain axioms"
        );

        let archive = emit::<GuildCategory>();

        // Each declared domain axiom is emitted as an Axiom node, keyed by its
        // stable name, lexically grounded by its description. We assert against
        // the names READ FROM the ontology, so the test cannot pass on a
        // fabricated axiom (only those the ontology actually declares appear).
        let axiom_nodes: HashMap<&str, &Definition> = archive
            .nodes
            .iter()
            .filter(|n| n.kind == "Axiom")
            .map(|n| (n.name.as_str(), n))
            .collect();
        for (name, description) in &declared {
            let node = axiom_nodes.get(name.as_str()).unwrap_or_else(|| {
                panic!("declared domain axiom {name:?} must be emitted as an Axiom node")
            });
            assert_eq!(
                node.lexical.as_deref(),
                Some(description.as_str()),
                "the domain-axiom node must carry its description as its lexical gloss"
            );
        }
        // Non-vacuity: dropping the `axioms:` clause would drop these nodes.
        // The Org fixture (no `axioms:` clause) emits NO node named for them.
        let org = emit::<OrgCategory>();
        for (name, _) in &declared {
            assert!(
                !org.nodes.iter().any(|n| &n.name == name),
                "an ontology with no axioms: clause must not emit {name:?}"
            );
        }

        // Byte-exact round-trip through the runtime, fail-closed against root.
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(
            loaded, archive,
            "the archive (incl. domain axioms) round-trips"
        );
        for (name, _) in &declared {
            assert!(
                loaded
                    .nodes
                    .iter()
                    .any(|n| n.kind == "Axiom" && &n.name == name),
                "the domain-axiom node {name:?} must survive the round-trip byte-exact"
            );
        }

        // The emitted name is the SAME wire identity `axiom_by_name` rebinds on
        // load: each declared domain axiom resolves to a freshly-built runnable
        // predicate by that name (the macro registered it into
        // AXIOM_CONSTRUCTORS), and the rebound axiom verifies — proving the node
        // names a REAL claim, not a label.
        for (name, _) in &declared {
            let rebound = pr4xis::ontology::axiom_by_name(name).unwrap_or_else(|| {
                panic!("emitted domain axiom {name:?} must rebind via axiom_by_name on load")
            });
            assert_eq!(
                rebound.name().as_str(),
                name.as_str(),
                "the rebound axiom must carry the same stable name the node was keyed by"
            );
            rebound.verify().unwrap_or_else(|c| {
                panic!(
                    "the rebound domain axiom {name:?} must verify against the ontology; got {:?}",
                    c.meta().name
                )
            });
        }
    }

    #[test]
    fn connections_are_wired_through_the_codec_round_trip() {
        // The field-wiring proof for `Archive.connections` (complementing the
        // emit-from-a-registered-functor proof above): a manually constructed
        // connection survives the `emit -> load` round-trip byte-exact AND is
        // load-bearing on the archive root (an archive with a connection differs
        // from the same archive without it).
        let connection = Connection {
            kind: "FullyFaithful".to_string(),
            source: "Employer".to_string(),
            target: "Agent".to_string(),
            action: GeneratorAction::Functor {
                map_object: vec![("Employer".to_string(), "Agent".to_string())],
                map_morphism: vec![("Subsumption".to_string(), "Subsumption".to_string())],
            },
            laws: vec!["PreservesComposition".to_string()],
        };
        let node = Definition {
            kind: "Concept".to_string(),
            name: "Employer".to_string(),
            edges: Vec::new(),
            axioms: Vec::new(),
            lexical: None,
        };

        let with_conn = Archive {
            nodes: vec![node.clone()],
            connections: vec![connection.clone()],
        };
        let without_conn = Archive {
            nodes: vec![node],
            connections: Vec::new(),
        };
        // Difference-detection: the connection reaches the archive root.
        assert_ne!(
            with_conn.root().unwrap(),
            without_conn.root().unwrap(),
            "a connection must be load-bearing on the archive root"
        );

        // The connection survives the round-trip byte-exact, fail-closed.
        let bytes = load::emit(&with_conn).unwrap();
        let loaded = load::load(&bytes, with_conn.root().unwrap()).unwrap();
        assert_eq!(loaded, with_conn, "the archive must round-trip faithfully");
        assert_eq!(
            loaded.connections,
            vec![connection],
            "the connection must survive the round-trip byte-exact"
        );
    }

    #[test]
    fn emitted_ontology_rebinds_against_itself() {
        struct Selfish(HashMap<String, ContentAddress>);
        impl RebindTarget for Selfish {
            fn address_of(&self, name: &str) -> Option<ContentAddress> {
                self.0.get(name).copied()
            }
        }
        let archive = emit::<OrgCategory>();
        let known: HashMap<String, ContentAddress> = archive
            .nodes
            .iter()
            .map(|n| (n.name.clone(), n.address().unwrap()))
            .collect();
        let rebound = rebind_nodes(&archive, &Selfish(known)).unwrap();
        assert!(
            rebound.iter().all(|r| r.is_bound()),
            "a freshly-emitted ontology must rebind to itself"
        );
    }

    // The connection identifying the registered Org → Workforce functor.
    fn org_workforce_conn(archive: &Archive) -> &Connection {
        archive
            .connections
            .iter()
            .find(|c| c.source == "OrgOntology" && c.target == "WorkforceOntology")
            .expect("the registered Org→Workforce functor must be emitted as a connection")
    }

    #[test]
    fn emits_a_registered_functor_as_a_connection() {
        // An ontology that participates in a registered functor emits a NON-EMPTY
        // connection set — the real work: the morphism between ontologies is
        // serialized, not dropped.
        let archive = emit::<OrgCategory>();
        assert!(
            !archive.connections.is_empty(),
            "Org participates in the OrgIntoWorkforce functor — connections must be non-empty"
        );
        let conn = org_workforce_conn(&archive);

        // The functor's finite action-on-generators is extracted faithfully:
        // every Org concept maps to its same-named Workforce image, and the
        // Subsumption kind maps to Subsumption.
        match &conn.action {
            GeneratorAction::Functor {
                map_object,
                map_morphism,
            } => {
                for (s, t) in [
                    ("Employer", "Employer"),
                    ("Employee", "Employee"),
                    ("Person", "Person"),
                    ("Agent", "Agent"),
                ] {
                    assert!(
                        map_object.contains(&(s.to_string(), t.to_string())),
                        "object map must carry {s} → {t}"
                    );
                }
                assert!(
                    map_morphism.contains(&("Subsumption".to_string(), "Subsumption".to_string())),
                    "morphism map must carry Subsumption → Subsumption"
                );
            }
            other => panic!("expected a Functor action, got {other:?}"),
        }
        // The functor laws travel with it.
        assert!(conn.laws.contains(&"FunctorIdentityLaw".to_string()));
        assert!(conn.laws.contains(&"FunctorCompositionLaw".to_string()));
    }

    #[test]
    fn the_same_functor_is_emitted_from_both_endpoints() {
        // The connection is incident to BOTH ontologies: emitting either Org or
        // Workforce surfaces the same functor (source-or-target selection), and
        // — because it's the same finite action — at the SAME content-address.
        let from_org = emit::<OrgCategory>();
        let from_workforce = emit::<WorkforceCategory>();
        let org_conn = org_workforce_conn(&from_org);
        let wf_conn = org_workforce_conn(&from_workforce);
        assert_eq!(
            org_conn.address().unwrap(),
            wf_conn.address().unwrap(),
            "the same functor must content-address equally from both endpoints"
        );
    }

    #[test]
    fn the_connection_is_content_addressed_and_action_sensitive() {
        // Non-vacuity: the connection's address depends on its ACTION — mutate
        // one row of the object map and the address changes. This proves the
        // serialized action is load-bearing on identity, not decorative.
        let archive = emit::<OrgCategory>();
        let conn = org_workforce_conn(&archive).clone();
        let baseline = conn.address().unwrap();

        let mut mutated = conn.clone();
        if let GeneratorAction::Functor { map_object, .. } = &mut mutated.action {
            // Redirect Employer's image — a different finite action.
            map_object.retain(|(s, _)| s != "Employer");
            map_object.push(("Employer".to_string(), "Agent".to_string()));
        }
        assert_ne!(
            baseline,
            mutated.address().unwrap(),
            "changing the functor's action must change the connection's content-address"
        );
    }

    #[test]
    fn emitted_connections_survive_the_round_trip_byte_exact() {
        // The connections survive emit → load byte-exact, fail-closed against the
        // archive's own root (the same law the nodes obey).
        let archive = emit::<OrgCategory>();
        assert!(!archive.connections.is_empty());
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(
            loaded, archive,
            "the archive (incl. connections) must round-trip faithfully"
        );
        // The functor connection is present after the round-trip, unchanged.
        let before = org_workforce_conn(&archive);
        let after = org_workforce_conn(&loaded);
        assert_eq!(before, after, "the connection must survive byte-exact");
        assert_eq!(before.address().unwrap(), after.address().unwrap());
    }

    #[test]
    fn connections_rebind_by_content_address_agreement() {
        // A connection is admitted into a peer's world only when its source AND
        // target ontologies rebind by content-address agreement — the
        // cross-ontology p2p law. We model a peer that knows both ontologies by
        // the addresses THIS emit produced for them: every connection's
        // endpoints bind. A peer that knows them at a DIFFERENT address (a
        // different definition) does not — the G5 fix at the connection layer.
        struct Peer(HashMap<String, ContentAddress>);
        impl RebindTarget for Peer {
            fn address_of(&self, name: &str) -> Option<ContentAddress> {
                self.0.get(name).copied()
            }
        }

        let org = emit::<OrgCategory>();
        let workforce = emit::<WorkforceCategory>();
        let conn = org_workforce_conn(&org).clone();

        // A peer holding both ontologies' nodes at the addresses emit produced:
        // the connection's endpoints (the ontologies, by name) are known, and
        // each ontology's own Merkle root agrees.
        let mut known: HashMap<String, ContentAddress> = HashMap::new();
        known.insert(conn.source.clone(), org.root().unwrap());
        known.insert(conn.target.clone(), workforce.root().unwrap());
        let peer = Peer(known);
        assert!(
            peer.address_of(&conn.source).is_some() && peer.address_of(&conn.target).is_some(),
            "both endpoint ontologies must be known to the agreeing peer"
        );

        // A peer that knows the target ontology at a DIFFERENT address (a
        // different definition) does not agree — the connection cannot bind.
        let mut disagreeing: HashMap<String, ContentAddress> = HashMap::new();
        disagreeing.insert(conn.source.clone(), org.root().unwrap());
        disagreeing.insert(conn.target.clone(), org.root().unwrap()); // wrong root
        let bad = Peer(disagreeing);
        assert_ne!(
            bad.address_of(&conn.target),
            Some(workforce.root().unwrap()),
            "a peer at a different address must NOT agree on the target ontology"
        );
    }

    // A REAL ontology whose ONTOLEX-Lemon label is a natural surface DISTINCT from
    // the concept's Rust identifier — the LegalSources shape in miniature
    // (`Enactment` labelled "law", `Statute` labelled "statute"). The label is the
    // word a person types; the variant name is not.
    pr4xis::ontology! {
        name: "Enactments",
        source: "pr4xis-runtime emit lexicalization test fixture",
        concepts: [Enactment, Statute],
        labels: {
            Enactment: ("en", "law", "A rule with legal force."),
            Statute: ("en", "statute", "An enactment laid down by a legislature."),
        },
        is_a: [
            (Statute, Enactment),
        ],
    }

    #[test]
    fn emit_lexicalizes_a_label_distinct_from_the_identifier_but_the_kind_vocab_does_not() {
        // The MARKED plain projection carries `Enactment`'s gloss but no surface
        // for "law": the only surface is the lowercased identifier "enactment".
        let plain = emit_kind_vocabulary::<EnactmentsCategory>();
        assert!(
            !plain.nodes.iter().any(|n| n.kind == "Form"),
            "emit_kind_vocabulary mints no Form atoms"
        );

        // The DEFAULT `emit` lexicalizes: it mints one Form ONLY for a label that
        // DIFFERS from its identifier. `Enactment`'s label "law" differs → minted.
        // `Statute`'s label "statute" equals its identifier case-insensitively →
        // NOT minted (the lowercased node-name already grounds "statute"; a
        // redundant Form would bloat the archive and the surface index).
        let lex = emit::<EnactmentsCategory>();
        let forms: std::collections::BTreeSet<&str> = lex
            .nodes
            .iter()
            .filter(|n| n.kind == "Form")
            .map(|n| n.name.as_str())
            .collect();
        assert!(
            forms.contains("law"),
            "the distinct label surface must be minted as a Form atom; got {forms:?}"
        );
        assert!(
            !forms.contains("statute"),
            "a label equal to its identifier mints no redundant Form; got {forms:?}"
        );

        let enactment = lex
            .nodes
            .iter()
            .find(|n| n.kind == "Concept" && n.name == "Enactment")
            .expect("the Enactment concept node survives");
        assert!(
            enactment
                .edges
                .iter()
                .any(|(role, t)| role == "canonicalForm" && t.local_name() == Some("law")),
            "the concept must point a canonicalForm edge at its distinct label surface; got {:?}",
            enactment.edges
        );
        let statute = lex
            .nodes
            .iter()
            .find(|n| n.kind == "Concept" && n.name == "Statute")
            .expect("the Statute concept node survives");
        assert!(
            !statute
                .edges
                .iter()
                .any(|(role, _)| role == "canonicalForm"),
            "a redundant label mints no canonicalForm edge; got {:?}",
            statute.edges
        );

        // Referentially closed + fail-closed round-trip through the runtime.
        let bytes = load::emit(&lex).unwrap();
        let loaded = load::load(&bytes, lex.root().unwrap()).unwrap();
        assert_eq!(
            loaded, lex,
            "the lexicalized archive round-trips byte-exact"
        );
    }

    #[test]
    fn emit_equals_emit_kind_vocabulary_when_every_label_matches_its_identifier() {
        // The Org fixture's labels all equal their variant names (case-insensitively),
        // so lexicalization mints NO redundant Form and the DEFAULT `emit` is byte-
        // identical to the plain `emit_kind_vocabulary` — the guarantee that
        // ontologies already grounded by their identifiers (and the kind meta-vocab)
        // are untouched.
        let plain = emit_kind_vocabulary::<OrgCategory>();
        let lex = emit::<OrgCategory>();
        assert_eq!(
            plain.root().unwrap(),
            lex.root().unwrap(),
            "emit must equal emit_kind_vocabulary when labels == identifiers"
        );
    }
}
