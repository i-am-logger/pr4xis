//! Statute-local lexicon MINTING — the capability G7 (the "written-form
//! floor" out-of-lexicon Form minting, `defines_lens`) needs: bind a
//! brand-new term with NO existing WordNet or loaded-lexicon entry (a
//! statutory coinage — "assistant secretary", "qualified medicare
//! beneficiary" — that will never appear in WordNet) into a domain-scoped
//! [`Lexicon`] namespace, AND materialize the matching [`RuntimeOntology`] so
//! the term becomes resolvable through the SAME lexical index every other
//! loaded surface goes through ([`ComposedReasoner`](crate::cognitive::linguistics::composed::ComposedReasoner)),
//! never a parallel ad-hoc lookup table.
//!
//! # Two outputs, one call — reusing what already exists
//!
//! [`mint`] does not invent a new lexicon representation. It COMPOSES the two
//! capabilities that already exist for exactly this shape of data:
//!
//! 1. [`Lexicon::add_sense`] — records the `(term, domain)` binding as a
//!    domain-scoped [`Sense`](super::lexicon::Sense) (`Sense::in_domain`'s
//!    existing salience-elevation mechanism), so a query already scoped to
//!    this statute's domain resolves the coinage as predominant, exactly the
//!    way a legal-domain sense of "person" is already elevated over its
//!    general WordNet sense (Koeling, McCarthy & Carroll 2005).
//! 2. [`materialize`] over a two-node [`Archive`] shaped exactly like every
//!    other producer in this codebase mints a lexicalized concept
//!    ([`synset_definition`](crate::cognitive::linguistics::english::bridge::synset_definition),
//!    `project_lexicon_archive`, the compiled-ontology `emit`): one `Concept`
//!    node carrying a `canonicalForm` edge to one `ontolex:Form` atom
//!    ([`form_atom`]). This is the SAME two-node shape `ComposedReasoner`'s
//!    construction-time indexing already recognizes (it assigns a queryable
//!    `ConceptId` to the Concept and indexes the Form's `writtenRep` as one of
//!    its surfaces) — a minted term needs no special-cased reading, only
//!    composition into a `loaded` ontology list.
//!
//! # The reference is a content-addressed pointer, not a duplicated string
//!
//! Per the "words are pointers into English" discipline
//! ([`form_atom`]; `denotes`/`defines` grounding already resolves this way —
//! see [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)),
//! the minted concept's IDENTITY is the content address of its own `Form`
//! atom ([`MintedTerm::form_address`]), never the raw term string re-used as
//! an arbitrary key. Two mint calls for the SAME term in the SAME domain
//! therefore derive the SAME [`ConceptRef`] and the SAME archive root — minting
//! is a deterministic, idempotent operation (verified by
//! `mint_is_deterministic_the_same_term_and_domain_mint_the_same_reference`),
//! matching [`Lexicon::add_sense`]'s own documented idempotency for the sense
//! it records.
//!
//! Literature:
//! - McCrae, Bosque-Gil, Gracia, Buitelaar & Cimiano (2017) *The OntoLex-Lemon
//!   Model*, Proc. eLex 2017 — a `LexicalEntry`'s `Form` carries the surface,
//!   its `Sense` points at the concept; the shape [`mint`] produces.
//! - Koeling, McCarthy & Carroll (2005) *Domain-Specific Sense Distributions
//!   and Predominant Sense Acquisition*, HLT/EMNLP — the domain-scoped sense
//!   elevation `Lexicon::add_sense`'s `domain` argument realizes.
//! - Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) *BLAKE3: one function,
//!   fast everywhere* — the content-addressing primitive a minted term's
//!   identity bottoms out in ([`ContentAddress::of`]).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::{CANONICAL_FORM_REL, Definition, EdgeTarget};
use pr4xis_runtime::ontology::{ConceptRef, MaterializeError, RuntimeOntology, materialize};

use crate::cognitive::linguistics::english::bridge::form_atom;

use super::lexicon::Lexicon;

/// The praxis kind a minted term's wrapping node carries — the SAME
/// `CONCEPT_KIND` [`crate::cognitive::linguistics::english::bridge`] relabels
/// every synset into, so a minted concept is structurally indistinguishable
/// from any other loaded `Concept` node to a generic reasoner.
const MINTED_CONCEPT_KIND: &str = "Concept";

/// A freshly minted lexicon entry — the resolvable reference [`mint`] hands
/// back to a caller (the future `defines_lens` G7 fix) that just bound a
/// statutory coinage with no prior lexicon entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintedTerm {
    /// The domain-scoped [`ConceptRef`] — the SAME typed `(ontology, name)`
    /// pointer [`crate::cognitive::linguistics::composed::ComposedReasoner`]
    /// indexes every loaded concept by. Compose the paired
    /// [`RuntimeOntology`] this call also returns into a `loaded` list (e.g.
    /// `ComposedReasoner::new`'s second argument) and this reference resolves
    /// through the SAME `lookup`/`decode`/`concept` surface every other
    /// loaded term does.
    pub reference: ConceptRef,
    /// The content address of the term's `ontolex:Form` atom — the honest
    /// written-form-floor target [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
    /// and [`denotes_pointers`](crate::social::judicial::statute_structure::grounding::denotes_pointers)
    /// already ground `EdgeTarget::Grounded` edges at, so a G7 caller can
    /// point a `defines`/`denotes` edge at THIS statute-local Form instead of
    /// `english_wordnet`'s when the term has no WordNet entry.
    pub form_address: ContentAddress,
}

/// Mint `term` into `domain`'s statute-local lexicon namespace.
///
/// 1. Records the `(term, domain, minted-concept)` triple as a domain-scoped
///    [`Sense`](super::lexicon::Sense) on `lexicon` via [`Lexicon::add_sense`]
///    — idempotent, exactly as every other call to that method is.
/// 2. Materializes a two-node [`RuntimeOntology`] named `domain`: one
///    `Concept` node (named by its own content address — see the module
///    doc's "pointer, not a duplicated string") carrying a `canonicalForm`
///    edge to `term`'s [`form_atom`], plus that `Form` atom itself. The
///    caller composes this into a `loaded` ontology list (e.g.
///    `ComposedReasoner::new`) so the general lexical index resolves the
///    coinage.
///
/// `gloss`, when given, becomes the minted concept's lexical grounding — the
/// SAME field [`synset_definition`](crate::cognitive::linguistics::english::bridge::synset_definition)
/// fills from a synset's WordNet definition; here it would be the statute's
/// own definiens once G7 composes with `defines_pointers`.
///
/// Fails only if content-addressing itself fails (an
/// [`alloc::string`]-encoding fault; unreachable in practice — see
/// [`Definition::address`]) or if the two-node archive somehow fails
/// referential closure, which cannot happen here: the Concept's ONE edge
/// targets the Form by its own `name`, which the same call always declares.
pub fn mint(
    lexicon: &mut Lexicon,
    domain: OntologyName,
    term: &str,
    gloss: Option<&str>,
) -> Result<(MintedTerm, RuntimeOntology), MaterializeError> {
    let form = form_atom(term);
    let form_address = form.address().map_err(MaterializeError::Root)?;
    // The concept's identity is the content address of its own Form atom — a
    // pointer, never the term string duplicated as an arbitrary key (see the
    // module doc). Two calls for the same term derive the same name here.
    let concept_name = form_address.to_hex();

    lexicon.add_sense(
        term.to_string(),
        domain.as_str().to_string(),
        concept_name.clone(),
        Some(domain.as_str().to_string()),
    );

    let concept = Definition {
        kind: MINTED_CONCEPT_KIND.to_string(),
        name: concept_name.clone(),
        edges: vec![(
            CANONICAL_FORM_REL.to_string(),
            EdgeTarget::Local(term.to_string()),
        )],
        axioms: Vec::new(),
        lexical: gloss.map(ToString::to_string),
    };
    let archive = Archive {
        nodes: vec![concept, form],
        connections: Vec::new(),
    };
    let onto = materialize(archive, domain.clone())?;

    Ok((
        MintedTerm {
            reference: ConceptRef::new(domain, concept_name),
            form_address,
        },
        onto,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SAME resolution path a real caller (the future G7 fix) would use
    /// on `Lexicon`: `resolve(term, Some(domain))` finds NOTHING before
    /// minting and the minted concept AFTER — over several distinct real
    /// out-of-lexicon statutory coinages (the exact motivating G7 examples),
    /// not one hand-picked string in isolation.
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn mint_records_a_domain_scoped_sense_resolvable_through_the_lexicon() {
        let domain = OntologyName::new_static("usc_t42_coinages");
        for term in ["assistant secretary", "qualified medicare beneficiary"] {
            let mut lexicon = Lexicon::new("en");
            assert!(
                lexicon.resolve(term, Some(domain.as_str())).is_none(),
                "{term:?} must be out-of-lexicon before minting"
            );

            let (minted, _onto) = mint(&mut lexicon, domain.clone(), term, None)
                .expect("a two-node archive with a self-consistent edge always materializes");

            let sense = lexicon.resolve(term, Some(domain.as_str())).expect(
                "the same term resolves through the SAME Lexicon::resolve path after minting",
            );
            assert_eq!(sense.reference.ontology, domain.as_str());
            assert_eq!(sense.reference.concept, minted.reference.name);
            assert_eq!(sense.domain.as_deref(), Some(domain.as_str()));
        }
    }

    /// Minting the SAME term in the SAME domain twice derives the SAME
    /// reference and the SAME archive root — the content-addressed-identity
    /// round-trip claim the module doc makes. Idempotent, matching
    /// `Lexicon::add_sense`'s own documented idempotency.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn mint_is_deterministic_the_same_term_and_domain_mint_the_same_reference() {
        let domain = OntologyName::new_static("usc_t42_coinages");
        let mut lexicon = Lexicon::new("en");

        let (first, first_onto) = mint(
            &mut lexicon,
            domain.clone(),
            "qualified medicare beneficiary",
            None,
        )
        .expect("mints");
        let (second, second_onto) = mint(
            &mut lexicon,
            domain.clone(),
            "qualified medicare beneficiary",
            None,
        )
        .expect("mints");

        assert_eq!(
            first, second,
            "the same (term, domain) mint the same reference"
        );
        assert_eq!(
            first_onto.root(),
            second_onto.root(),
            "the same (term, domain) materialize byte-identical archives"
        );
        // Re-minting is idempotent on the Lexicon side too — add_sense does not
        // duplicate the identical sense (Lexicon's own documented contract).
        let entry = lexicon
            .lookup("qualified medicare beneficiary")
            .expect("the term has an entry");
        assert_eq!(
            entry.senses.len(),
            1,
            "the repeat mint added no duplicate sense"
        );
    }

    /// Distinct terms — or the SAME term in distinct domains — mint distinct
    /// references: the content address is a genuine identity, not a constant.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn distinct_terms_or_domains_mint_distinct_references() {
        let domain = OntologyName::new_static("usc_t42_coinages");
        let mut lexicon = Lexicon::new("en");

        let (secretary, _) =
            mint(&mut lexicon, domain.clone(), "assistant secretary", None).unwrap();
        let (beneficiary, _) = mint(
            &mut lexicon,
            domain.clone(),
            "qualified medicare beneficiary",
            None,
        )
        .unwrap();
        assert_ne!(
            secretary.reference, beneficiary.reference,
            "distinct terms in the same domain mint distinct references"
        );
        assert_ne!(secretary.form_address, beneficiary.form_address);

        let other_domain = OntologyName::new_static("usc_t20_coinages");
        let (secretary_elsewhere, _) = mint(
            &mut lexicon,
            other_domain.clone(),
            "assistant secretary",
            None,
        )
        .unwrap();
        assert_ne!(
            secretary.reference, secretary_elsewhere.reference,
            "the same term minted into a different domain gets a distinct ontology-scoped reference"
        );
        // The Form atom is a property of the WRITTEN FORM alone, so it agrees
        // across domains — only the domain-scoped Concept reference differs.
        assert_eq!(secretary.form_address, secretary_elsewhere.form_address);
    }

    /// The minted archive is a genuine [`RuntimeOntology`]: the concept
    /// resolves by name, carries the gloss, and its `canonicalForm` edge
    /// targets the Form atom — the exact two-node shape a real caller
    /// composes into `ComposedReasoner`'s `loaded` list (proven end-to-end in
    /// `composed.rs`'s `a_minted_statute_local_term_resolves_through_the_composed_lexical_index`).
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn the_minted_archive_materializes_and_carries_the_gloss() {
        let domain = OntologyName::new_static("usc_t42_coinages");
        let mut lexicon = Lexicon::new("en");
        let (minted, onto) = mint(
            &mut lexicon,
            domain.clone(),
            "qualified medicare beneficiary",
            Some("an individual entitled to Medicare Part A"),
        )
        .expect("mints");

        assert_eq!(onto.id(), &domain);
        assert_eq!(
            onto.lexical(&minted.reference),
            Some("an individual entitled to Medicare Part A"),
            "the gloss rides into the minted concept's lexical grounding"
        );
        let node = onto
            .node_by_name(&minted.reference.name)
            .expect("the minted concept is a declared node")
            .expect("it decodes to an owned Definition");
        assert_eq!(node.kind, MINTED_CONCEPT_KIND);
        assert_eq!(
            node.edges,
            vec![(
                CANONICAL_FORM_REL.to_string(),
                EdgeTarget::Local("qualified medicare beneficiary".to_string()),
            )],
            "the minted concept's only edge is its canonicalForm pointer to the Form atom"
        );
    }
}
