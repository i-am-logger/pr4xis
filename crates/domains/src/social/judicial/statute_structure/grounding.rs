//! Grounding statute prose into the English lexicon — the honest written-form
//! `denotes` floor at the statute level, as a grounding lens over the generic
//! substrate.
//!
//! A span of statute text is scanned for content-word lemmas
//! ([`extract_lemmas`](crate::social::judicial::statute_structure::term_extractor::extract_lemmas)
//! — stopwords and numerals filtered, deduped); each lemma that English knows as
//! a written form
//! becomes a typed `denotes` pointer into the `english_wordnet` archive: a
//! [`Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge targeting
//! the word's [`ontolex:Form`](crate::cognitive::linguistics::english::bridge::form_atom)
//! atom by content address.
//!
//! It is the WRITTEN-FORM FLOOR (the weakest adequate claim): the pointer lands
//! on a Form — "this written form occurred" — NEVER on a sense. Fine-grained
//! word-sense disambiguation is 59–82% accurate (Navigli 2009, "Word Sense
//! Disambiguation: A Survey", ACM Comput. Surv. 41(2)); a written-form anchor is
//! ~0 error — the design's written-form-floor decision. Sense is licensed only
//! by a statute's own definitions, a stronger kind deferred.
//!
//! # The ontological, general way
//!
//! `denotes` is ONE grounding lens. [`denotes_lens`](crate::social::judicial::statute_structure::grounding::denotes_lens) adapts
//! the producer to the generic [`ground`](pr4xis_runtime::grounding::ground): any
//! content [`Archive`](pr4xis_runtime::archive::Archive) — a USC title projected
//! by `uslm::corpus::bridge`, English itself, anything — grounds the same way,
//! gaining typed
//! [`EdgeTarget::Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded)
//! edges in the GENERIC substrate that
//! resolve through the generic `AtomResolver`. English is confined to the lens;
//! [`cites_lens`](crate::social::judicial::statute_structure::grounding::cites_lens)
//! is a second lens of the same shape — a provision's typed `UsCodeRef`
//! citations resolving `Local` (same archive), `Grounded` (a supplied peer
//! title), or unresolved (no edge, no coverage) — and
//! [`defines_lens`](crate::social::judicial::statute_structure::grounding::defines_lens)
//! is a third: it recognizes the "the term 'X' means Y" declarative shape (a
//! close-apposition definiendum subject, a VerbNet-confirmed
//! Theme/Co-Theme-ordered "mean"-class verb) by running the FULL
//! tokenize/chart/Montague pipeline over a provision's prose, never a
//! string/regex match. Its coverage is deliberately bounded to what that
//! grammar actually derives: it recognizes single-definiendum, single-clause
//! declaratives, OPTIONALLY preceded by a FRONTED (pre-subject) scope-setting
//! sentential adjunct — "For purposes of this subsection," / "In this
//! subsection," / "Except for the purposes of X," / "Subject to Y," (the
//! defines-lens gap backlog's G1: [`crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_np`]/
//! [`fronted_scope_adjunct_pp`](crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_pp),
//! CCGbank's own `S/S` sentence-initial-adjunct category, Hockenmaier &
//! Steedman 2007 §3.6 — the adjunct is consumed as a transparent
//! scope-setting modifier and contributes no argument of its own,
//! `montague::apply`'s S-result branch) — OPTIONALLY interrupted by a
//! MEDIAL, comma-delimited supplement in either of two positions (the
//! backlog's G2): between the definiendum and its verb ("the term 'X', used
//! with respect to Y, means Z" / "the term 'X', as used in this title, means
//! Z"), or between the verb and its object ("the term 'X' means, with
//! respect to Y, Z" — the EVV headline shape, 42 U.S.C. § 1396b(l)(5)) —
//! [`crate::cognitive::linguistics::lambek::types::svo::medial_supplement_np`]/
//! [`medial_supplement_verb`](crate::cognitive::linguistics::lambek::types::svo::medial_supplement_verb),
//! Huddleston & Pullum (2002) Ch. 15 "Supplements": the ENTIRE bracketed
//! interior is merged into one opaque token by the tokenizer
//! (`tokenize::collapse_medial_comma_adjuncts`) and dropped transparently by
//! `montague::apply`, so its own internal shape (participial, PP-headed,
//! whatever) never needs to parse. The DEFINIENS side may also be an n-ary
//! COORDINATED list — "bodily injury, impairment, or disease" (42 U.S.C. §
//! 3002(42)), "an unpaid family member, a foster parent, or another unpaid
//! individual" (the backlog's G4(a)):
//! `lambek::tokenize::find_list_coordinator_commas`
//! mints each list-separator comma as its own instance of the EXISTING
//! `nominal_coordinator_np`/`nominal_coordinator_n` category (Huddleston &
//! Pullum (2002) Ch. 15 §3, "coordination of three or more elements": a
//! comma stands in for the list's OWN repeated conjunction, not a separate
//! construction), reducing an n-item list by iterated binary application;
//! `montague::flatten_coordination` builds the genuine n-ary meaning
//! (Partee & Rooth 1983, "generalized conjunction") rather than the earlier
//! "just keep the left conjunct" default. A determiner-LESS conjunct
//! ("contract" in "a grant, contract, or cooperative agreement," 42 U.S.C. §
//! 289b–1(f)(2); "abuse" in "abuse of an older individual," 42 U.S.C. §
//! 3002(15) — the backlog's G4(f)) is licensed via
//! this module's own `definiens_cost_table`'s SCOPED `N → NP` unary promotion — a real,
//! published CCGbank rule (`crate::cognitive::linguistics::lambek::
//! supertag_costs::bare_noun_phrase_unary_rule`) a corpus-gate measurement
//! already rejected at the GLOBAL level, so it is composed onto a NEW table
//! `defines_pointers` alone uses, never the shared production one the live
//! chat pipeline runs on
//! (`crate::cognitive::linguistics::lambek::supertag_costs::
//! SupertagCostTable::with_extra_unary`'s own doc has the full citation and
//! rationale); the promotion applies to a derived multi-word SPAN as well as
//! a lexical leaf (`montague::interpret_with_unary_rules`'s own per-cell
//! closure, mirroring `reduce::close_unary`'s fixpoint). A definiens-side PP
//! CHAIN ("the Secretary of Health and Human Services," 42 U.S.C. §
//! 242q–4(2) — the backlog's G4(b)) needs no new mechanism at all: the
//! PRE-EXISTING `svo::preposition` category already composes it once its
//! own conjuncts (bare or coordinated) resolve. Definiens-side RELATIVE
//! CLAUSES (G4(c)) and PARTICIPIAL postmodifiers (G4(d)) remain
//! UNSCAFFOLDED beyond what the loaded `RelativePronoun` category already
//! covers for a SUBJECT relative in isolation (`the dog that runs`); a bare
//! noun-noun compound ("family member": no adjective sense loaded for
//! "family") and an out-of-vocabulary attributive adjective ("in-home") are
//! SEPARATE, unaddressed gaps this module's own test suite documents
//! honestly rather than silently working around. COORDINATED DEFINIENDA
//! (the backlog's G5 — "the terms 'exploitation' and 'financial
//! exploitation' mean ...", 42 U.S.C. § 3002(18)(A); "the term 'X' and the
//! term 'Y' mean ...", 42 U.S.C. § 1395x(aa)(5)(A)) DO reduce: a plural
//! "the terms" ALSO takes a coordinated PAIR of quoted definienda as one
//! combined close-apposition modifier —
//! [`crate::cognitive::linguistics::lambek::types::svo::nominal_coordinator_apposition`]
//! (Steedman's general `(X\X)/X` coordination schema instantiated at
//! `X = close_apposition`), minted only for the tokenizer's OWN reserved
//! marker (`tokenize::mark_apposition_coordinators`, gated on "and"/"or"
//! sitting directly between two quote-collapsed spans, OR bridging a
//! repeated "the term(s)" prefix on the right — "and the term 'Y'" — the
//! SAME closed-vocabulary bridging `tokenize::skip_the_term_prefix`
//! documents, needed for the coordinated-FULL-NP variant) — NEVER the
//! literal "and"/"or" surface, deliberately: the derived shape is
//! structurally IDENTICAL to ordinary NP-subject coordination (a REAL,
//! measured regression this exact restriction fixes — `definiendum_words`'s
//! own doc has the full story), and `Sem::Concept` carries no provenance
//! that could otherwise tell a close-apposition-promoted quoted
//! definiendum apart from an ordinary determined common noun once
//! coordinated. `definiendum_words` extracts every conjunct, so one
//! shared "means"/"includes" declarative yields ONE `DefinesPointer` PER
//! coordinated term. Colon-introduced enumerated definientia beyond what
//! G3's reassembly already recovers still have no further grammar
//! scaffolding, so a sentence in that shape simply fails to reduce to a
//! complete derivation and yields no pointer (the same honest "no
//! coverage → no pointer" discipline `denotes`/`cites` already
//! establish), not a guessed or partial extraction. There is no bespoke
//! string side-channel and no per-source codec.
//!
//! A GENUINELY DIFFERENT frame — RENVOI, definition-BY-REFERENCE (the
//! backlog's G6) — "The term 'skilled nursing facility' has the meaning
//! given such term in section 1395i–3(a) of this title." (42 U.S.C. §
//! 1395x(j)) / "...has the same meaning given the term in the Family
//! Violence Prevention and Services Act [42 U.S.C. 10401 et seq.]." (42
//! U.S.C. § 3002(19)) — states NO definiens locally at all, so it is not a
//! "means"/"includes" declarative and `defines_pointers` correctly never
//! reduces it to a complete `Sem::Prop`. `renvoi_pointers` is a
//! DELIBERATELY SEPARATE code path: `renvoi_predicate_start` recognizes
//! the closed legislative-drafting idiom "has"/"have" (+ "the" (+
//! "same")) "meaning" "given" over the TOKEN sequence (never a
//! substring/regex match on raw text — the same closed-class-check
//! discipline `is_partial_definition_verb` establishes), the definiendum
//! span before it is read off the tokenizer's OWN close-apposition
//! alternative-offering (no chart/Montague derivation needed — there is no
//! verb in that span for one to reduce against), and the REFERENT/citation
//! phrase after "given" is never parsed at all: resolution composes
//! directly with the citation substrate, reusing `cites_pointers`
//! VERBATIM over the node's own `<ref>` citations — the SAME
//! `Local`/cross-title-`Grounded`/unresolved discipline `cites_lens`
//! already establishes, so an unresolved renvoi target is left
//! ungrounded, never guessed. The target is `DefinesByReferencePointer`,
//! a DISTINCT type from `DefinesPointer` (its target is WHERE the
//! meaning lives — a provision, the SAME shape `cites` resolves to — never
//! a Form atom, since no meaning is stated here to ground). The RENVOI
//! mechanism itself recovers BOTH real headline definienda above (proven
//! by isolating each real predicate/citation with a WordNet-known
//! substitute definiendum — this module's own test suite's standing
//! precedent throughout G1-G5); neither real compound ("skilled nursing
//! facility", "family violence") is itself a WordNet-indexed headword, so
//! the REAL, unmodified sentences still ground nothing — the SAME
//! separately-scoped G7 written-form-floor gap, honestly, not a renvoi
//! mechanism failure.
//!
//! `defines_lens` also takes a `shadowed_prose` table — the fix for a
//! DIFFERENT, upstream gap (the defines-lens backlog's S1 and L1): a
//! provision's `Definition.lexical` is heading-first at the subdivision
//! level (`uslm::corpus::bridge::project_archive`'s
//! `heading.or(chapeau).or(content)`) and heading-ONLY at the section root
//! (the same projector's unconditional `lexical:
//! Some(section.heading.clone())`, with no body fallthrough at all), so a
//! heading-carrying subdivision's real definitional prose — and a
//! subdivision-LESS section's entire operative text — never even reach the
//! grammar above. `shadowed_prose`
//! (`uslm::corpus::bridge::defines_prose_index`'s output) recovers both as a
//! per-URN side-channel, the SAME shape `cites_lens` already takes
//! `refs_by_urn` in — never a second field on the shared `Definition`.
//!
//! `defines_lens` also takes a `reassembled` table — the STRUCTURAL fix for
//! a THIRD, related gap (the backlog's G3 dangling-chapeau enumeration and
//! S2 split-definition findings): a chapeau paragraph deliberately ends its
//! OWN sentence in an em dash or colon, with the definiens continuing in
//! enumerated `children` (or, for a genuinely split definition like 42
//! U.S.C. § 1396n(c)(5) "habilitation services", the "means"/"includes"
//! verb itself living on a SIBLING child) — no single node ever carries a
//! parseable declarative, so no per-node extractor can see it AT ALL,
//! independent of whatever the grammar above can otherwise derive.
//! `reassembled`
//! (`uslm::corpus::bridge::dangling_chapeau_reassembly_index`'s output)
//! recovers virtual-declarative-sentence candidates per dangling URN — the
//! SAME per-URN side-channel shape as `shadowed_prose`, run through the
//! IDENTICAL `defines_pointers` grammar check (no bespoke reassembly
//! grammar): a candidate whose reassembled shape the grammar still cannot
//! reduce (e.g. a coordinated relative-clause list — coordination has no
//! scaffolding yet, only turned into a syntactically complete sentence)
//! yields no pointer, honestly, exactly as before reassembly existed.
//! `is_partial_definition_verb` additionally recognizes "includes" as a
//! second, PARTIAL-definition verb frame alongside "means" (Dickerson's
//! means/includes distinction in legislative drafting,
//! [`DefinitionExhaustiveness`](crate::social::judicial::statute_structure::grounding::DefinitionExhaustiveness)'s
//! own doc carries the citation) — needed for S2, since a split
//! definition's "includes" clause never reaches a VerbNet-confirmed
//! "means"-class verb.
//!
//! The WRITTEN-FORM FLOOR itself (point 4 of
//! [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)'s
//! own doc) has two failure modes the backlog's G7 closes. First, a
//! genuinely CASE-marked WordNet lemma is unreachable by an exact-case
//! lookup alone — the tokenizer lowercases every surface before the floor
//! ever runs, so "Indian" (indexed capitalized) needs the SAME case-folded
//! fallback tier `crates/chat/src/lib.rs::resolve_surface` already applies
//! to a question surface (`is_known_written_form`'s own doc has the full
//! two-tier mechanism). Second, a definiendum WordNet will NEVER carry — a
//! genuine statutory coinage ("assistant secretary", "qualified medicare
//! beneficiary", …) the "the term 'X' …" declarative exists specifically to
//! mint — is no longer dropped:
//! [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
//! mints it into a caller-supplied statute-local namespace
//! ([`lemon::mint::mint`](crate::cognitive::linguistics::lemon::mint::mint),
//! McCrae, Bosque-Gil, Gracia, Buitelaar & Cimiano 2017's OntoLex-Lemon
//! `Form`/`Sense` shape — `mint`'s own doc has the full citation set), so
//! the pointer still grounds, at a content-addressed Form the mint domain
//! owns rather than `english_wordnet`.
//!
//! [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)'s
//! own chart derivation also runs under a WIDER, explicitly SCOPED
//! chart-width bound than the shared production one
//! (`DEFINES_MAX_CHART_WIDTH` — see that constant's own doc): the backlog
//! measured a real, complete, single-declarative Title 42 definition — 42
//! U.S.C. § 1395x(r) ("physician") — at 351 tokens, past the SHARED bound
//! (`lambek::reduce::chart_reduce_with_costs`'s `MAX_CHART_WIDTH`, 256,
//! sized for a live per-turn chat question, left completely UNCHANGED).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::grounding::LinkError;

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::english::LexicalReasoner;
use crate::cognitive::linguistics::english::bridge::{ENGLISH_ONTOLOGY, form_atom};
use crate::cognitive::linguistics::lambek::montague::{self, Sem};
use crate::cognitive::linguistics::lambek::reduce::clause_fragments_with_alternatives_and_table_and_width;
use crate::cognitive::linguistics::lambek::reduce::{ExpressionUse, TypedToken};
use crate::cognitive::linguistics::lambek::supertag_costs::{
    SupertagCostTable, bare_noun_phrase_unary_rule, reduced_passive_relative_unary_rule,
};
use crate::cognitive::linguistics::lambek::tokenize;
use crate::cognitive::linguistics::lambek::types::LambekType;
use crate::cognitive::linguistics::lambek::types::svo;
use crate::cognitive::linguistics::lemon::lexicon::Lexicon;
use crate::cognitive::linguistics::lemon::mint::mint;
use crate::cognitive::linguistics::morphology::lemmatizer::{self, Language as MorphLanguage};
use crate::cognitive::linguistics::verbnet::ontology::VerbNet;
use crate::social::software::markup::xml::uslm::UsCodeRef;
use crate::social::software::markup::xml::uslm::corpus::title_number_of_urn;

use super::term_extractor::extract_lemmas;

/// One written-form `denotes` pointer: the surface `word` that occurred and the
/// [`Grounded`](EdgeTarget::Grounded) edge into its `ontolex:Form` atom in
/// `english_wordnet`. The edge is what a statute subdivision carries; resolving
/// it (via the runtime `AtomResolver`) yields the Form atom — never a sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenotesPointer {
    /// The written form (lowercased lemma) that occurred in the prose.
    pub word: String,
    /// The grounded edge — `denotes` into the word's Form atom by content address.
    pub target: EdgeTarget,
}

/// The written-form `denotes` pointers for a span of statute prose: one per
/// content-word lemma English knows as a written form, each pointing at that
/// word's [`form_atom`] by content address.
///
/// A word English does not know is left UNGROUNDED (no pointer) — the floor only
/// asserts forms that actually exist in the lexicon, never invents a target.
pub fn denotes_pointers(text: &str, english: &English) -> Vec<DenotesPointer> {
    extract_lemmas(text)
        .into_iter()
        // Only a written form English actually knows is grounded — the floor
        // never points at a non-existent atom.
        .filter(|form| !english.lookup(&form.written_rep).is_empty())
        .filter_map(|form| {
            let atom = form_atom(&form.written_rep).address().ok()?;
            Some(DenotesPointer {
                word: form.written_rep.clone(),
                target: EdgeTarget::Grounded {
                    ontology: ENGLISH_ONTOLOGY.to_string(),
                    atom,
                },
            })
        })
        .collect()
}

/// The lexical `denotes` grounding LENS — the lens form of [`denotes_pointers`],
/// for the generic [`ground`](pr4xis_runtime::grounding::ground).
///
/// It grounds ANY archive node (a statute provision, …) into the English
/// `ontolex:Form` atoms its lexical prose denotes, producing typed
/// `(denotes, `[`EdgeTarget::Grounded`]`)` edges resolved by the generic
/// `AtomResolver`. English is confined to THIS lens — `ground` itself is
/// source-agnostic, and `cites` / `defines` are other lenses of the same shape.
pub fn denotes_lens(
    english: &English,
) -> impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError> + '_ {
    // The written-form floor never fails closed: a word English does not know is a
    // legitimate no-op (the floor only grounds forms that exist), not an
    // authoring fault. So every node yields `Ok(..)` — but the lens is fallible in
    // shape so it composes with the generic (fallible) [`ground`].
    move |node| {
        Ok(node.lexical.as_deref().map_or_else(Vec::new, |text| {
            denotes_pointers(text, english)
                .into_iter()
                .map(|p| ("denotes".to_string(), p.target))
                .collect()
        }))
    }
}

/// The `Cites` relation-kind edge label — CiTO `cito:cites` (Peroni & Shotton
/// 2012), the SAME [`RelationsConcept::Cites`](crate::formal::relations::ontology::RelationsConcept::Cites)
/// already registered as the twelfth canonical relation kind. Capitalized (not
/// the lowercase `denotes`/`heading`/`citation` Lemon-role style): unlike those
/// lexicalization roles, `Cites` is a real relation kind meant to participate in
/// `relations_kind("Cites")`/`reaches()` queries over a materialized ontology,
/// the same way `PARTHOOD_REL` names the canonical `Parthood` kind.
pub const CITES_REL: &str = "Cites";

/// The `Cites` grounding pointers for ONE provision's own `refs` (a section's or
/// subdivision's [`UsCodeRef`] list, per
/// [`citation_index`](crate::social::software::markup::xml::uslm::corpus::bridge::citation_index)'s
/// per-node scoping) — the [`cites_lens`] producer, factored out so the lens
/// itself stays a thin per-node adapter.
///
/// Each `href` resolves one of three ways, checked in order:
/// 1. **Same-archive** — `href` names a node `own_names` already declares (a
///    citation to a section in the SAME loaded title/archive): `EdgeTarget::Local`,
///    cheap, no peer needed.
/// 2. **Cross-title** — `href`'s title number (via [`title_number_of_urn`])
///    names a SUPPLIED peer archive (keyed by the loaded-source naming
///    convention `praxis.toml`'s `[sources.usc_title_<N>]` entries already use:
///    `"usc_title_<N>"`) that declares a node named `href`: `EdgeTarget::Grounded`
///    at that node's real content address (computed from the actual peer
///    `Definition`, never guessed — a cited section's address depends on its own
///    heading/edges, unlike a `denotes` Form atom, which is why a real peer
///    archive is required in hand rather than just a name).
/// 3. **Unresolved** — no supplied peer covers that title, or the peer doesn't
///    declare that URN (a citation to a specific subsection that didn't survive
///    projection, a repealed provision, a genuinely stale href): **no edge**,
///    silently — the same "no coverage → no pointer" discipline
///    [`denotes_pointers`] established for an unknown word. This is a
///    deliberate CONTRAST with [`type_lens`](pr4xis_runtime::grounding::type_lens),
///    which fails closed on a declared-but-unrealizable target: a type mapping
///    is a functor's DECLARATION that must hold, while a citation routinely and
///    legitimately points outside whatever is currently loaded (most of the
///    real ~54 USC titles are never all loaded at once) — an unresolved
///    citation is not an authoring fault.
pub fn cites_pointers(
    refs: &[UsCodeRef],
    own_names: &BTreeSet<String>,
    peers: &BTreeMap<String, Archive>,
) -> Result<Vec<(String, EdgeTarget)>, LinkError> {
    let mut edges = Vec::new();
    for r in refs {
        if own_names.contains(&r.href) {
            edges.push((CITES_REL.to_string(), EdgeTarget::Local(r.href.clone())));
            continue;
        }
        let Some(title) = title_number_of_urn(&r.href) else {
            continue; // not a canonical USC URN — nothing to resolve against
        };
        let peer_name = format!("usc_title_{title}");
        let Some(peer) = peers.get(&peer_name) else {
            continue; // that title isn't among the supplied peers — no coverage
        };
        let Some(target) = peer.nodes.iter().find(|n| n.name == r.href) else {
            continue; // the peer is loaded but doesn't declare this exact URN
        };
        let atom = target.address().map_err(LinkError::Codec)?;
        edges.push((
            CITES_REL.to_string(),
            EdgeTarget::Grounded {
                ontology: peer_name,
                atom,
            },
        ));
    }
    Ok(edges)
}

/// The `cites` grounding LENS — the lens form of [`cites_pointers`], for the
/// generic [`ground`](pr4xis_runtime::grounding::ground). Closes over the
/// per-node citation side-table
/// ([`citation_index`](crate::social::software::markup::xml::uslm::corpus::bridge::citation_index),
/// since [`Definition`] carries no structured citation field itself — see that
/// function's doc), the grounding archive's own declared node names (for the
/// same-archive case), and whichever peer title archives the caller supplies
/// (for the cross-title case). A node with no entry in `refs_by_urn` is a
/// legitimate no-op (`Ok(vec![])`) — most provisions cite nothing.
pub fn cites_lens<'a>(
    refs_by_urn: &'a BTreeMap<String, Vec<UsCodeRef>>,
    own_names: &'a BTreeSet<String>,
    peers: &'a BTreeMap<String, Archive>,
) -> impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError> + 'a {
    move |node| {
        let Some(refs) = refs_by_urn.get(&node.name) else {
            return Ok(Vec::new());
        };
        cites_pointers(refs, own_names, peers)
    }
}

/// The `defines` relation-kind edge label — a Lemon-role-style lexicalization
/// pointer (lowercase, matching `denotes`), NOT a bibliographic relation kind
/// (unlike [`CITES_REL`], which reuses `RelationsConcept::Cites`): `defines`
/// names which written form a provision's OWN prose declares the meaning of,
/// the same lexicalization-pointer shape `denotes` already establishes.
pub const DEFINES_REL: &str = "defines";

/// Whether a recognized "the term 'X' V Y" declarative is EXHAUSTIVE
/// ("means") or PARTIAL/non-exhaustive ("includes") — Dickerson's
/// means/includes distinction in legislative drafting (Reed Dickerson
/// (1986), *The Fundamentals of Legal Drafting*, 2nd ed., Little, Brown &
/// Co., ISBN 0316183970 — the specific page was not independently
/// re-verified: every candidate mirror of the primary text returned HTTP
/// 403 to a live fetch on 2026-07-19; corroborated instead by the U.S.
/// House Office of the Legislative Counsel's own drafting guidance, "Quick
/// Guide to Legislative Drafting" and "Introduction to Legislative
/// Drafting", <https://legcounsel.house.gov/holc-guide-legislative-drafting>
/// — the SAME publisher-class citation
/// [`section_aux`](crate::social::software::markup::xml::uslm::corpus::section_aux)
/// already uses for USLM itself): "the term X means A, B, and C" asserts X
/// means ONLY A, B, and C; "the term X includes A" asserts A falls under X
/// WITHOUT asserting X means only A — an ILLUSTRATIVE, non-exhaustive
/// membership claim, not the term's whole meaning.
///
/// A rich enum (never a bare `is_partial: bool`) so a `defines` consumer can
/// tell the two claims apart without re-deriving the verb: an EXHAUSTIVE
/// pointer licenses "X means Y" paraphrase; a PARTIAL one licenses only "Y
/// is (an) X", never the converse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionExhaustiveness {
    /// "means" (or a VerbNet-confirmed Theme/Co-Theme synonym, e.g.
    /// "denote") — [`VerbNet::basic_transitive_theme_order`] confirms the
    /// Basic Transitive frame (Kipper et al. 2008); the definiens is
    /// asserted to be the term's WHOLE meaning.
    Exhaustive,
    /// "includes" — `is_partial_definition_verb`; the definiens is
    /// asserted to be A MEMBER of the term's meaning, not the whole of it.
    Partial,
}

/// One `defines` pointer: the definiendum `term` a provision's prose declares
/// via the "the term 'X' means Y" construction, and the [`Grounded`](EdgeTarget::Grounded)
/// edge into its `ontolex:Form` atom in `english_wordnet` — the SAME target
/// shape [`DenotesPointer`] resolves to (a written form, never a sense; sense
/// disambiguation is out of scope here exactly as it is for `denotes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinesPointer {
    /// The defined term (the close-apposition-promoted quoted definiendum).
    pub term: String,
    /// The grounded edge — `defines` into the term's Form atom by content
    /// address.
    pub target: EdgeTarget,
    /// Whether the declarative that produced this pointer was an
    /// exhaustive ("means") or partial ("includes") definition — see
    /// [`DefinitionExhaustiveness`]'s own doc for the citation.
    pub exhaustiveness: DefinitionExhaustiveness,
}

/// The chart-width bound [`defines_pointers`]'s OWN chart derivation runs
/// under — DOUBLE the shared production bound
/// ([`crate::cognitive::linguistics::lambek::reduce::chart_reduce_with_costs`]'s
/// own `MAX_CHART_WIDTH`, 256, which every PER-TURN chat parse still runs
/// under, UNCHANGED). The defines-lens gap backlog's G7 measured a REAL,
/// complete, single-declarative Title 42 definition — 42 U.S.C. §
/// 1395x(r) ("physician") — at 351 tokens, past the shared bound: the FIRST
/// failing stage there was `over-chart-width-cap`, not a grammar gap (the
/// FULL derivation the grammar already covers never got the chance to run).
/// Doubling the shared bound clears that real record with margin while
/// staying a genuine, finite resource bound, never "no limit" — a CYK chart
/// is O(n²) space / O(n³) time, so this only widens the ceiling.
///
/// Scoped, not global — the SAME [`SupertagCostTable::with_extra_unary`]
/// precedent [`definiens_cost_table`]'s own doc explains:
/// [`defines_pointers`]'s chart derivation is corpus-BUILD-time-only
/// (`crate::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology_with_defines`
/// / `compute_defines_overlay`'s own doc — this function is never called
/// from process startup, and the LIVE per-turn CLI/chat path resolves
/// through `usc_runtime_ontology_from_cached_defines`'s COMMITTED, cached
/// overlay, never this function at all), so the DoS-avoidance rationale the
/// shared bound exists for (`chart_reduce_with_costs`'s own doc:
/// "resource-exhaustion DoS on the user-facing chat path") genuinely does
/// not apply here — raising the SHARED constant instead would have widened
/// the ceiling for every live chat turn too, for a benefit only this
/// offline, batch corpus-grounding pass needs.
const DEFINES_MAX_CHART_WIDTH: usize = 512;

/// The `defines` pointers for a span of statute prose: recognizes the "the
/// term 'X' means Y" declarative shape by running the FULL
/// tokenize → chart-reduce → Montague-interpret pipeline over `text` (the
/// same pipeline shape `crates/chat/src/lib.rs::process_with_reasoner` runs,
/// minus the chat-specific multi-word-surface collapse and self-referential
/// stages, which have no bearing on definienda extraction) — never a
/// string/regex match.
///
/// A pointer is produced only when the FULL derivation:
/// 1. reduces to a complete [`Sem::Prop`] with exactly two arguments (a
///    two-place relation — subject and object, no more, no fewer);
/// 2. whose predicate, once lemmatized (the surface as parsed carries
///    whatever inflection the prose used — "means", not VerbNet's bare
///    member lemma "mean" — so every candidate lemma
///    [`lemmatizer::lemmatize`] offers, plus the surface itself, is tried),
///    is CONFIRMED (not assumed) by EITHER of two verb frames — the
///    resulting pointer's [`DefinesPointer::exhaustiveness`] records which:
///    [`VerbNet::basic_transitive_theme_order`] confirming a "mean"-class
///    verb whose Basic Transitive frame orders `[Theme, Co-Theme]` — Theme
///    (subject) is the definiendum, Co-Theme (object) the definiens
///    (Kipper et al. 2008) — is EXHAUSTIVE
///    ([`DefinitionExhaustiveness::Exhaustive`]); or
///    `is_partial_definition_verb` confirming "include" (Dickerson's
///    means/includes distinction — [`DefinitionExhaustiveness`]'s own doc
///    carries the full citation) is PARTIAL
///    ([`DefinitionExhaustiveness::Partial`]);
/// 3. whose SUBJECT (the LAST-absorbed argument — backward application
///    absorbs the subject after every object, the same absorption-order
///    convention [`montague`]'s own ditransitive-verb test documents) names
///    one or more definienda via `definiendum_words` — a SINGLE
///    close-apposition-promoted quoted definiendum
///    ([`Sem::Concept`], `montague`'s private `apply` function's NP-result
///    branch, close apposition guard), OR a COORDINATED SET of them
///    (defines-lens gap G5 — "the terms 'exploitation' and 'financial
///    exploitation' mean ...", 42 U.S.C. § 3002(18)(A); "the term 'X' and
///    the term 'Y' mean ...", 42 U.S.C. § 1395x(aa)(5)(A)): each conjunct
///    yields its OWN pointer, all sharing this ONE declarative's verb
///    confirmation and exhaustiveness; and
/// 4. each of whose definiendum words is resolved by `is_known_written_form`
///    (the defines-lens gap backlog's G7: EXACT ∪ lemmatized, then
///    case-folded — the SAME two-tier resolution
///    `crates/chat/src/lib.rs::resolve_surface` already applies to a
///    question surface, recovering a WordNet lemma the tokenizer's own
///    lowercasing hides, e.g. "Indian") — checked per-term, so one
///    coordinated conjunct's resolution never affects a sibling conjunct's.
///    A term NONE of those tiers know is a genuine statutory coinage (this
///    grammar only ever reaches this check for a term the prose ITSELF
///    names via "the term 'X' …" — never scraped from ordinary running
///    text), so it is MINTED into `mint_domain`'s statute-local namespace
///    ([`lemon::mint::mint`](crate::cognitive::linguistics::lemon::mint::mint))
///    rather than dropped — see `is_known_written_form`'s own doc and this
///    function's body for the full G7 mechanism.
///
/// Any sentence that does not reach exactly this shape (a parenthetical-
/// interrupted definition beyond G2's closed adjunct set, a
/// colon-introduced enumerated definiens beyond G3's reassembly, a
/// definiens shape beyond G4's coordination/PP-chain/bare-noun coverage —
/// none of which this grammar has further scaffolding for) simply fails
/// one of these checks and yields NO pointer — the same honest "no
/// coverage → no pointer" discipline
/// [`denotes_pointers`]/[`cites_pointers`] already establish.
pub fn defines_pointers(
    text: &str,
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    verbnet: &VerbNet,
    mint_domain: &OntologyName,
) -> Vec<DefinesPointer> {
    // Split FIRST, always — never assume the caller already handed a
    // single sentence. `reduce()` has no notion of "the next sentence" (it
    // combines only within the token array it is given), so a multi-
    // sentence blob ("when used in this subchapter... 'X' means ...; 'Y'
    // means ...") pays the FULL blob's combinatorial chart cost as one
    // whole-span parse attempt below — confirmed directly against real USC
    // Title 15 candidates during this investigation: a 23,342-character
    // preamble (`/us/usc/t15/s80a-2/a`) took ~88 CPU-MINUTES as one
    // whole-blob attempt and extracted ZERO definitions. Not a performance-
    // only concern: every real "'X' means Y" sentence inside such a blob is
    // silently lost, because the derivation below only ever succeeds when
    // the ENTIRE input reduces to one complete two-argument `Sem::Prop`
    // (this function's own doc, point 1) — a whole multi-sentence blob
    // essentially never does. A single already-one-sentence `text` splits
    // into exactly one piece here, so this is behaviorally a no-op for
    // every existing single-sentence caller/test.
    let vocab = crate::cognitive::linguistics::lambek::operators::vocabulary();
    let dashes = crate::cognitive::linguistics::symbols::dash_punctuation::vocabulary();
    // Built ONCE per call, not once per split sentence: `definiens_cost_
    // table()` clones-and-extends the shared base table
    // (`SupertagCostTable::with_extra_unary`, real allocation work, not
    // free), so hoisting it above the sentence loop below matters — a
    // candidate that splits into N sentences previously rebuilt this table
    // N times. Confirmed as a REAL, measured regression this session (a
    // corpus-wide before/after audit found 3 ordinary-length Title 15
    // candidates that completed in 4-5s before the sentence-splitting fix
    // now timing out after it, with no pathological length/complexity to
    // explain the slowdown — repeated table construction across the
    // now-multiple `defines_pointers_single_span` calls per candidate).
    let table = definiens_cost_table();
    tokenize::split_into_sentences(text, vocab, dashes, lang)
        .iter()
        .flat_map(|sentence| {
            defines_pointers_single_span(sentence, lang, reasoner, verbnet, mint_domain, &table)
        })
        .collect()
}

/// The single-sentence chart-parse [`defines_pointers`] now always splits
/// into — see its own doc for why splitting happens there, unconditionally,
/// rather than trusting each caller to pre-split. Behavior below is
/// otherwise UNCHANGED from before this split was introduced, other than
/// `table` now being a caller-supplied reference (built once per
/// `defines_pointers` call) rather than reconstructed on every span.
fn defines_pointers_single_span(
    text: &str,
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    verbnet: &VerbNet,
    mint_domain: &OntologyName,
    table: &SupertagCostTable,
) -> Vec<DefinesPointer> {
    // A pre-tokenization early-reject using raw whitespace word count as a
    // proxy for the eventual token count was tried and REVERTED here: the
    // corpus-wide ratchet
    // (`crates/praxis-corpus-tests/tests/defines_pointers_corpus_ratchet.rs`)
    // caught it as a real regression (Titles 15/42/49 dropped below their
    // committed floors), and a direct probe
    // (`crates/praxis-corpus-tests/tests/scratch_probe.rs`,
    // `probe_early_reject_false_positives_in_titles_15_42_49`) confirmed why:
    // 42 U.S.C. § 1585(a) is 548 raw words (over `DEFINES_MAX_CHART_WIDTH`)
    // but tokenizes to only 454 real tokens (`collapse_capitalized_runs`
    // merging proper-noun runs) — comfortably under the bound, and a real
    // definition the proxy would have silently dropped. The genuinely
    // pathological case this was chasing (42 U.S.C. § 1395l(a)(2), a
    // 2,180-raw-word exception clause with no internal sentence boundary
    // for `split_into_sentences` to exploit) tokenizes in ~8.2s in a release
    // build with the spelling-correction memoization and alphanumeric-mix
    // guard above already in place — bounded and acceptable for this
    // offline, corpus-BUILD-time-only pipeline (this function's own doc)
    // without an unsafe proxy in front of it.
    let (tokens, mut alternatives) = tokenize::tokenize_with_alternatives(text, lang);
    if tokens.is_empty() {
        return Vec::new();
    }
    // The SCOPED participial reading (see `participle_alternatives`): purely
    // ADDITIVE, so every derivation that already succeeded still does — the
    // extra category can only ever open a reading, never close one.
    // `tokenize_with_alternatives` returns one alternatives row per token, so
    // the zip below is total.
    for (row, extra) in alternatives
        .iter_mut()
        .zip(participle_alternatives(&tokens, lang))
    {
        for t in extra {
            if !row.contains(&t) {
                row.push(t);
            }
        }
    }

    // The PARTIAL-PARSE goal (`clause_fragments_with_alternatives_and_table_
    // and_width`), not the whole-string one. A definitional provision is
    // running statutory text, not a curated single sentence: it routinely
    // carries material that is not part of the definitional predication at
    // all — a fronted participial preamble ("As used in this section, the
    // term “X” means …"), a trailing infinitival purpose clause ("… means any
    // person authorized by law TO PERFORM THE DUTIES THEREOF"), an
    // enumeration the dash-reassembly index re-joined. Under a whole-string
    // goal ONE such unattachable adjunct empties `chart[0][n]` and destroys a
    // derivation the chart in fact completed over the definitional clause
    // itself, discarding a definiendum it had already fully analysed. That is
    // Abney's attachment/constituency separation ("Parsing By Chunks", 1991,
    // §1–§3; the longest-match selection rule, "Partial Parsing via
    // Finite-State Cascades", 1996, §1) — see
    // `clause_fragments_with_costs_bounded`'s own doc for the full argument
    // and for why reading these sub-span goals costs no extra parse (Sheil
    // 1976: the CYK chart is a well-formed substring table; the whole-string
    // goal built these cells and threw them away).
    //
    // NOTHING about WHICH readings count as definitional is relaxed here: the
    // per-clause checks below (a two-argument `Sem::Prop`, a VerbNet-confirmed
    // definitional predicate, a MENTIONED subject — `definiendum_words`) are
    // the same ones the whole-string path applied, and they are what keeps a
    // clause of ordinary prose from minting a `defines` edge. Only the
    // requirement that the definitional clause be the ENTIRE span is dropped.
    //
    // When the whole string DOES derive an S, the cover is the single
    // fragment `[0, n)` with the same type and the same per-token assignment
    // the whole-string goal produced, so every span that extracted before
    // extracts identically.
    let clauses = clause_fragments_with_alternatives_and_table_and_width(
        &tokens,
        &alternatives,
        table,
        DEFINES_MAX_CHART_WIDTH,
    );
    // The chart-failure fallback the whole-string path already had: when the
    // syntax chart derives no S at all, the interpreter still ran over the
    // raw (default-typed) tokens, because its OWN goal falls back to any
    // full-span derivation (`interpret_with_unary_rules`'s own doc). Kept as
    // an ADDITIONAL candidate rather than replaced, so no span that extracted
    // through that route can lose its pointer; skipped when a clause already
    // covers the whole string, where it would be the identical parse.
    let whole_span_already_covered = clauses
        .first()
        .is_some_and(|clause| clause.span.is_whole(tokens.len()));
    let candidates = clauses
        .iter()
        .map(|clause| clause.tokens.as_slice())
        .chain((!whole_span_already_covered).then_some(tokens.as_slice()));

    let mut pointers: Vec<DefinesPointer> = Vec::new();
    for candidate in candidates {
        for pointer in definitional_pointers(candidate, text, lang, reasoner, verbnet, mint_domain)
        {
            // Two clauses of one provision may define the SAME term (a
            // re-statement, or an enumeration the reassembly index joined);
            // the pointer is the same claim either way.
            if !pointers.contains(&pointer) {
                pointers.push(pointer);
            }
        }
    }
    pointers
}

/// The definitional reading of ONE already-typed clause: interpret it, and if
/// it is a definitional predication over a MENTIONED subject, the `defines`
/// pointer(s) it declares.
///
/// This is [`defines_pointers_single_span`]'s per-clause body, factored out
/// UNCHANGED when that function moved from the whole-string parse goal to the
/// partial-parse one — every check below (two-argument `Sem::Prop`,
/// VerbNet-confirmed predicate, `definiendum_words`' mention gate, G7 minting)
/// is the one the whole-string path applied to the whole string.
///
/// `provision_text` is the WHOLE provision's raw prose, not the clause's — it
/// is used only as the gloss of a G7-minted coinage, and minting is
/// content-addressed on `(domain, word)` alone ([`mint`]'s own doc), so the
/// gloss never changes a target address.
fn definitional_pointers(
    clause: &[TypedToken],
    provision_text: &str,
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    verbnet: &VerbNet,
    mint_domain: &OntologyName,
) -> Vec<DefinesPointer> {
    // The SEMANTIC side of the SAME two scoped type-changing rules the
    // syntax chart ran under (`definiens_cost_table`) — the pair must match,
    // or a syntactically-derived reading has no meaning to compose.
    //
    // Read over EVERY sub-span of the clause, not only the clause as a whole
    // (`interpret_maximal_spans_where`, longest first). The syntactic cover
    // above already dropped the requirement that the definitional clause BE
    // the whole provision; this drops the remaining half of the same
    // requirement, on the semantic side: an adjunct that composes at the TOP
    // of a clause (`S + S\S → S` — a trailing infinitival purpose clause,
    // "… means any person authorized by law TO PERFORM THE DUTIES THEREOF")
    // leaves the clause-level cell holding the ADJUNCT's meaning, so the
    // definitional `Sem::Prop` is complete but one cell down and the
    // whole-clause goal never looks at it. Longest-first with disjointness
    // means the whole clause is ALWAYS tested first, so a clause that
    // extracted before extracts identically and no sub-span reading can
    // displace it.
    montague::interpret_maximal_spans_where(
        clause,
        reasoner,
        &[
            (LambekType::n(), LambekType::np()),
            (
                svo::passive_participle_verb(),
                svo::reduced_relative_postmodifier(),
            ),
        ],
        &mut |sem| definitional_reading(sem, verbnet).is_some(),
    )
    .into_iter()
    .flat_map(|(_span, meaning)| {
        let Some((exhaustiveness, terms)) = definitional_reading(&meaning, verbnet) else {
            return Vec::new();
        };
        definiendum_targets(terms, exhaustiveness, provision_text, lang, mint_domain)
    })
    .collect()
}

/// Is `meaning` a DEFINITIONAL predication — and if so, exhaustive or partial
/// ([`DefinitionExhaustiveness`]), and over which definiendum word(s)?
///
/// The three checks are exactly the ones [`defines_pointers_single_span`]
/// applied to its whole-string reading before this file moved to the
/// partial-parse goal, in the same order they mattered, and nothing has been
/// weakened: a two-argument `Sem::Prop`, a subject that MENTIONS its term
/// ([`definiendum_words`]), and a predicate VerbNet confirms is a basic
/// transitive `[Theme, Co-Theme]` verb (or the closed-class partial-definition
/// verb "include"). The cheap, highly selective checks run first because this
/// is now called once per chart cell rather than once per span.
fn definitional_reading(
    meaning: &Sem,
    verbnet: &VerbNet,
) -> Option<(DefinitionExhaustiveness, Vec<String>)> {
    let Sem::Prop {
        predicate,
        arguments,
    } = meaning
    else {
        return None;
    };
    if arguments.len() != 2 {
        return None;
    }

    // The subject is the LAST-absorbed argument (backward application
    // absorbs the subject after every object) — see `defines_pointers`'s own
    // doc for the full absorption-order citation. `definiendum_words` handles
    // both the single-term and the coordinated-set (G5) shape.
    let terms = definiendum_words(arguments.last()?);
    if terms.is_empty() {
        return None;
    }

    // The predicate as parsed carries whatever inflection the prose used
    // ("means"); VerbNet's member lemma is bare ("mean"). Try the surface
    // itself first, then every lemmatized candidate — the same dual-route
    // (identity ∪ de-inflection) `crates/chat/src/lib.rs::resolve_surface`
    // already applies to a question surface.
    let mut lemma_candidates = alloc::vec![predicate.clone()];
    for form in lemmatizer::lemmatize(predicate, MorphLanguage::English) {
        if !lemma_candidates.contains(&form.written_rep) {
            lemma_candidates.push(form.written_rep);
        }
    }
    let exhaustiveness = if lemma_candidates
        .iter()
        .any(|lemma| verbnet.basic_transitive_theme_order(lemma).is_some())
    {
        DefinitionExhaustiveness::Exhaustive
    } else if lemma_candidates
        .iter()
        .any(|lemma| is_partial_definition_verb(lemma))
    {
        DefinitionExhaustiveness::Partial
    } else {
        return None;
    };
    Some((exhaustiveness, terms))
}

/// The grounded `defines` edge each definiendum word resolves to — the
/// EXACT G7 resolution [`defines_pointers_single_span`] carried inline before
/// the partial-parse goal split it out: a known English written form grounds
/// into `english_wordnet`'s Form atom, and anything else is a genuine
/// statutory coinage minted into `mint_domain`.
fn definiendum_targets(
    terms: Vec<String>,
    exhaustiveness: DefinitionExhaustiveness,
    provision_text: &str,
    lang: &English,
    mint_domain: &OntologyName,
) -> Vec<DefinesPointer> {
    terms
        .into_iter()
        .filter_map(|word| {
            if is_known_written_form(lang, &word) {
                let atom = form_atom(&word).address().ok()?;
                return Some(DefinesPointer {
                    term: word,
                    target: EdgeTarget::Grounded {
                        ontology: ENGLISH_ONTOLOGY.to_string(),
                        atom,
                    },
                    exhaustiveness,
                });
            }
            // G7: a genuine out-of-lexicon statutory coinage — reaching this
            // point already means the "the term 'X' means/includes Y"
            // declarative ITSELF names `word` as the term it is coining (see
            // this function's own G7 doc), never a candidate scraped from
            // ordinary prose. Mint it into `mint_domain`'s statute-local
            // namespace (`lemon::mint::mint`) instead of dropping the
            // pointer. The Lexicon is a throwaway, one-shot instance —
            // minting is deterministic BY CONTENT ADDRESS on `(domain,
            // word)` alone (`mint`'s own doc), so a caller who later mints
            // the SAME pair again (e.g. composing a real, resolving
            // `ComposedReasoner`) re-derives the identical reference; no
            // state needs to persist across calls for THIS pointer's target
            // to be correct. The raw declarative sentence itself becomes the
            // minted concept's gloss — the same "raw prose as gloss"
            // convention `ConceptView::Loaded` already reads off a loaded
            // node's own `lexical`.
            let mut throwaway_lexicon = Lexicon::new("en");
            let (minted, _onto) = mint(
                &mut throwaway_lexicon,
                mint_domain.clone(),
                &word,
                Some(provision_text),
            )
            .ok()?;
            Some(DefinesPointer {
                term: word,
                target: EdgeTarget::Grounded {
                    ontology: mint_domain.as_str().to_string(),
                    atom: minted.form_address,
                },
                exhaustiveness,
            })
        })
        .collect()
}

/// Is `word` a KNOWN English written form — the SAME two-tier resolution
/// `crates/chat/src/lib.rs::resolve_surface` already applies to a live
/// question surface (that function's own doc; Slice D,
/// `.notes/chat-fix-c-build-state.md`): EXACT case ∪ every DE-INFLECTED
/// candidate ([`lemmatizer::lemmatize`]) tried FIRST; only when that whole
/// tier is empty, the CASE-FOLDED fallback
/// ([`English::lookup_case_folded`], the loaded Unicode simple case-folding
/// table — never `str::to_lowercase`) over the SAME raw-∪-lemmatized
/// candidate set, recovering a capitalized WordNet lemma the tokenizer's own
/// lowercasing hides ("Indian" — the defines-lens gap backlog's G7).
///
/// [`denotes_pointers`]'s own written-form floor, and [`renvoi_pointers`]'s
/// (the SAME `lang.lookup` check, over a RENVOI definiendum span), are
/// deliberately UNCHANGED: G7 (this gap) is scoped to the shape its own
/// report measured — a `defines_pointers` derivation that reaches a full
/// `Sem::Prop` with a VerbNet-confirmed verb before the floor rejects it.
/// `renvoi_pointers`'s own documented G7 baselines ("Indian tribe", "United
/// States", "skilled nursing facility") are unresolvable multi-word
/// COMPOUNDS no case-fold of the whole span recovers (case-folding "indian
/// tribe" is still not a WordNet compound headword), so applying this same
/// tier there would not change either of that function's own honest-
/// baseline test outcomes.
fn is_known_written_form(lang: &English, word: &str) -> bool {
    if !lang.lookup(word).is_empty() {
        return true;
    }
    let forms = lemmatizer::lemmatize(word, MorphLanguage::English);
    if forms
        .iter()
        .any(|form| form.written_rep != word && !lang.lookup(&form.written_rep).is_empty())
    {
        return true;
    }
    if !lang.lookup_case_folded(word).is_empty() {
        return true;
    }
    forms
        .iter()
        .any(|form| !lang.lookup_case_folded(&form.written_rep).is_empty())
}

/// The definiendum WORD(s) a "means"/"includes" declarative's SUBJECT
/// argument names.
///
/// # A definiendum is MENTIONED, never used
///
/// The one property every definiendum has and no ordinary subject has is
/// that the provision talks ABOUT the expression rather than with it —
/// Quine's use/mention distinction
/// ([`ExpressionUse`](crate::cognitive::linguistics::lambek::reduce::ExpressionUse)
/// carries the full citation). `The term “vessel” includes every description
/// of watercraft` MENTIONS "vessel"; `Any benefit provided under subsection
/// (c) may … be provided to a family member` (5 U.S.C. § 5569(g)) USES every
/// word in it and defines nothing. Both reduce to a two-argument
/// `Sem::Prop` whose subject is a `Sem::Concept`, and both carry a predicate
/// VerbNet confirms — so NOTHING in the derivation's shape separates them.
/// Only the mention does, and this function is where that check belongs.
///
/// This was a MEASURED defect, not a hypothetical one: without the check,
/// title-1's committed overlay carried `(/us/usc/t1/s1, "words")` off the
/// section HEADING "Words denoting number, gender, and so forth" (a real
/// `[Theme, Co-Theme]` derivation over "denote", subject "words" — used),
/// and the reduced-relative grammar coverage added for the HCBS definitions
/// turned every `NOUN provided/described under …` clause in the corpus into
/// a `defines` edge on the participle.
///
/// The marking is the TOKENIZER's, carried through the chart and the
/// interpreter (`lex`'s mention branch, `montague::apply`'s
/// close-apposition-with-a-mentioned-head branch) — never re-derived here by
/// looking for quote glyphs in the source text, which by this point no
/// longer exist.
///
/// # The two accepted shapes
///
/// A single mentioned definiendum ([`Sem::Concept`] carrying
/// [`ExpressionUse::Mentioned`] — either the bare-`NP` mention subject of
/// `“radiation” means …` (42 U.S.C. § 10003(1)) or the close-apposition
/// promotion of `the term “X”`), OR a COORDINATED SET of them
/// ([`Sem::Func`] under [`tokenize::apposition_coordinator_canonical`]'s
/// reserved marker — "the terms 'X' and 'Y' mean ...", the defines-lens gap
/// backlog's G5, attested at 42 U.S.C. § 3002(18)(A) and § 1395x(aa)(5)(A)).
/// Anything else names no definiendum at all (an empty result).
fn definiendum_words(subject: &Sem) -> Vec<String> {
    match subject {
        Sem::Concept {
            word,
            expression_use: ExpressionUse::Mentioned,
            ..
        } => alloc::vec![word.clone()],
        // Deliberately [`tokenize::apposition_coordinator_canonical`]
        // ONLY, never the literal-surface
        // [`tokenize::nominal_coordinator_canonical`]: an ORDINARY
        // coordinated NP subject ("the term 'consumer', the county, and
        // the state cover this benefit.") reduces to the IDENTICAL
        // `Sem::Func{"and", [Concept, Concept, Concept]}` shape — a REAL,
        // measured regression
        // (`a_coordinated_subject_is_not_mistaken_for_a_medial_supplement`)
        // this exact guard fixes. The reserved apposition-coordinator
        // marker is unambiguous BY CONSTRUCTION: the tokenizer only ever
        // mints it where BOTH conjuncts were genuinely quoted spans
        // (`tokenize::mark_apposition_coordinators`'s own doc). Each
        // conjunct is ALSO required to be mentioned, exactly as the single
        // definiendum above is — the marker vouches for the coordinator,
        // the per-conjunct check vouches for each conjunct, and neither
        // stands in for the other.
        Sem::Func { word, body } if tokenize::apposition_coordinator_canonical(word).is_some() => {
            body.iter()
                .filter_map(|item| match item {
                    Sem::Concept {
                        word,
                        expression_use: ExpressionUse::Mentioned,
                        ..
                    } => Some(word.clone()),
                    _ => None,
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Is `lemma` a LEGISLATIVE-DRAFTING partial-definition verb — "include"?
/// Dickerson's means/includes distinction ([`DefinitionExhaustiveness`]'s
/// own doc carries the full citation) is a REGISTER-SPECIFIC convention of
/// statutory drafting, not a general-English verb-semantic fact: checked
/// against the loaded VerbNet 3.3 class data, "include" is a member of
/// exactly two classes (`admit-64.3-1` and `involve-107.1`), and NEITHER
/// carries a Basic Transitive frame with the `[Theme, Co-Theme]` NP-role
/// order [`VerbNet::basic_transitive_theme_order`] requires — both use
/// Agent/Theme/Location or Agent/Theme/Goal roles instead (ordinary
/// "the box includes a manual" is an Agent-Theme containment predicate, not
/// a definitional copula). So this is a hand-authored closed-class check,
/// not a VerbNet query — the SAME rationale
/// [`crate::cognitive::linguistics::lambek::tokenize`]'s
/// `is_nominal_coordinator`/`is_do_support`/`is_modal_auxiliary` already
/// establish for a closed grammatical (here: legal-drafting-register)
/// class no loaded vocabulary correctly isolates.
fn is_partial_definition_verb(lemma: &str) -> bool {
    lemma == "include"
}

/// The chart cost table [`defines_pointers`] runs ITS OWN chart derivation
/// against — the shared production table
/// ([`crate::cognitive::linguistics::lambek::supertag_costs::supertag_cost_table`])
/// PLUS two scoped extras, both real published CCGbank type-changing rules:
///
/// 1. [`bare_noun_phrase_unary_rule`] (`N → NP`; defines-lens gap
///    G4(f) — "the term 'elder abuse' means abuse of an older individual," 42
///    U.S.C. § 3002(15): a bare, determiner-less common-noun definiens is a
///    routine statutory-drafting pattern — Dickerson 1986, this module's own
///    `DefinitionExhaustiveness` doc carries the full citation).
/// 2. [`reduced_passive_relative_unary_rule`] (`NP\S[pss] → NP\NP`, CCGbank
///    Manual §3.8 (54)a "workers exposed to it"; defines-lens gap G4(d), the
///    participial postmodifier that doc previously recorded as UNSCAFFOLDED).
///    The definiens of 42 U.S.C. § 1396b(l)(5)(B)/(C) — "services described
///    in section 1396d(a)(7) of this title PROVIDED under a State plan…" /
///    "personal care services PROVIDED under a State plan…" — is exactly
///    this shape, and it is the whole reason those two HCBS definitions
///    extract nothing today. See [`participle_alternatives`] for the
///    lexical half.
///
/// NEVER the shared table itself: a corpus-gate measurement already
/// rejected the bare-NP rule at the GLOBAL level (`define −6`,
/// 2026-07-10 — `supertag_costs`'s own
/// `tests::the_shipped_table_carries_no_unary_rows` test doc), and the
/// participial rule is held to the same discipline: a global row would
/// change every live chat derivation and can only be admitted by that same
/// corpus-gate measurement, not by assertion.
/// [`SupertagCostTable::with_extra_unary`] composes a NEW table
/// (cloning the shared table's lexical rows, never mutating it) so this
/// scoping is structural, not just documented discipline: the live chat
/// pipeline's own calls to
/// [`crate::cognitive::linguistics::lambek::reduce::reduce_with_alternatives`]
/// never see this rule at all.
///
/// Built fresh per call (no process-level cache): `defines_pointers` already
/// re-tokenizes from scratch every call, and cloning the shared table's
/// small (~13-row) lexical map costs far less than the WordNet lookups the
/// SAME call already performs.
fn definiens_cost_table() -> SupertagCostTable {
    #[cfg(feature = "std")]
    let base = crate::cognitive::linguistics::lambek::supertag_costs::supertag_cost_table();
    #[cfg(not(feature = "std"))]
    let base = &crate::cognitive::linguistics::lambek::supertag_costs::build_table();
    base.with_extra_unary(alloc::vec![
        bare_noun_phrase_unary_rule(),
        reduced_passive_relative_unary_rule(),
    ])
}

/// The OLiA form-level class an `-ed`/`-en` surface carries when the loaded
/// morphology recognizes it as a PAST/PASSIVE PARTICIPLE of its lemma —
/// minted by `English::lexical_lookup_all`'s participle block through
/// [`crate::cognitive::linguistics::lexicon::olia::form_level_class`], the
/// SAME route the gerund-participial `ing` mark already takes.
fn past_participle_form_class() -> Option<&'static str> {
    crate::cognitive::linguistics::lexicon::olia::form_level_class(
        crate::cognitive::linguistics::morphology::SemanticEffect::PastParticiple,
    )
}

/// The SCOPED lexical half of the reduced-relative analysis: the extra
/// chart reading a token gets when the LOADED morphology has marked it a
/// past participle — CCGbank's `S[pss]\NP`
/// ([`svo::passive_participle_verb`]), which
/// [`reduced_passive_relative_unary_rule`] then type-changes to the `NP\NP`
/// postmodifier inside [`definiens_cost_table`]'s chart.
///
/// Returns the per-token alternatives to ADD to
/// [`tokenize::tokenize_with_alternatives`]'s own output — additive only,
/// never replacing a reading, exactly as the tokenizer's own OLiA-class
/// projection is additive.
///
/// # Why here and not in the loaded OLiA→CCG functor
///
/// A `PastParticiple → S[pss]\NP` row in `data/grammar/olia-ccg-categories.tsv`
/// would give the category to EVERY caller, including every live chat turn,
/// and a global grammar row is admissible in this repo only after the
/// chat corpus gate measures it — the exact discipline
/// `bare_noun_phrase_unary_rule` was subjected to (three measured attempts,
/// all REJECTED at the global level, `ccg-supertag-costs.tsv`'s own
/// commentary) before being scoped to this lens instead. So the binding is
/// scoped HERE, beside the type-changing rule it exists to feed, and the
/// two halves are inseparable: neither does anything without the other.
///
/// The class identity itself is NOT hardcoded here — it is the loaded OLiA
/// Reference-Model class the morphology attached
/// ([`past_participle_form_class`]); this function only decides WHICH
/// scoped category that class projects to for this lens.
fn participle_alternatives(tokens: &[TypedToken], lang: &English) -> Vec<Vec<LambekType>> {
    use crate::cognitive::linguistics::language::Language;
    let Some(class) = past_participle_form_class() else {
        return alloc::vec![Vec::new(); tokens.len()];
    };
    let participle = svo::passive_participle_verb();
    tokens
        .iter()
        .map(|t| {
            let marked = lang
                .lexical_lookup_all(&t.word.to_lowercase())
                .iter()
                .any(|e| e.olia_class() == Some(class));
            if marked && t.lambek_type != participle {
                alloc::vec![participle.clone()]
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// The `defines` grounding LENS — the lens form of [`defines_pointers`], for
/// the generic [`ground`](pr4xis_runtime::grounding::ground). Mirrors
/// [`denotes_lens`]'s exact shape: every node yields `Ok(..)` (a sentence
/// that doesn't parse to the recognized shape is a legitimate no-op, never an
/// authoring fault), so the lens composes with the generic (fallible)
/// `ground` even though `defines_pointers` itself never fails closed.
///
/// `shadowed_prose` is the S1+L1 fix (the defines-lens gap backlog's
/// heading-shadowing findings): a node's `lexical` is
/// heading-first (`uslm::corpus::bridge::project_archive`'s
/// `heading.or(chapeau).or(content)`), so a subdivision that carries a
/// `<heading>` never offers its real definitional prose to the check below;
/// and at a SECTION root `lexical` is heading-ONLY (the same projector's
/// unconditional `Some(section.heading.clone())`), so a subdivision-less
/// section — USLM's ordinary shape for a short prose-only section — never
/// offers its operative text at all.
/// `shadowed_prose` (keyed by the node's own name/URN,
/// [`defines_prose_index`](crate::social::software::markup::xml::uslm::corpus::bridge::defines_prose_index)'s
/// output) is the side-channel that recovers both, the SAME shape [`cites_lens`]
/// already takes `refs_by_urn` as a side parameter instead of a bespoke
/// `Definition` field. It never keys a section that HAS subdivisions: that
/// section's `text` is a mereological closure over its parts, already scanned
/// under their own URNs, and keying it would attribute a part's definition to
/// its ancestor (see `defines_prose_index`'s own doc).
///
/// `reassembled` is the G3+S2 fix (the gap backlog's dangling-chapeau
/// finding): a node whose OWN text ends mid-sentence (an em dash or colon —
/// the definiens continues in enumerated `children`, or, for a genuinely
/// SPLIT definition like 42 U.S.C. § 1396n(c)(5) "habilitation services",
/// on a SIBLING child entirely) never carries a parseable declarative by
/// itself — [`dangling_chapeau_reassembly_index`](crate::social::software::markup::xml::uslm::corpus::bridge::dangling_chapeau_reassembly_index)'s
/// output supplies, per dangling URN, every virtual-declarative-sentence
/// candidate that node's chapeau plus its children can form; EACH candidate
/// is run through the FULL [`defines_pointers`] pipeline exactly like
/// `lexical`/`shadowed_prose`, so a still-uncovered grammar shape (a
/// coordinated relative-clause list, a fronted scoping adjunct) simply
/// yields no pointer for that candidate — the honest "no coverage → no
/// pointer" outcome, never a guess.
///
/// Pointers found via `lexical`, `shadowed_prose`, and EVERY `reassembled`
/// candidate are UNIONED (deduped by term): a node absent from a table (the
/// overwhelming majority) costs one `BTreeMap` probe per table and changes
/// nothing.
///
/// `mint_domain` is the G7 fix's statute-local minting namespace — see
/// [`defines_pointers`]'s own doc (point 4) and `is_known_written_form`'s
/// doc for the full mechanism; passed straight through to every
/// `defines_pointers` call this lens makes.
pub fn defines_lens<'a>(
    lang: &'a English,
    reasoner: &'a dyn LexicalReasoner,
    verbnet: &'a VerbNet,
    shadowed_prose: &'a BTreeMap<String, String>,
    reassembled: &'a BTreeMap<String, Vec<String>>,
    mint_domain: &'a OntologyName,
) -> impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError> + 'a {
    move |node| {
        let mut pointers = node
            .lexical
            .as_deref()
            .map(|text| defines_pointers(text, lang, reasoner, verbnet, mint_domain))
            .unwrap_or_default();
        let extend_from = |candidate: &str, pointers: &mut Vec<DefinesPointer>| {
            for extra in defines_pointers(candidate, lang, reasoner, verbnet, mint_domain) {
                if !pointers
                    .iter()
                    .any(|p: &DefinesPointer| p.term == extra.term)
                {
                    pointers.push(extra);
                }
            }
        };
        if let Some(prose) = shadowed_prose.get(&node.name) {
            extend_from(prose, &mut pointers);
        }
        if let Some(candidates) = reassembled.get(&node.name) {
            for candidate in candidates {
                extend_from(candidate, &mut pointers);
            }
        }
        Ok(pointers
            .into_iter()
            .map(|p| (DEFINES_REL.to_string(), p.target))
            .collect())
    }
}

// ---- G6: renvoi (definition-by-reference) ----

/// The `definesByReference` relation-kind edge label — the RENVOI
/// counterpart of [`DEFINES_REL`]: a provision's prose declares a
/// definiendum but DEFERS its meaning to another cited provision (the
/// defines-lens gap backlog's G6), rather than stating the definiens
/// itself. Lemon-role-style lowercase (matching `defines`/`denotes`), not
/// a [`RelationsConcept`](crate::formal::relations::ontology::RelationsConcept)
/// bibliographic kind (unlike [`CITES_REL`]): the edge still POINTS at a
/// provision (the SAME `Local`/`Grounded` shape [`cites_pointers`] already
/// produces), but its ROLE is lexicalization-by-reference, not a
/// bibliographic citation — kept a DISTINCT label from both `defines`
/// (whose target is always a Form, never a provision) and `cites` (which
/// asserts "this provision cites that one," not "this provision's
/// definiendum's meaning lives there").
pub const DEFINES_BY_REFERENCE_REL: &str = "definesByReference";

/// One `definesByReference` (renvoi) pointer: the definiendum `term` a
/// provision's prose declares via the "the term 'X' has the meaning given
/// ... in/by \[CITATION\]" construction (Dickerson's renvoi/cross-reference
/// convention in legislative drafting — a term is DEFERRED to another
/// provision's own definition, never restated locally —
/// [`DefinitionExhaustiveness`]'s own doc has the full drafting-guidance
/// citation), and the TARGET provision that carries the real definition —
/// the SAME [`EdgeTarget`] shape [`cites_pointers`] resolves a citation to
/// (`Local`/`Grounded` at a Section/subdivision's own content address),
/// NEVER a Form atom: unlike [`DefinesPointer`] (whose target IS the
/// term's own meaning, an English Form), a renvoi pointer's target is
/// WHERE the meaning lives, not the meaning itself — resolving it
/// recursively (following THAT provision's own `defines` edge) is a
/// downstream consumer's job, not this lens's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinesByReferencePointer {
    /// The defined term (the same close-apposition-quote-detected
    /// definiendum [`defines_pointers`] extracts, possibly coordinated —
    /// see [`renvoi_pointers`]'s own doc).
    pub term: String,
    /// The grounded (or `Local`) edge to the PROVISION the definition is
    /// deferred to.
    pub target: EdgeTarget,
}

/// Locate the START (the "has"/"have" token index) of a renvoi
/// (definition-by-reference) predicate — "has"/"have" (+ "the" (+
/// "same")) "meaning" "given" — in `tokens`, or `None` if the sentence
/// carries no such predicate. A hand-authored closed-class check over the
/// TOKEN sequence (never a substring/regex match on raw text) — the SAME
/// rationale [`is_partial_definition_verb`] gives for its own closed
/// "includes" class: this is a REGISTER-SPECIFIC legislative-drafting
/// formula, not a general-English verb-semantic fact VerbNet could
/// confirm. Every real byte-verified variant this module's test suite
/// carries matches: "has the meaning given such term in ..." (42 U.S.C. §
/// 1395x(j)), "has the same meaning given the term in ..." (42 U.S.C. §
/// 3002(19)), "have the meaning given to them by ..." (42 U.S.C. §
/// 1395x(x)).
///
/// Deliberately stops at "given" — the REFERENT/citation phrase that
/// follows ("such term in section 1395i–3(a) of this title" / "to them by
/// subsections (h) and (i) ... of section 410") is never parsed: renvoi
/// target resolution comes entirely from the node's own `<ref>` citations
/// via [`cites_pointers`], never a re-derived reading of the citation
/// PROSE.
fn renvoi_predicate_start(tokens: &[TypedToken]) -> Option<usize> {
    for (i, tok) in tokens.iter().enumerate() {
        if tok.word != "meaning" {
            continue;
        }
        if tokens.get(i + 1).map(|t| t.word.as_str()) != Some("given") {
            continue;
        }
        let mut j = i;
        let mut steps = 0;
        while j > 0 && steps < 3 {
            j -= 1;
            steps += 1;
            match tokens[j].word.as_str() {
                "has" | "have" => return Some(j),
                "the" | "same" => continue,
                _ => break,
            }
        }
    }
    None
}

/// The `definesByReference` pointers for a span of statute prose — the
/// defines-lens gap backlog's G6. A GENUINELY DIFFERENT frame from
/// [`defines_pointers`]'s "means"/"includes" declarative: no definiens is
/// stated locally at all, so this deliberately does NOT run the
/// means-pipeline's chart-reduce/Montague derivation (that grammar
/// correctly finds no complete `S` here — "has"/"have" + "meaning" +
/// "given" is not a `mean`-class VerbNet frame).
///
/// 1. `renvoi_predicate_start` locates where the renvoi predicate
///    begins;
/// 2. every token STRICTLY BEFORE that position that the tokenizer itself
///    offered the [`close_apposition`](svo::close_apposition) alternative
///    for (a quote-collapsed span, `tokenize::collapse_quoted_spans`) is a
///    definiendum candidate — the SAME quote-detection primitive
///    [`defines_pointers`]'s own close-apposition promotion relies on,
///    read directly rather than re-derived through a chart/Montague pass:
///    there is no verb in this prefix for a "means"-shaped derivation to
///    reduce against, so recognizing the definiendum span needs only the
///    tokenizer's own quote-detection, not sentential composition. A
///    coordinated PAIR (defines-lens gap G5 — "the terms 'State' and
///    'United States' have the meaning given to them by ...", 42 U.S.C. §
///    1395x(x)) is handled the SAME way `defines_pointers` handles G5:
///    every quote-collapsed span in the prefix is its own candidate,
///    regardless of how many;
/// 3. each candidate that survives the SAME written-form floor
///    [`denotes_pointers`]/[`defines_pointers`] already apply
///    (`lang.lookup`) becomes one pointer, ALL sharing the SAME resolved
///    target(s) — the node's own citation `refs`, resolved via
///    [`cites_pointers`] VERBATIM (the SAME `Local`/cross-title-`Grounded`/
///    unresolved discipline `cites_lens` already establishes: an
///    unresolved renvoi target, or a node with no `<ref>` at all, is left
///    ungrounded, never guessed). When more than one definiendum shares a
///    single cited target (the "State"/"United States" case above), EACH
///    gets its own pointer at that SAME target — an honest, section-level
///    edge; this does NOT attempt to resolve the "...by subsections (h)
///    and (i), respectively..." distributive split to a PER-TERM
///    subsection, a separate, unaddressed capability.
pub fn renvoi_pointers(
    text: &str,
    lang: &English,
    refs: &[UsCodeRef],
    own_names: &BTreeSet<String>,
    peers: &BTreeMap<String, Archive>,
) -> Result<Vec<DefinesByReferencePointer>, LinkError> {
    let (tokens, alternatives) = tokenize::tokenize_with_alternatives(text, lang);
    let Some(predicate_start) = renvoi_predicate_start(&tokens) else {
        return Ok(Vec::new());
    };
    let apposition = svo::close_apposition();
    let terms: Vec<String> = tokens[..predicate_start]
        .iter()
        .zip(alternatives[..predicate_start].iter())
        .filter(|(_, alts)| alts.contains(&apposition))
        .map(|(tok, _)| tok.word.clone())
        .filter(|word| !lang.lookup(word).is_empty())
        .collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let targets = cites_pointers(refs, own_names, peers)?;
    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let mut pointers = Vec::new();
    for term in &terms {
        for (_, target) in &targets {
            pointers.push(DefinesByReferencePointer {
                term: term.clone(),
                target: target.clone(),
            });
        }
    }
    Ok(pointers)
}

/// The `definesByReference` grounding LENS — the lens form of
/// [`renvoi_pointers`], for the generic [`ground`](pr4xis_runtime::grounding::ground).
/// Mirrors [`cites_lens`]'s own shape (the SAME `refs_by_urn`/`own_names`/
/// `peers` side-tables, since renvoi resolution genuinely IS a citation
/// resolution): a node with no entry in `refs_by_urn`, or whose prose
/// carries no renvoi predicate at all, is a legitimate no-op
/// (`Ok(vec![])`) — the overwhelming majority of provisions.
pub fn renvoi_lens<'a>(
    lang: &'a English,
    refs_by_urn: &'a BTreeMap<String, Vec<UsCodeRef>>,
    own_names: &'a BTreeSet<String>,
    peers: &'a BTreeMap<String, Archive>,
) -> impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError> + 'a {
    move |node| {
        let Some(text) = node.lexical.as_deref() else {
            return Ok(Vec::new());
        };
        let Some(refs) = refs_by_urn.get(&node.name) else {
            return Ok(Vec::new());
        };
        let pointers = renvoi_pointers(text, lang, refs, own_names, peers)?;
        Ok(pointers
            .into_iter()
            .map(|p| (DEFINES_BY_REFERENCE_REL.to_string(), p.target))
            .collect())
    }
}

// The TYPE grounding lens is no longer statute-specific: it generalized to the
// source-agnostic `pr4xis_runtime::grounding::type_lens` (target-ontology name +
// peer archive parameterized), driven by the loader's general
// `crate::formal::meta::grounding::ground_declared` step over the grounding
// functor a `.prx` carries as data. A USC section grounding into LegalSources is
// now a plain special case of that mechanism, with zero statute-specific code.

/// A definiendum is a MENTIONED expression, never a used one.
///
/// A statutory definition talks ABOUT a written form — `The term “vessel”
/// includes every description of watercraft` (1 U.S.C. § 3), `“radiation”
/// means ionizing … radiation` (42 U.S.C. § 10003(1)) — while ordinary
/// operative prose USES every word in it. Quine's use/mention distinction
/// (W. V. O. Quine (1940), *Mathematical Logic*, §4 "Use versus mention";
/// Cappelen & Lepore, "Quotation", *Stanford Encyclopedia of Philosophy*
/// §3.1) is what separates the two, and it is the ONLY thing that does:
/// both shapes reduce to a two-argument `Sem::Prop` whose subject is a
/// `Sem::Concept` under a VerbNet-confirmed predicate.
///
/// This axiom states the closure `definiendum_words` enforces — a `defines`
/// pointer's term is read ONLY off a subject carrying
/// [`ExpressionUse::Mentioned`] — as a machine-checkable claim over the
/// semantic representation itself, with no lexicon, VerbNet or corpus load:
/// an ordinary used subject NP (`Any benefit provided … may be provided …`,
/// 5 U.S.C. § 5569(g), whose derivation really does put `Concept{"provided"}`
/// in the subject slot) yields NO definiendum, and the SAME concept marked
/// mentioned yields exactly one — including inside the coordinated-definienda
/// shape, whose reserved coordinator marker vouches for the coordinator but
/// never for its conjuncts.
pub struct ADefiniendumIsMentionedNeverUsed;

impl Axiom for ADefiniendumIsMentionedNeverUsed {
    fn verify(&self) -> Verdict {
        let concept = |word: &str, expression_use| Sem::Concept {
            word: word.to_string(),
            concepts: Vec::new(),
            role: montague::GrammaticalRole::Argument,
            expression_use,
        };
        // A used subject names no definiendum, whatever its surface.
        for used in ["provided", "words", "code", "may", "vessel"] {
            if !definiendum_words(&concept(used, ExpressionUse::Used)).is_empty() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // The same surface, mentioned, names exactly itself.
        if definiendum_words(&concept("vessel", ExpressionUse::Mentioned))
            != alloc::vec!["vessel".to_string()]
        {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Coordinated definienda (gap G5): the reserved apposition-coordinator
        // marker admits the SHAPE; each conjunct still has to be mentioned on
        // its own account.
        let marker = tokenize::APPOSITION_COORDINATOR_MARKER_AND;
        let coordinated = |items: Vec<Sem>| Sem::Func {
            word: marker.to_string(),
            body: items,
        };
        if definiendum_words(&coordinated(alloc::vec![
            concept("exploitation", ExpressionUse::Mentioned),
            concept("financial exploitation", ExpressionUse::Mentioned),
        ])) != alloc::vec![
            "exploitation".to_string(),
            "financial exploitation".to_string()
        ] {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        if !definiendum_words(&coordinated(alloc::vec![
            concept("county", ExpressionUse::Used),
            concept("state", ExpressionUse::Used),
        ]))
        .is_empty()
        {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ADefiniendumIsMentionedNeverUsed",
        "a `defines` pointer's definiendum is read only off a subject constituent whose expression is MENTIONED (metalinguistic), never off an ordinary used subject NP",
        "Quine (1940) Mathematical Logic §4 'Use versus mention'; Cappelen & Lepore, 'Quotation', Stanford Encyclopedia of Philosophy §3.1; Dickerson (1986) The Fundamentals of Legal Drafting 2nd ed. (means/includes definitional frames)"
    );
}
pr4xis::register_axiom!(
    ADefiniendumIsMentionedNeverUsed,
    "Quine (1940) Mathematical Logic §4; Cappelen & Lepore, 'Quotation', SEP §3.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;

    use crate::cognitive::linguistics::lambek::reduce::reduce_with_alternatives_and_table_and_width;
    use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};

    use crate::cognitive::linguistics::english::bridge::{FORM_KIND, project_archive_with_forms};

    /// The G7 statute-local minting namespace this file's own `defines`
    /// tests use — see [`is_known_written_form`]'s own doc.
    fn mint_domain() -> OntologyName {
        OntologyName::new_static("usc_t42_coinages")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn grounds_only_the_content_words_english_knows() {
        // "a dog is an animal" — content words dog, animal (a/is/an are stopwords);
        // both are sample written forms, so both ground.
        let english = English::sample();
        let pointers = denotes_pointers("a dog is an animal", &english);
        let words: Vec<&str> = pointers.iter().map(|p| p.word.as_str()).collect();
        assert!(
            words.contains(&"dog"),
            "dog is a known content word; got {words:?}"
        );
        assert!(words.contains(&"animal"), "animal is a known content word");
        assert!(
            !words.iter().any(|w| ["a", "is", "an"].contains(w)),
            "stopwords are not grounded; got {words:?}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_unknown_word_is_left_ungrounded() {
        // "dog" grounds; "xyzzy" is not a written form English knows → no pointer.
        let english = English::sample();
        let pointers = denotes_pointers("dog xyzzy", &english);
        assert_eq!(pointers.len(), 1);
        assert_eq!(pointers[0].word, "dog");
    }

    /// END-TO-END: statute prose → a `denotes` pointer → resolves (via the runtime
    /// `AtomResolver`) into the word's `ontolex:Form` atom in `english_wordnet`,
    /// and the resolved target IS a Form (never a sense). The producer + G3a
    /// resolver + G3b-1 Form layer, joined.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_produced_pointer_resolves_to_a_form_atom() {
        let english = English::sample();
        let archive = project_archive_with_forms(&english);
        let english_root = archive.root().unwrap();

        let pointer = denotes_pointers("the dog", &english)
            .into_iter()
            .find(|p| p.word == "dog")
            .expect("dog grounds");

        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "denotes".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();

        let resolved = resolver
            .resolve(&pointer.target)
            .expect("the produced denotes pointer resolves by content address");
        assert_eq!(
            resolved.kind, FORM_KIND,
            "the floor pointer resolves to an ontolex:Form, never a sense"
        );
        assert_eq!(resolved.name, "dog");
    }

    /// THE GENERIC LOOP: a content archive grounds via `ground(denotes_lens)` —
    /// adding typed `EdgeTarget::Grounded` edges to its nodes — and those edges
    /// resolve through the GENERIC `AtomResolver` to `ontolex:Form` atoms. No
    /// English-hardcoding outside the lens; the same `ground` would carry a `cites`
    /// lens over the same substrate. This is the ontological replacement for the
    /// reverted string side-channel.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_content_archive_grounds_via_the_lens_and_resolves_to_forms() {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::Definition;
        use pr4xis_runtime::grounding::ground;

        let english = English::sample();

        // A content archive — e.g. a statute provision node carrying prose. (The
        // USC bridge produces exactly such Definitions; here a bare one isolates
        // the grounding loop.)
        let content = Archive {
            nodes: alloc::vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t1/s1/a".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("the dog is an animal".to_string()),
            }],
            connections: alloc::vec![],
        };

        // Ground it with the lexical denotes lens — typed Grounded edges added.
        let grounded = ground(&content, denotes_lens(&english)).expect("the denotes floor grounds");
        let provision = &grounded.nodes[0];
        let denotes: Vec<&str> = provision
            .edges
            .iter()
            .filter(|(k, _)| k == "denotes")
            .filter_map(|(_, t)| match t {
                EdgeTarget::Grounded { .. } => Some("denotes"),
                EdgeTarget::Local(_) => None,
            })
            .collect();
        assert!(
            !denotes.is_empty(),
            "the provision grounds its content words"
        );

        // Resolve every grounded edge through the GENERIC resolver — each lands on
        // a Form atom (never a sense).
        let archive = project_archive_with_forms(&english);
        let english_root = archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "denotes".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();

        let mut resolved_forms = Vec::new();
        for (_, target) in provision.edges.iter().filter(|(k, _)| k == "denotes") {
            let form = resolver.resolve(target).expect("a grounded edge resolves");
            assert_eq!(
                form.kind, FORM_KIND,
                "grounds to an ontolex:Form, never a sense"
            );
            resolved_forms.push(form.name.clone());
        }
        assert!(resolved_forms.contains(&"dog".to_string()));
        assert!(resolved_forms.contains(&"animal".to_string()));
    }

    /// A same-archive citation — `href` names a node already declared in
    /// `own_names` — resolves `Local`, no peer needed.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_same_archive_citation_resolves_local() {
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t18/s1512".to_string(),
            text: "section 1512 of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t18/s1512".to_string());
        let peers = BTreeMap::new();

        let edges =
            cites_pointers(&refs, &own_names, &peers).expect("cites never fails closed here");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, CITES_REL);
        assert_eq!(
            edges[0].1,
            EdgeTarget::Local("/us/usc/t18/s1512".to_string())
        );
    }

    /// A cross-title citation whose target title IS supplied as a peer, and
    /// whose peer archive DOES declare the cited URN, resolves `Grounded` at
    /// the real content address — and that address genuinely resolves through
    /// the generic `AtomResolver`, mirroring
    /// `a_produced_pointer_resolves_to_a_form_atom`'s denotes/Form proof for
    /// cites/Section.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_cross_title_citation_resolves_and_the_edge_resolves_through_atom_resolver() {
        let cited = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t15/s78j".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: Some("Manipulative and deceptive devices".to_string()),
        };
        let peer_archive = Archive {
            nodes: alloc::vec![cited.clone()],
            connections: alloc::vec![],
        };
        let peer_root = peer_archive.root().unwrap();

        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t15/s78j".to_string(),
            text: "section 78j of title 15".to_string(),
        }];
        let own_names = BTreeSet::new(); // the citing archive does NOT declare it
        let mut peers = BTreeMap::new();
        peers.insert("usc_title_15".to_string(), peer_archive);

        let edges = cites_pointers(&refs, &own_names, &peers).expect("the peer declares the URN");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, CITES_REL);
        let target = edges[0].1.clone();
        assert!(
            matches!(&target, EdgeTarget::Grounded { ontology, .. } if ontology == "usc_title_15"),
            "resolves against the usc_title_15 peer, by the title parsed from the href"
        );

        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: "usc_title_15".to_string(),
            root: peer_root,
            role: "cites".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(&target)
            .expect("the minted Grounded edge resolves by content address");
        assert_eq!(resolved.name, "/us/usc/t15/s78j");
        assert_eq!(resolved.kind, "Section");
    }

    /// A citation whose title is not among the supplied peers, and one whose
    /// peer IS supplied but does not declare the exact cited URN, both leave
    /// NO edge — the same "no coverage → no pointer" discipline
    /// `an_unknown_word_is_left_ungrounded` proves for `denotes`. Never a
    /// guessed or partial bind.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_unresolvable_citation_is_left_ungrounded() {
        let unloaded_title = UsCodeRef {
            href: "/us/usc/t26/s501".to_string(), // title 26 is not a supplied peer
            text: "section 501 of title 26".to_string(),
        };
        let missing_subsection = UsCodeRef {
            href: "/us/usc/t15/s78zzz".to_string(), // wrong URN within a loaded peer
            text: "section 78zzz of title 15".to_string(),
        };
        let refs = alloc::vec![unloaded_title, missing_subsection];
        let own_names = BTreeSet::new();
        let mut peers = BTreeMap::new();
        peers.insert(
            "usc_title_15".to_string(),
            Archive {
                nodes: alloc::vec![Definition {
                    kind: "Section".to_string(),
                    name: "/us/usc/t15/s78j".to_string(), // present, but not the cited URN
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: None,
                }],
                connections: alloc::vec![],
            },
        );

        let edges = cites_pointers(&refs, &own_names, &peers).expect("unresolved is not a failure");
        assert!(
            edges.is_empty(),
            "neither citation resolves — no edge, never a guessed bind; got {edges:?}"
        );
    }

    /// THE GENERIC LOOP, cites edition: a content archive grounds via
    /// `ground(cites_lens(...))`, and the minted edge resolves through the
    /// GENERIC `AtomResolver` — the verbatim shape of
    /// `a_content_archive_grounds_via_the_lens_and_resolves_to_forms`, proving
    /// `cites` really is "another lens of the same shape" as `denotes`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_content_archive_grounds_via_the_cites_lens_and_resolves() {
        use pr4xis_runtime::grounding::ground;

        let citing = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t18/s1513".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: Some("Retaliating against a witness".to_string()),
        };
        let content = Archive {
            nodes: alloc::vec![citing],
            connections: alloc::vec![],
        };
        let own_names: BTreeSet<String> = content.nodes.iter().map(|n| n.name.clone()).collect();

        let cited = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t15/s78j".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: None,
        };
        let peer_archive = Archive {
            nodes: alloc::vec![cited],
            connections: alloc::vec![],
        };
        let peer_root = peer_archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert("usc_title_15".to_string(), peer_archive);

        let mut refs_by_urn = BTreeMap::new();
        refs_by_urn.insert(
            "/us/usc/t18/s1513".to_string(),
            alloc::vec![UsCodeRef {
                href: "/us/usc/t15/s78j".to_string(),
                text: "section 78j of title 15".to_string(),
            }],
        );

        let grounded = ground(&content, cites_lens(&refs_by_urn, &own_names, &peers))
            .expect("the cites lens grounds");
        let provision = &grounded.nodes[0];
        let (kind, target) = provision
            .edges
            .iter()
            .find(|(k, _)| k == CITES_REL)
            .expect("the citation grounded");
        assert_eq!(kind, CITES_REL);

        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: "usc_title_15".to_string(),
            root: peer_root,
            role: "cites".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(target)
            .expect("the grounded citation resolves");
        assert_eq!(resolved.name, "/us/usc/t15/s78j");
    }

    // ---- defines_pointers / defines_lens ----
    //
    // Uses the REAL loaded English (`english_loaded`, the full committed
    // WordNet, mirroring `lambek::integration_tests`'s own "no lies" full-
    // pipeline discipline) and the REAL loaded VerbNet
    // (`verbnet::store::verbnet_classes_loaded`) throughout — a sparse
    // fixture lexicon would conflate "the grammar genuinely has no
    // scaffolding for this shape" with "the fixture just didn't know this
    // word", which is exactly the ambiguity these tests exist to rule out.

    use crate::cognitive::linguistics::english::english_loaded;
    use crate::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;

    /// The real, complete, standalone sentence at `/us/usc/t15/s6603/h/6/A`
    /// (byte-verified against
    /// `crates/domains/data/legal/uscode/usc_title_15/usc_title_15-pl-119-90.xml`,
    /// offset 26334083) — the simplest shape this grammar actually covers:
    /// a single-word definiendum, "means", and a determiner+adjective+noun
    /// object with no PP/coordination/relative clause.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_from_real_statute_prose() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive,
            "a VerbNet-confirmed \"means\" declarative is exhaustive"
        );
    }

    /// TASK #14 (S2's "includes" partial-definition verb frame): the real,
    /// complete, standalone sentence at `/us/usc/t50/s797/a/4/D` (byte-
    /// verified against `usc_title_50-pl-119-90.xml`, heading "Regulation as
    /// including order"): "The term "regulation" includes an order." —
    /// EXACTLY `recognizes_the_term_x_means_y_from_real_statute_prose`'s own
    /// shape (single-word definiendum, a two-word determiner+noun object, no
    /// PP/coordination/relative clause) with "includes" standing in for
    /// "means" — proving `is_partial_definition_verb` alone (no reassembly
    /// involved; this node's OWN content is already a complete declarative)
    /// closes a real, previously-uncovered shape: VerbNet does NOT confirm
    /// "include" (see `is_partial_definition_verb`'s own doc), so before
    /// this task this sentence yielded NO pointer at all.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_includes_y_as_a_partial_definition_from_real_statute_prose() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}regulation\u{201D} includes an order.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "regulation");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Partial,
            "\"includes\" is Dickerson's PARTIAL definition frame, not means's exhaustive one"
        );
    }

    /// A non-defining sentence (no "the term ... means" shape at all) yields
    /// no pointer — the Honest baseline, the same "no coverage → no
    /// pointer" discipline `an_unknown_word_is_left_ungrounded` proves for
    /// `denotes`.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_non_defining_sentence_yields_no_pointer() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers("The dog runs to the store.", en, en, vn, &mint_domain());
        assert!(pointers.is_empty(), "got {pointers:?}");
    }

    /// WAS AN HONEST BASELINE (TASK #18, G5), NOW GREEN: the REAL,
    /// UNMODIFIED coordinated-definiendum sentence — title 15's fiber
    /// definition, `/us/usc/t15/s70/b` (byte-verified against
    /// `usc_title_15-pl-119-90.xml`): "The term 'fiber' or 'textile fiber'
    /// means ...". Coordination stopped being the blocker when G5 scaffolded
    /// "the term 'X' or 'Y'"
    /// (`recognizes_the_term_x_or_y_means_z_coordinated_apposition` below
    /// proves the mechanism on this EXACT pair, isolated from this
    /// sentence's own definiens); what still blocked it after that was its
    /// OWN definiens — a relative-clause chain ("which is capable of being
    /// spun...", "which is the basic structural element..."). That is the
    /// definiens-side attachment class the PARTIAL-PARSE goal removes: the
    /// definitional clause no longer has to be the whole span, so an
    /// unattachable relative-clause chain in the definiens cannot discard the
    /// definienda the chart already analysed
    /// (`AnUnattachableAdjunctNeverHidesItsClause`).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_coordinated_definiendum_sample_recovers_both_definienda() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}fiber\u{201D} or \u{201C}textile fiber\u{201D} means \
                     a unit of matter which is capable of being spun into a yarn or made \
                     into a fabric by bonding or by interlacing in a variety of methods \
                     including weaving, knitting, braiding, felting, twisting, or webbing, \
                     and which is the basic structural element of textile products.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        let terms: BTreeSet<&str> = pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            ["fiber", "textile fiber"].into_iter().collect(),
            "got {pointers:?}"
        );
    }

    /// HONEST BASELINE (TASK #16, G2): the real, UNMODIFIED headline
    /// sentence — title 18's "vessel of the United States" definition
    /// (byte-verified against `usc_title_18-pl-119-90.xml`): "The term
    /// 'vessel of the United States', as used in this title, means ...".
    /// BEFORE task #16, the comma-set-off ", as used in this title," adjunct
    /// itself broke subject-verb adjacency and blocked the parse outright.
    /// AFTER: the adjunct is absorbed (see
    /// `recognizes_the_term_x_means_y_behind_a_medial_as_used_adjunct`
    /// below, which isolates exactly this fix against the SAME proven base
    /// declarative G1's own isolation tests use), but this REAL, UNMODIFIED
    /// sentence still yields no pointer — for a DIFFERENT, deeper-in-the-
    /// derivation reason this task does not touch: its definiens is a
    /// coordinated NP chain ("a vessel ... , or any citizen thereof, or any
    /// corporation ...") — G4 territory (definiens-side coordination),
    /// separately scoped. The SAME "isolate the fix, then honestly show the
    /// real sentence needs a DIFFERENT capability too" pattern
    /// `a_real_unmodified_for_purposes_of_this_subsection_sentence_still_yields_no_pointer`
    /// (G1) already establishes.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_unmodified_vessel_of_the_united_states_sentence_still_yields_no_pointer() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}vessel of the United States\u{201D}, as used in this \
                     title, means a vessel belonging in whole or in part to the United \
                     States, or any citizen thereof, or any corporation created by or \
                     under the laws of the United States, or of any State, Territory, \
                     District, or possession thereof.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert!(
            pointers.is_empty(),
            "the definiens-side coordination needs a separate G4 capability \
             this task does not build; got {pointers:?}"
        );
    }

    // ---- TASK #17 (G4): definiens-side NP complexity ----
    //
    // Coordination (a) and PP chains (b) — this task's own priority order —
    // plus the bare-noun definiens (f) mechanism that turned out to be their
    // shared prerequisite (a coordinated OR PP-modified definiens routinely
    // has at least one determiner-less conjunct: "bodily injury,
    // impairment, or disease"; "the Secretary of Health and Human
    // Services"). Grounded in `tokenize::find_list_coordinator_commas`,
    // `montague::flatten_coordination`, and `definiens_cost_table`'s scoped
    // `N -> NP` unary rule — see each one's own doc for citations.

    /// The REAL report-cited n-ary coordination shape, 42 U.S.C. §
    /// 289b–1(f)(2) "assistance" — a genuine WordNet-known SINGLE-WORD
    /// definiendum (unlike "physical harm"/"family caregiver"/"elder
    /// abuse" below, none of which WordNet indexes as a compound headword —
    /// see the HONEST BASELINEs below for why that written-form-floor gap,
    /// G7, is separately scoped), so this real sentence proves the
    /// coordination mechanism (`tokenize::find_list_coordinator_commas` +
    /// `montague::flatten_coordination`) end to end with NO substitution
    /// needed: "The term "assistance" ... means a grant, contract, or
    /// cooperative agreement." (dropping only the medial ", with respect to
    /// conducting a project of research," supplement — byte-verified as
    /// real prose, but its own interior head "with" is outside G2's closed
    /// `is_medial_supplement_interior_head` set, "as"/"when"/"used", a
    /// separately-scoped gap, not this task's). THREE conjuncts, mixed:
    /// "a grant" (determined), "contract" (a BARE single-word noun,
    /// `definiens_cost_table`'s leaf-level unary rule), "cooperative
    /// agreement" (a BARE two-word adjective+noun SPAN, the SAME unary rule
    /// applied to a derived span, not just a leaf —
    /// `montague::close_unary_semantic` runs at every chart cell, mirroring
    /// `reduce::close_unary`'s own per-span fixpoint).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_real_three_item_np_coordination() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}assistance\u{201D} means a grant, contract, or \
                     cooperative agreement.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "assistance");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
    }

    // ---- Tokenizer comma-dispatch priority regression: a lexically-
    // ambiguous (OOV/rare-verb-sense) conjunct inside a 6+-item coordinated
    // definiens ----
    //
    // Root cause (confirmed by direct chart/Montague instrumentation,
    // `probe_company_chart_divergence_point`,
    // crates/praxis-corpus-tests/tests/scratch_probe.rs — NOT a chart-level
    // coordination-composition gap): `tokenize::collapse_medial_comma_
    // adjuncts`'s post-verb medial-supplement gate used to be checked BEFORE
    // its own `list_coordinator_commas` map, so a comma the SAME function had
    // already, structurally identified as a plain list-coordination comma
    // could still be hijacked by the post-verb gate whenever the PRECEDING
    // conjunct's multi-word compound HEAD noun carried an incidental
    // transitive-verb sense not adjacent to its own determiner (`is_
    // determiner` only checks the IMMEDIATELY preceding word — a nominal
    // premodifier, e.g. "joint-stock", defeats it) AND 2+ conjuncts remained
    // in the list (giving the gate a plausible-looking closing comma). The
    // fix reorders the checks; see `collapse_medial_comma_adjuncts`'s own
    // doc for the full mechanism and why this is a priority-inversion fix,
    // not an arity cap.

    /// Sweeps the EXACT arity range the original bisection isolated —
    /// "joint-stock company" (a REAL WordNet-attested archaic verb sense on
    /// "company") sitting in a coordinated list with 0, 1, 2, and 3
    /// conjuncts trailing it. The fix must be N-ARY ROBUST (every arity
    /// extracts), not an arity ceiling nudged from 5 to 7: a coordinated
    /// list of ANY length containing this — or any other — verb-polysemous
    /// compound conjunct must extract, because `list_coordinator_commas` is
    /// computed the same way regardless of how many conjuncts follow.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_joint_stock_company_conjunct_extracts_regardless_of_how_many_conjuncts_trail_it() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let cases: [(&str, &str); 4] = [
            (
                "0 trailing (joint-stock company is the LAST conjunct)",
                "\u{201C}Company\u{201D} means a corporation, a partnership, an \
                 association, or a joint-stock company.",
            ),
            (
                "1 trailing conjunct",
                "\u{201C}Company\u{201D} means a corporation, a partnership, an \
                 association, a joint-stock company, or a trust.",
            ),
            (
                "2 trailing conjuncts — the exact arity the original \
                 bisection isolated as the first failure",
                "\u{201C}Company\u{201D} means a corporation, a partnership, an \
                 association, a joint-stock company, a trust, or a fund.",
            ),
            (
                "3 trailing conjuncts",
                "\u{201C}Company\u{201D} means a corporation, a partnership, an \
                 association, a joint-stock company, a trust, a fund, or a group.",
            ),
        ];
        for (label, text) in cases {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert_eq!(pointers.len(), 1, "[{label}] got {pointers:?} for {text:?}");
            assert_eq!(pointers[0].term, "company", "[{label}]");
            assert_eq!(
                pointers[0].exhaustiveness,
                DefinitionExhaustiveness::Exhaustive,
                "[{label}]"
            );
        }
    }

    /// The REAL, byte-verified Investment Company Act definition, 15 U.S.C.
    /// § 80a-2(a)(8) (<https://www.law.cornell.edu/uscode/text/15/80a-2>) —
    /// the exact statute the original bisection that found this bug was
    /// modeled on. Trimmed to the in-scope declarative clause only (the
    /// statute's own trailing "; or any receiver, trustee ... in his
    /// capacity as such" is a SEPARATE semicolon-joined clause, out of scope
    /// for this single-declarative grammar, the same "trim to the tested
    /// capability" discipline
    /// `recognizes_the_term_x_means_y_behind_a_real_three_item_np_coordination`'s
    /// own "assistance" fixture above already establishes for its dropped
    /// medial supplement).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_investment_company_act_company_definition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}Company\u{201D} means a corporation, a \
                     partnership, an association, a joint-stock company, a trust, \
                     a fund, or any organized group of persons whether incorporated \
                     or not.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "company");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
    }

    /// A SECOND, independent real statute carrying the SAME "joint-stock
    /// company" conjunct inside a longer coordinated list — confirms the bug
    /// (and the fix) is not a one-off single-example coincidence: the
    /// Securities Act of 1933's "person" definition, 15 U.S.C. § 77b(a)(2)
    /// (<https://www.law.cornell.edu/uscode/text/15/77b>), an EIGHT-item
    /// list with THREE conjuncts trailing "a joint-stock company". Trimmed
    /// to drop only the statute's own trailing "thereof" (a bare, archaic
    /// pronominal adverb postmodifying "political subdivision" — "thereof"/
    /// "herein"/"thereto"-class postmodification has no grammar coverage in
    /// this file at all, a SEPARATE, unrelated gap confirmed by direct probe
    /// (`probe_person_definition_residual_failure`,
    /// crates/praxis-corpus-tests/tests/scratch_probe.rs): the untrimmed
    /// sentence still yields 0 pointers even WITH this task's fix, but
    /// dropping "thereof" alone (keeping every other word, including
    /// "joint-stock company" mid-list) already succeeds — proving the
    /// coordination fix and the "thereof" gap are independent, the same
    /// "isolate the fix, then honestly show what ELSE the real sentence
    /// needs" pattern this file's other HONEST BASELINE tests establish).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_securities_act_person_definition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}person\u{201D} means an individual, a \
                     corporation, a partnership, an association, a joint-stock \
                     company, a trust, any unincorporated organization, or a \
                     government or political subdivision.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "person");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
    }

    /// WAS AN HONEST BASELINE, NOW GREEN: the REAL, complete, UNMODIFIED
    /// § 77b(a)(2) "person" sentence, trailing "thereof" and all. The
    /// "thereof" gap it recorded was a definiens-side ATTACHMENT gap — the
    /// archaic pronominal adverb has no postmodifier category, so it could
    /// not attach to "a government or political subdivision" and the
    /// whole-string goal threw away the complete definitional clause in front
    /// of it. Under the partial-parse goal that clause is found where it sits
    /// (`AnUnattachableAdjunctNeverHidesItsClause`); the postmodifier
    /// capability itself is still unbuilt, and still not needed to know WHICH
    /// term the provision defines.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_securities_act_person_sentence_recovers_its_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}person\u{201D} means an individual, a \
                     corporation, a partnership, an association, a joint-stock \
                     company, a trust, any unincorporated organization, or a \
                     government or political subdivision thereof.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "person");
    }

    /// HONEST BASELINE: the REAL, complete, UNMODIFIED § 300ii(5) "family
    /// caregiver" sentence (byte-verified against
    /// `usc_title_42-pl-119-90.xml`): "The term "family caregiver" means an
    /// unpaid family member, a foster parent, or another unpaid individual,
    /// who provides in-home monitoring, management, supervision, or
    /// treatment of a child or adult with a special need." Confirmed (via
    /// `PRAXIS_DEBUG_DEFINES=1`) still blocked BEFORE the coordination
    /// mechanism above ever gets a chance to matter, for a DIFFERENT reason
    /// this task does not build: "family member" is a bare NOUN-NOUN
    /// compound — "family" carries no adjective (`N/N`) sense in WordNet at
    /// all (verified directly against the loaded data), so it cannot
    /// combine with "member" the way "cooperative agreement" above does.
    /// The relative clause's own "in-home" (an out-of-vocabulary hyphenated
    /// compound with no attributive-adjective reading either) is a SECOND,
    /// independent blocker further along the same sentence. Both are
    /// separately-scoped, unbuilt capabilities (bare noun-noun premodification
    /// generally). BOTH sit inside the DEFINIENS, so both are exactly what the
    /// partial-parse goal stops treating as fatal: neither is needed to know
    /// that this provision defines "family caregiver", and under the
    /// whole-string goal each one on its own discarded that definiendum
    /// (`AnUnattachableAdjunctNeverHidesItsClause`). The two lexical gaps
    /// themselves remain open and unbuilt.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_family_caregiver_sentence_recovers_its_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}family caregiver\u{201D} means an unpaid family \
                     member, a foster parent, or another unpaid individual, who \
                     provides in-home monitoring, management, supervision, or \
                     treatment of a child or adult with a special need.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "family caregiver");
    }

    /// The REAL report-cited example, 42 U.S.C. § 3002(42): "The term
    /// "physical harm" means bodily injury, impairment, or disease." — the
    /// SAME coordination + span-promotion mechanism the isolation test
    /// above proves, recombined here with the proven "consumer" definiendum
    /// (15 U.S.C. § 6603(h)(6)(A), this file's own base declarative
    /// throughout G1-G3) rather than the real "physical harm": confirmed
    /// (via `PRAXIS_DEBUG_DEFINES=1`) that WordNet does not index "physical
    /// harm" as a compound headword, so the REAL, unmodified sentence hits
    /// the SAME separately-scoped written-form-floor gap (G7) the HONEST
    /// baseline below documents — this isolation proves the DEFINIENS-side
    /// mechanism this task actually builds, independent of that gap.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_the_real_physical_harm_bare_noun_coordination() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}consumer\u{201D} means bodily injury, impairment, or disease.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// UPDATED BY G7: the REAL, complete, UNMODIFIED § 3002(42) sentence
    /// (byte-verified): "The term "physical harm" means bodily injury,
    /// impairment, or disease." WordNet does not index "physical harm" as a
    /// compound headword (verified directly against the loaded lexicon) —
    /// before G7 (defines-lens gap backlog), `defines_pointers`' own
    /// written-form floor dropped the pointer outright; NOW it mints
    /// "physical harm" into the statute-local `mint_domain` instead (see
    /// [`is_known_written_form`]'s own doc), so this REAL sentence grounds a
    /// real (Grounded, non-`english_wordnet`) `defines` edge.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_physical_harm_sentence_now_mints_its_out_of_lexicon_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text =
            "The term \u{201C}physical harm\u{201D} means bodily injury, impairment, or disease.";
        let domain = mint_domain();
        let pointers = defines_pointers(text, en, en, vn, &domain);
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "physical harm");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "\"physical harm\" is not a WordNet-indexed compound headword, so it \
             is MINTED into the statute-local domain rather than dropped — got \
             {:?}",
            pointers[0].target
        );
    }

    /// The REAL report-cited example, 42 U.S.C. § 3002(15): "The term
    /// "elder abuse" means abuse of an older individual." — defines-lens
    /// gap G4(f), a bare (determiner-less) SINGLE-WORD definiens ("abuse")
    /// immediately followed by a PP chain ("of an older individual") that
    /// already composes via the pre-existing `svo::preposition` mechanism
    /// (the same "Secretary of Commerce" shape this module's own sibling
    /// test suite in `montague.rs` already proves) — so this ALSO stands as
    /// this task's clean gap-(b) PP-chain proof, needing only the bare-noun
    /// leaf promotion, no coordination at all. Recombined with the proven
    /// "consumer" definiendum for the SAME reason as the physical-harm
    /// isolation test above ("elder abuse" is not a WordNet-indexed
    /// compound headword either — see the honest baseline below).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_the_real_elder_abuse_bare_noun_definiens() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}consumer\u{201D} means abuse of an older individual.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// UPDATED BY G7: the REAL, complete, UNMODIFIED § 3002(15) sentence:
    /// "The term "elder abuse" means abuse of an older individual." The
    /// SAME mechanism
    /// (`a_real_unmodified_physical_harm_sentence_now_mints_its_out_of_lexicon_definiendum`'s
    /// own doc) — "elder abuse" is not a WordNet-indexed compound headword
    /// either, so it is now MINTED rather than dropped.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_elder_abuse_sentence_now_mints_its_out_of_lexicon_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}elder abuse\u{201D} means abuse of an older individual.";
        let domain = mint_domain();
        let pointers = defines_pointers(text, en, en, vn, &domain);
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "elder abuse");
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "\"elder abuse\" is not a WordNet-indexed compound headword, so it \
             is MINTED into the statute-local domain rather than dropped — got \
             {:?}",
            pointers[0].target
        );
    }

    /// The REAL report-cited PP-CHAIN shape, 42 U.S.C. § 242q–4(2): "The
    /// term "Secretary" means the Secretary of Health and Human Services."
    /// — a determiner-headed PP object ("the Secretary") postmodified by a
    /// PP whose own complement is itself COORDINATED ("Health and Human
    /// Services": a bare-noun "Health" coordinated with the collapsed
    /// capitalized-run proper-noun surface "Human Services").
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_the_real_secretary_pp_chain() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}Secretary\u{201D} means the Secretary of Health and \
                     Human Services.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "secretary");
    }

    // ---- TASK #15 (G1): fronted scope-setting sentential adjunct ----
    //
    // Four REAL, byte-verified fragments, all from
    // `usc_title_42-pl-119-90.xml` (the report's own dominant title),
    // covering all four constructions the gap report names verbatim ("For
    // purposes of X," / "In this subsection," / "Except for the purposes of
    // X," / "Subject to Y,") and both new categories
    // (`fronted_scope_adjunct_np` for "for"/"in", `fronted_scope_adjunct_pp`
    // for "except"/"subject"):
    // - "For this purpose, any evaluation of such assets shall be made..."
    // - "In this section, the term "fiscal agent" means a carrier
    //   described in..."
    // - "Except for a capital offense, no individual or person shall be
    //   prosecuted..."
    // - "Subject to this part, a State to which a grant is made under..."
    //
    // Each is recombined with "The term 'consumer' means a natural
    // person." (15 U.S.C. § 6603(h)(6)(A)) — the SAME already-grammar-complete
    // real declarative this file's own test suite grounds throughout —
    // isolating the G1 adjunct-attachment fix from a SEPARATE, still-open
    // gap: the REPORT's own unmodified headline sentences ("For purposes of
    // this subsection, the term 'X' means...", "Subject to subparagraphs
    // (B) and (C), the term 'hospice care' means...") still do not reach a
    // full green, because their OWN internal NP objects need capabilities
    // G1 does not touch — a bare plural noun ("purposes"/"purpose") with no
    // determiner (WordNet's verb sense of "purpose" also outranks its noun
    // sense in this grammar's entry ordering), and coordination inside an
    // apposed NP ("subparagraphs (B) and (C)") — confirmed empirically
    // (`chart_reduce` fails on the UNMODIFIED report sentences even with
    // this task's fix applied). This is the SAME isolation precedent
    // `defines_lens_recovers_a_definition_shadowed_by_a_heading` (S1)
    // already establishes for its own upstream fix.
    //
    // BEFORE this task: the fronted adjunct alone stranded the derivation —
    // `reduce_with_alternatives` never reached a complete `S`, so
    // `defines_pointers` fell back to `montague::interpret` on PRIMARY types
    // only, which got stuck at the adjunct itself (the report's own
    // diagnostic signature, `interpret=Func(word="for"|"in"|"except"|
    // "subject", absorbed=0)`) — no pointer, for the WRONG reason (a grammar
    // gap, not an honest absence of a definition).
    // AFTER: the adjunct is consumed as a transparent scope-setting
    // modifier and the REST clause grounds exactly as it already does on
    // its own.

    /// The NP-complement variant ("for"/"in",
    /// [`crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_np`]).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_fronted_for_this_purpose_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "For this purpose, the term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive,
            "the REST clause's own \"means\" declarative is exhaustive, unaffected by the adjunct"
        );
    }

    /// The SAME NP-complement variant, "in" instead of "for" — proving the
    /// closed class, not just one member of it.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_fronted_in_this_section_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "In this section, the term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// The PP-complement variant ("except",
    /// [`crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_pp`]):
    /// "except" absorbs the ALREADY-derived `NP\NP` "for a capital
    /// offense" (via the ordinary, unmodified [`preposition`
    /// category](crate::cognitive::linguistics::lambek::types::svo::preposition)).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_fronted_except_for_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "Except for a capital offense, the term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// The SAME PP-complement variant, "subject" instead of "except" — the
    /// Huddleston & Pullum absolute-adjunct reading ("subject" heading its
    /// own small clause over a "to"-PP).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_fronted_subject_to_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "Subject to this part, the term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    // ---- TASK #16 (G2): medial comma-delimited supplement ----
    //
    // Two positions, both real report-cited fragments recombined with the
    // SAME already-grammar-complete "the term 'consumer' means a natural
    // person." declarative (15 U.S.C. § 6603(h)(6)(A)) G1's own tests
    // isolate on — the SAME isolation precedent, for the SAME reason: the
    // report's own unmodified headline sentences carry SEPARATE,
    // still-open gaps (G4 definiens-side coordination/PP-chains) in their
    // OWN object NP, orthogonal to the adjunct-attachment mechanism this
    // task builds. See `a_real_unmodified_vessel_of_the_united_states_sentence_still_yields_no_pointer`
    // below for the honest real-sentence baseline.

    /// POST-VERB position (`svo::medial_supplement_verb`): "means, with
    /// respect to Y, Z" — the EVV headline shape, verbatim from 42 U.S.C.
    /// § 1396b(l)(5): 'The term "electronic visit verification system"
    /// means, with respect to personal care services or home health care
    /// services, a system under which ...'. This is the SINGLE MOST
    /// caregiving-costly construction the gap report names (EVV).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_medial_verb_supplement() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D} means, with respect to personal care \
             services or home health care services, a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive,
            "the REST clause's own \"means\" declarative is exhaustive, \
             unaffected by the post-verb supplement"
        );
    }

    /// SUBJECT-VERB position (`svo::medial_supplement_np`), "used" heading
    /// the interior directly: ", used with respect to Y, means ..." —
    /// verbatim from the report's "inclusion" example: 'The term
    /// "inclusion", used with respect to individuals with developmental
    /// disabilities, means ...'.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_medial_used_with_respect_to_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D}, used with respect to individuals with \
             developmental disabilities, means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// SUBJECT-VERB position, "as" heading the interior: ", as used in this
    /// title, means ..." — verbatim from title 18's "vessel of the United
    /// States" definition (the SAME real fragment
    /// `a_real_unmodified_vessel_of_the_united_states_sentence_still_yields_no_pointer`
    /// below carries as its own committed fixture — this test isolates the
    /// ADJUNCT-ATTACHMENT fix from that sentence's SEPARATE, still-open G4
    /// definiens-coordination gap).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_medial_as_used_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D}, as used in this title, means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
    }

    /// TRAILING comma-set-off "whether X or Y" adjunct — an EXHAUSTIVE
    /// CONDITIONAL (Huddleston & Pullum 2002, Ch. 8 "Adjuncts") with no
    /// CLOSING comma, so it is architecturally distinct from every OTHER
    /// medial-supplement case in this test module (those all require a
    /// closing comma to bound the span). Root-caused via direct bisection
    /// this session: "whether" carried no valid chart category, breaking
    /// the WHOLE derivation even for a trivial single-item definiens —
    /// fixed by `tokenize::is_trailing_alternative_adjunct_head` +
    /// `collapse_medial_comma_adjuncts`'s new trailing-span drop. Real
    /// fragment, 15 U.S.C. § 80a-2(a) ("Company" definition, Investment
    /// Company Act of 1940) minus its own separate, still-open coordination
    /// gap (see `defines_pointers_still_misses_a_long_or_coordinated_
    /// definiens_containing_an_ambiguous_compound` below) — isolated here
    /// to a single-item definiens so this test proves ONLY the adjunct fix.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_means_y_behind_a_trailing_whether_or_adjunct() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "\u{201C}Company\u{201D} means any organized group of persons, \
             whether incorporated or unincorporated.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "company");
    }

    /// GREEN — was the RED ratchet target for a real, bisected gap; now
    /// closed. The real definiendum-plus-definiens shape, 15 U.S.C. §
    /// 80a-2(a) ("Company"), combining BOTH fixes this session made to
    /// `collapse_medial_comma_adjuncts` (`tokenize.rs`): the trailing
    /// exhaustive-conditional adjunct ("whether incorporated or
    /// unincorporated", `is_trailing_alternative_adjunct_head`) AND the
    /// list-coordination/medial-supplement priority-inversion bug
    /// (`list_coordinator_commas` now checked BEFORE either supplement
    /// gate). Bisected precisely (see
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`'s
    /// `probe_bisect_company_definition_failure` and
    /// `probe_company_chart_divergence_point`): NOT pure coordination arity
    /// (a 7-item all-plain-word "or" list always succeeded); NOT the
    /// trailing adjunct alone (removing it still failed); the trigger was
    /// specifically a lexically-ambiguous item ("joint-stock company", an
    /// out-of-vocabulary hyphenated compound carrying BOTH a nominal-
    /// premodifier N/N reading and a proper-noun NP reading — the SAME
    /// dual-reading OOV mechanism
    /// `an_oov_singleton_also_offers_nominal_premodifier_alongside_proper_noun`
    /// in `tokenize.rs` documents) combined with a 6+-item "or"
    /// coordination, which corrupted the token stream BEFORE the chart ever
    /// ran (not a chart-search/composition gap — see
    /// `nominal_coordinator_np`/`_n`'s own doc in `types.rs` for why that
    /// hypothesis was ruled out). This exact sentence is the arity-2
    /// (2-trailing-conjunct) case
    /// `a_joint_stock_company_conjunct_extracts_regardless_of_how_many_conjuncts_trail_it`
    /// sweeps in isolation; kept here as the full combined-fixture
    /// end-to-end regression (both fixes' trigger text in one real
    /// statutory sentence), not deleted as a duplicate.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_company_definition_combining_the_whether_adjunct_and_n_ary_or_coordination_fixes()
     {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "\u{201C}Company\u{201D} means a corporation, a partnership, an \
             association, a joint-stock company, a trust, a fund, or any \
             organized group of persons, whether incorporated or \
             unincorporated.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "company");
    }

    // ---- Title 42 pathological-outlier fixes (root causes A/B/C) ----

    /// Root cause A regression: the REAL, table-contaminated 42 U.S.C. §
    /// 1586(a) reassembled candidate (chapeau + all three child paragraphs —
    /// `dangling_chapeau_reassembly_index`'s "ALL-joined" shape) — verbatim,
    /// as `defines_pointers` receives it TODAY (fetched via this session's
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`
    /// `temp_dump_title42_candidate_text_for_regression_fixtures` probe
    /// against the live corpus, then copied here so this test needs no
    /// corpus load of its own — matching every other `recognizes_the_real_*`
    /// fixture's style in this file).
    ///
    /// BEFORE the fix (`UsCodeMixed::plain_text` backing the flat
    /// `content`/`chapeau` fields): this candidate's real text was 7804
    /// chars, because paragraph (a)(3)'s `<content>` embeds a ~50-row XHTML
    /// `<table>` of historical housing-project state/project-number/agency
    /// data (42 U.S.C. § 1586(a)(3) — `crates/domains/data/legal/uscode/
    /// usc_title_42/usc_title_42-pl-119-90.xml`, line 238061) directly after
    /// "...for the administration of the project:" — and TIMED OUT past 60s
    /// (`probe_title42_candidates_with_bounded_timeout`'s own committed
    /// table). AFTER the fix (`content`/`chapeau` now derive from
    /// `UsCodeMixed::prose_text`, which skips an embedded `<table>` subtree
    /// the same way it already skipped `<note type="footnote">` — see
    /// `UsCodeContentNode::push_prose_text`'s own doc): this candidate is
    /// 1366 chars (the table's ~6400 chars of row data gone), and this
    /// exact text is the FULL real candidate, not a hand-trimmed excerpt.
    ///
    /// Asserts BOTH halves of the regression: no pointer (this is
    /// "authorized to convey... if— [conditions]" prose, never a "means"/
    /// "includes" declarative — the honest "no coverage → no pointer"
    /// answer, unchanged by the fix), AND completes fast — a generous 5s
    /// bound, ~20x the measured post-fix time (`probe_bisect_title42_
    /// pathological_candidates`'s own printed numbers), so a REAL
    /// regression (a table, or something structurally similar, leaking
    /// gibberish into `content`/`chapeau` again) fails loud here long
    /// before anyone has to re-run the corpus-wide sweep to notice.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_housing_project_table_candidate_is_fast_and_table_free() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The Secretary of Housing and Urban Development is specifically \
             authorized to convey the following housing projects to the following \
             local public housing agencies respectively, if\u{2014} on or before \
             January 30, 1953, (i) the conveyance is requested by the governing \
             body of the municipality or county and (ii) the public housing \
             agency has demonstrated to the satisfaction of the Secretary of \
             Housing and Urban Development that there is a need for low-rent \
             housing (as such term is defined in the United States Housing Act \
             of 1937 [42 U.S.C. 1437 et seq.]) within the area of operation of \
             such public housing agency which is not being met by private \
             enterprise; the Secretary of Housing and Urban Development \
             determines that the project requested will meet such need in whole \
             or in part, and is suitable for low-rent housing use; and on or \
             before June 30, 1953, the governing body of the municipality or \
             county enters into an agreement with the public housing agency \
             (satisfactory to the Secretary of Housing and Urban Development) \
             providing for local cooperation and payments in lieu of taxes not \
             in excess of the amount permitted by subsection (c)(5) of this \
             section, and the public housing agency enters into an agreement \
             with the Secretary of Housing and Urban Development (in accordance \
             with subsection (c) of this section) or for the administration of \
             the project:";
        assert!(
            !text.contains("Birmingham") && !text.contains("Gadsden"),
            "the real candidate must no longer carry the embedded table's row \
             data — if this fails, the fetched fixture above is stale, re-derive \
             it via `temp_dump_title42_candidate_text_for_regression_fixtures`"
        );
        let started = std::time::Instant::now();
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        let elapsed = started.elapsed();
        assert!(
            pointers.is_empty(),
            "no \"means\"/\"includes\" declarative in this prose; got {pointers:?}"
        );
        assert!(
            elapsed.as_secs_f64() < 5.0,
            "regressed: table-free candidate took {:.3}s (was ~0.25s post-fix, \
             would have timed out past 60s pre-fix)",
            elapsed.as_secs_f64()
        );
    }

    // ---- Footnote-exclusion regression (tried, reverted) ----
    //
    // A `NonProseSubtreeKind::Footnote`/`FootnoteRef` exclusion (mirroring
    // the `Table` exclusion above) was tried in `runtime_types.rs` and
    // REVERTED: the corpus-wide ratchet
    // (`crates/praxis-corpus-tests/tests/defines_pointers_corpus_ratchet.rs`)
    // caught a real regression on Titles 15/42/49, and a direct A/B probe
    // (`crates/praxis-corpus-tests/tests/scratch_probe.rs`,
    // `probe_prose_text_vs_plain_text_defines_pointer_delta`) found the
    // mechanism: an LRC editorial footnote is embedded INLINE inside
    // `<content>`/`<chapeau>` — often between the definiendum and "means",
    // or glued into the term's own spelling — and stripping it changes the
    // token stream `defines_pointers` sees, net NEGATIVELY across all three
    // titles (26 real cases where removal broke a working extraction vs. 13
    // where it helped). The three tests below pin the exact real witnesses
    // that regressed, verbatim as `defines_pointers` receives them TODAY
    // (`UsCodeMixed::prose_text`, which now only excludes `<table>`, never a
    // footnote) — so any future attempt to re-exclude footnotes fails these
    // fast, in milliseconds, without needing the ~66-minute full sweep to
    // notice.

    /// 15 U.S.C. § 689(8) — real text, footnote INSIDE the definiens
    /// ("means such 33 So in original. Probably should be "each". of the
    /// several States…"). Confirmed regressed: prose_text (footnote
    /// stripped) dropped this to 0 pointers; plain_text (footnote kept)
    /// found 1. `probe_prose_text_vs_plain_text_defines_pointer_delta`'s own
    /// captured evidence.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_state_definition_despite_its_embedded_so_in_original_footnote() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}State\u{201D} means such 33 So in original. \
             Probably should be \u{201C}each\u{201D}. of the several States, the \
             District of Columbia, the Commonwealth of Puerto Rico, the Virgin \
             Islands, Guam, American Samoa, the Commonwealth of the Northern \
             Mariana Islands, and any other commonwealth, territory, or \
             possession of the United States.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(
            pointers.len(),
            1,
            "regressed: the embedded editorial footnote must not block this \
             real 15 U.S.C. § 689(8) definition; got {pointers:?}"
        );
        assert_eq!(pointers[0].term, "state");
    }

    /// 42 U.S.C. § 13368(p)(6) — real text, footnote inside the definiens'
    /// adverb ("forseeably 33 So in original. Probably should be
    /// "foreseeably". be commercially worked…"). Confirmed regressed the
    /// same direction as the § 689(8) witness above.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_coal_seam_definition_despite_its_embedded_so_in_original_footnote() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}coal seam\u{201D} means any stratum of coal \
             20 inches or more in thickness, unless a stratum of less \
             thickness is being commercially worked, or can in the judgment \
             of the Secretary of the Interior forseeably 33 So in original. \
             Probably should be \u{201C}foreseeably\u{201D}. be commercially \
             worked and will require protection if wells are being drilled \
             through it.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(
            pointers.len(),
            1,
            "regressed: the embedded editorial footnote must not block this \
             real 42 U.S.C. § 13368(p)(6) definition; got {pointers:?}"
        );
        assert_eq!(pointers[0].term, "coal seam");
    }

    /// 42 U.S.C. § 10802(1) — real text, footnote inside the definiens,
    /// spanning an article ("caused, or may have caused, injury or death to
    /// a 11 So in original. Probably should be "an". individual with mental
    /// illness"). The chapeau ends in an em-dash (introducing an
    /// enumeration this test does not need), matching the real fetched
    /// text exactly.
    ///
    /// TWO baselines have now been retired on this one witness, in order.
    ///
    /// It first asserted one pointer, term "individual" — a mis-attachment
    /// its own doc declined to call correct ("what `defines_pointers`
    /// verifiably does today … not an assumed 'correct' term"). That was the
    /// fabrication class the use/mention closure
    /// ([`ADefiniendumIsMentionedNeverUsed`]) removed: the chapeau's trailing
    /// "…individual with mental illness, and includes acts such as—" resolves
    /// "includes" against a USED "individual", while the only MENTIONED
    /// expression in the sentence is "abuse". It then asserted ZERO
    /// pointers, and recorded the remaining gap by name — the chapeau-tag
    /// attachment gap that kept the real definiendum "abuse" out of reach.
    ///
    /// That gap is what the PARTIAL-PARSE goal closes: the trailing
    /// ", and includes acts such as—" tag attaches to nothing, and under the
    /// whole-string goal it discarded the complete "The term “abuse” means
    /// …" clause in front of it. The clause is now found where it sits
    /// (`AnUnattachableAdjunctNeverHidesItsClause`), so this witness asserts
    /// the real definiendum — the outcome its own previous doc named as the
    /// condition for this update.
    ///
    /// The footnote-tolerance guarantee this witness was recruited for is
    /// also still covered by the TWO witnesses that extract the right
    /// definiendum:
    /// `recognizes_the_real_state_definition_despite_its_embedded_so_in_original_footnote`
    /// and
    /// `recognizes_the_real_coal_seam_definition_despite_its_embedded_so_in_original_footnote`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_abuse_chapeau_recovers_its_real_definiendum_behind_an_unattachable_tag() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}abuse\u{201D} means any act or failure to \
             act by an employee of a facility rendering care or treatment \
             which was performed, or which was failed to be performed, \
             knowingly, recklessly, or intentionally, and which caused, or \
             may have caused, injury or death to a 11 So in original. \
             Probably should be \u{201C}an\u{201D}. individual with mental \
             illness, and includes acts such as\u{2014}";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "abuse");
    }

    /// The footnote is no longer load-bearing — and this witness is where
    /// that was measured, in both directions.
    ///
    /// It recorded a surprising, checked finding: stripping the editorial
    /// footnote did not merely change WHICH term this chapeau resolved to,
    /// it dropped the derivation to ZERO pointers (footnote-free: `[]`;
    /// footnote-present: one pointer, term "individual"). The footnote's
    /// numeral/text was not incidental noise the chart parsed around; on this
    /// real sentence it was what let the chart reach a complete two-argument
    /// `Sem::Prop` over the WHOLE span at all. That fragility was a symptom
    /// of the whole-string goal, not of the sentence: a derivation that has
    /// to span every token is hostage to every token.
    ///
    /// Under the partial-parse goal the definitional clause is found on its
    /// own, so both variants now yield the same real definiendum, and the
    /// footnote is irrelevant to the outcome rather than load-bearing for it.
    /// The sibling witness above carries the footnote-present half.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_abuse_chapeau_recovers_the_same_definiendum_without_its_footnote() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}abuse\u{201D} means any act or failure to \
             act by an employee of a facility rendering care or treatment \
             which was performed, or which was failed to be performed, \
             knowingly, recklessly, or intentionally, and which caused, or \
             may have caused, injury or death to an individual with mental \
             illness, and includes acts such as\u{2014}";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "abuse");
    }

    /// KNOWN GAP, documented not fixed: `dangling_chapeau_reassembly_
    /// index`'s "ALL-joined" candidate (chapeau + every child's prose,
    /// `bridge.rs`'s `candidates.push(format!("{chapeau} {}",
    /// child_texts.join(" ")))`) can glue a "shall be—" requirements
    /// enumeration into a shape the chart spuriously reduces to a complete
    /// two-argument `Sem::Prop`, producing a FALSE-POSITIVE pointer for a
    /// function/participle word that is never actually a defined term.
    ///
    /// Real witness (42 U.S.C. § 6833(a)(2), fetched verbatim via
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`'s
    /// `probe_all_joined_based_false_positive_full_text`, no chart-parse
    /// needed to fetch it): the ALL-joined candidate for this node is "The
    /// determination referred to in paragraph (1) shall be— made after
    /// public notice and hearing; in writing; based upon findings included
    /// in such determination and upon the evidence presented at the
    /// hearing; and available to the public." — an enumeration of
    /// requirements for "the determination," never a "the term X means Y"
    /// declarative anywhere in it. `defines_pointers` currently still
    /// extracts one pointer, term "based" — confirmed via
    /// `probe_all_joined_candidate_ever_uniquely_contributes` (this
    /// session), which found this same function/participle-word false-
    /// positive pattern ("and", "or", "a", "can", "may", "based", …)
    /// repeating across 175+ real ALL-joined candidates corpus-wide, a
    /// SEPARATE mechanism from the footnote-exclusion regression this
    /// module's other tests guard.
    ///
    /// FIXED as a side effect of the N-ary-coordination chart-parsing bug
    /// fix (Title-1 "State"/"appropriate department or agency" defines-lens
    /// audit): `defines_pointers` no longer extracts a pointer for this
    /// candidate. The winning derivation before that fix reached a
    /// spurious complete `Sem::Prop` by using "based" as a degenerate
    /// two-argument predicate; the new alternative Lambek readings this
    /// fix adds (`svo::transitive_verb_coordinator`,
    /// `svo::transitive_verb_particle` — offered to every "and"/"or" and
    /// every preposition respectively) change which derivation the
    /// Viterbi chart prefers for this "shall be— ... ; based upon ...; and
    /// available ..." enumeration, and no derivation now completes at all
    /// — the correct, honest "no coverage → no pointer" outcome for a
    /// requirements enumeration that is never a "the term X means Y"
    /// declarative. Confirmed via direct re-run (this session): `got []`,
    /// not a different wrong pointer. Kept as a REGRESSION GUARD per this
    /// test's own original instruction ("if this now returns 0, the false
    /// positive is fixed — replace this test's assertion... rather than
    /// deleting it, so a REGRESSION back to the false positive would still
    /// be caught") — updated, not deleted.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_all_joined_requirements_enumeration_no_longer_yields_the_false_positive_based_pointer()
     {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The determination referred to in paragraph (1) shall be\u{2014} \
             made after public notice and hearing; in writing; based upon \
             findings included in such determination and upon the evidence \
             presented at the hearing; and available to the public.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert!(
            pointers.is_empty(),
            "if this now returns a NON-empty result again, the false positive \
             is BACK — a regression in the N-ary-coordination chart-parsing \
             fix's cost ranking; got {pointers:?}"
        );
    }

    /// Root cause B regression: the REAL 42 U.S.C. § 1395l(a)(1) Medicare
    /// Part B payment-formula enumeration — subparagraphs (A) through (HH),
    /// 34 letters, a single flat comma-coordinated clause with only 2
    /// semicolons and 6 periods across ~2500 words — verbatim, fetched the
    /// same way as the table fixture above (`temp_dump_title42_candidate_
    /// text_for_regression_fixtures`).
    ///
    /// This construction is a structural sibling of the already-fixed
    /// joint-stock-company N-ary "or" coordination bug
    /// (`recognizes_the_real_company_definition_combining_the_whether_
    /// adjunct_and_n_ary_or_coordination_fixes`, above) at roughly an order
    /// of magnitude larger arity, and — direct instrumentation this session
    /// found (`crates/praxis-corpus-tests/tests/scratch_probe.rs`
    /// `probe_bisect_title42_pathological_candidates`) — its real cost was
    /// NEVER the chart-parse step (`DEFINES_MAX_CHART_WIDTH` = 512 already
    /// rejects a span this size near-instantly) but `tokenize_with_
    /// alternatives`'s own preprocessing: `multiword_surface_spans`
    /// (reached via `correct_unknown_word_surfaces`) tried every window
    /// length from 2 up to the FULL remaining sentence for every start
    /// position — O(n²) window checks, each paying its own O(window)
    /// string allocation, so O(n³) worst case — BEFORE the width bound was
    /// ever reached. Bounding that search by `Language::
    /// max_known_surface_words()` (measured 11 for the loaded English
    /// WordNet) `.max(registry_max_surface_words)` (real data, not a
    /// hand-picked constant) turned this from timing out past 60s into
    /// single-digit seconds.
    ///
    /// No pointer is the CORRECT, honest answer (a payment-formula clause,
    /// no "means"/"includes" verb anywhere) — this test's real assertion is
    /// that it completes at all, and quickly, not that it extracts
    /// anything: this specific 2500-word single-clause enumeration is
    /// documented, not silently dropped, as the one Title 42 shape this
    /// session's fixes make FAST but do not make COVERABLE (see this
    /// investigation's own summary for why: no grammar extension recovers a
    /// definition from a sentence containing no "means"/"includes" verb).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_medicare_payment_formula_enumeration_completes_fast() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "in the case of services described in section 1395k(a)(1) of \
             this title\u{2014}80 percent of the reasonable charges for the \
             services; except that (A) an organization which provides medical \
             and other health services (or arranges for their availability) on \
             a prepayment basis (and either is sponsored by a union or \
             employer, or does not provide, or arrange for the provision of, \
             any inpatient hospital services) may elect to be paid 80 percent \
             of the reasonable cost of services for which payment may be made \
             under this part on behalf of individuals enrolled in such \
             organization in lieu of 80 percent of the reasonable charges for \
             such services if the organization undertakes to charge such \
             individuals no more than 20 percent of such reasonable cost plus \
             any amounts payable by them as a result of subsection (b), (B) \
             with respect to items and services described in section \
             1395x(s)(10)(A) of this title, the amounts paid shall be 100 \
             percent of the reasonable charges for such items and services, \
             (C) with respect to expenses incurred for those physicians\u{2019} \
             services for which payment may be made under this part that are \
             described in section 1395y(a)(4) of this title, the amounts paid \
             shall be subject to such limitations as may be prescribed by \
             regulations, (D) with respect to clinical diagnostic laboratory \
             tests for which payment is made under this part (i)(I) on the \
             basis of a fee schedule under subsection (h)(1) (for tests \
             furnished before January 1, 2017) or subsection (d)(1) or (aa) of \
             section 1395m of this title, the amount paid shall be equal to 80 \
             percent (or 100 percent, in the case of such tests for which \
             payment is made on an assignment-related basis) of the lesser of \
             the amount determined under such fee schedule, the limitation \
             amount for that test determined under subsection (h)(4)(B), or \
             the amount of the charges billed for the tests, or (II) under \
             section 1395m\u{2013}1 of this title (for tests furnished on or \
             after January 1, 2017), the amount paid shall be equal to 80 \
             percent (or 100 percent, in the case of such tests for which \
             payment is made on an assignment-related basis) of the lesser of \
             the amount determined under such section or the amount of the \
             charges billed for the tests, or (ii) for tests furnished before \
             January 1, 2017, on the basis of a negotiated rate established \
             under subsection (h)(6), the amount paid shall be equal to 100 \
             percent of such negotiated rate,, (E) with respect to services \
             furnished to individuals who have been determined to have end \
             stage renal disease, the amounts paid shall be determined subject \
             to the provisions of section 1395rr of this title, (F) with \
             respect to clinical social worker services under section \
             1395x(s)(2)(N) of this title, the amounts paid shall be 80 \
             percent of the lesser of (i) the actual charge for the services \
             or (ii) 75 percent of the amount determined for payment of a \
             psychologist under clause (L), (G) with respect to facility \
             services furnished in connection with a surgical procedure \
             specified pursuant to subsection (i)(1)(A) and furnished to an \
             individual in an ambulatory surgical center described in such \
             subsection, for services furnished beginning with the \
             implementation date of a revised payment system for such \
             services in such facilities specified in subsection (i)(2)(D), \
             the amounts paid, subject to subsection (i)(9), shall be 80 \
             percent of the lesser of the actual charge for the services or \
             the amount determined by the Secretary under such revised \
             payment system, (H) with respect to services of a certified \
             registered nurse anesthetist under section 1395x(s)(11) of this \
             title, the amounts paid shall be 80 percent of the least of the \
             actual charge, the prevailing charge that would be recognized \
             (or, for services furnished on or after January 1, 1992, the fee \
             schedule amount provided under section 1395w\u{2013}4 of this \
             title) if the services had been performed by an \
             anesthesiologist, or the fee schedule for such services \
             established by the Secretary in accordance with subsection (l), \
             (I) with respect to covered items (described in section \
             1395m(a)(13) of this title), the amounts paid shall be the \
             amounts described in section 1395m(a)(1) of this title, and (J) \
             with respect to expenses incurred for radiologist services (as \
             defined in section 1395m(b)(6) of this title), subject to \
             section 1395w\u{2013}4 of this title, the amounts paid shall be \
             80 percent of the lesser of the actual charge for the services \
             or the amount provided under the fee schedule established under \
             section 1395m(b) of this title, (K) with respect to certified \
             nurse-midwife services under section 1395x(s)(2)(L) of this \
             title, the amounts paid shall be 80 percent of the lesser of the \
             actual charge for the services or the amount determined by a fee \
             schedule established by the Secretary for the purposes of this \
             subparagraph (but in no event shall such fee schedule exceed 65 \
             percent of the prevailing charge that would be allowed for the \
             same service performed by a physician, or, for services \
             furnished on or after January 1, 1992, 65 percent (or 100 \
             percent for services furnished on or after January 1, 2011) of \
             the fee schedule amount provided under section 1395w\u{2013}4 of \
             this title for the same service performed by a physician), (L) \
             with respect to qualified psychologist services under section \
             1395x(s)(2)(M) of this title, the amounts paid shall be 80 \
             percent of the lesser of the actual charge for the services or \
             the amount determined by a fee schedule established by the \
             Secretary for the purposes of this subparagraph, (M) with \
             respect to prosthetic devices and orthotics and prosthetics (as \
             defined in section 1395m(h)(4) of this title), the amounts paid \
             shall be the amounts described in section 1395m(h)(1) of this \
             title, (N) with respect to expenses incurred for physicians\u{2019} \
             services (as defined in section 1395w\u{2013}4(j)(3) of this \
             title) other than personalized prevention plan services (as \
             defined in section 1395x(hhh)(1) of this title), the amounts \
             paid shall be 80 percent of the payment basis determined under \
             section 1395w\u{2013}4(a)(1) of this title, (O) with respect to \
             services described in section 1395x(s)(2)(K) of this title \
             (relating to services furnished by physician assistants, nurse \
             practitioners, or clinic nurse specialists), the amounts paid \
             shall be equal to 80 percent of (i) the lesser of the actual \
             charge or 85 percent of the fee schedule amount provided under \
             section 1395w\u{2013}4 of this title, or (ii) in the case of \
             services as an assistant at surgery, the lesser of the actual \
             charge or 85 percent of the amount that would otherwise be \
             recognized if performed by a physician who is serving as an \
             assistant at surgery, (P) with respect to surgical dressings, \
             the amounts paid shall be the amounts determined under section \
             1395m(i) of this title, (Q) with respect to items or services \
             for which fee schedules are established pursuant to section \
             1395u(s) of this title, the amounts paid shall be 80 percent of \
             the lesser of the actual charge or the fee schedule established \
             in such section, (R) with respect to ambulance services, (i) the \
             amounts paid shall be 80 percent of the lesser of the actual \
             charge for the services or the amount determined by a fee \
             schedule established by the Secretary under section 1395m(l) of \
             this title and (ii) with respect to ambulance services described \
             in section 1395m(l)(8) of this title, the amounts paid shall be \
             the amounts determined under section 1395m(g) of this title for \
             outpatient critical access hospital services, (S)(i) except as \
             provided in clause (ii), subject to subparagraph (EE), with \
             respect to drugs and biologicals (including intravenous immune \
             globulin (as defined in section 1395x(zz) of this title)) not \
             paid on a cost or prospective payment basis as otherwise \
             provided in this part (other than items and services described \
             in subparagraph (B)), the amounts paid shall be 80 percent of \
             the lesser of the actual charge or the payment amount \
             established in section 1395u(o) of this title (or, if \
             applicable, under section 1395w\u{2013}3, 1395w\u{2013}3a, or \
             1395w\u{2013}3b of this title), and (ii) with respect to insulin \
             furnished on or after July 1, 2023, through an item of durable \
             medical equipment covered under section 1395x(n) of this title, \
             the amounts paid shall be, subject to the fourth sentence of \
             this subsection, 80 percent of the payment amount established \
             under section 1395w\u{2013}3a of this title (or section \
             1395w\u{2013}3b of this title, if applicable) for such insulin, \
             (T) with respect to medical nutrition therapy services (as \
             defined in section 1395x(vv) of this title), the amount paid \
             shall be 80 percent (or 100 percent if such services are \
             recommended with a grade of A or B by the United States \
             Preventive Services Task Force for any indication or population \
             and are appropriate for the individual) of the lesser of the \
             actual charge for the services or 85 percent of the amount \
             determined under the fee schedule established under section \
             1395w\u{2013}4(b) of this title for the same services if \
             furnished by a physician, (U) with respect to facility fees \
             described in section 1395m(m)(2)(B) of this title, the amounts \
             paid shall be 80 percent of the lesser of the actual charge or \
             the amounts specified in such section, (V) notwithstanding \
             subparagraphs (I) (relating to durable medical equipment), (M) \
             (relating to prosthetic devices and orthotics and prosthetics), \
             and (Q) (relating to 1395u(s) items), with respect to \
             competitively priced items and services (described in section \
             1395w\u{2013}3(a)(2) of this title) that are furnished in a \
             competitive area, the amounts paid shall be the amounts \
             described in section 1395w\u{2013}3(b)(5) of this title, (W) \
             with respect to additional preventive services (as defined in \
             section 1395x(ddd)(1) of this title), the amount paid shall be \
             (i) in the case of such services which are clinical diagnostic \
             laboratory tests, the amount determined under subparagraph (D) \
             (if such subparagraph were applied, by substituting \u{201C}100 \
             percent\u{201D} for \u{201C}80 percent\u{201D}), and (ii) in the \
             case of all other such services, 100 percent of the lesser of \
             the actual charge for the service or the amount determined \
             under a fee schedule established by the Secretary for purposes \
             of this subparagraph, (X) with respect to personalized \
             prevention plan services (as defined in section 1395x(hhh)(1) \
             of this title), the amount paid shall be 100 percent of the \
             lesser of the actual charge for the services or the amount \
             determined under the payment basis determined under section \
             1395w\u{2013}4 of this title, (Y) subject to subsection (dd), \
             with respect to preventive services described in subparagraphs \
             (A) and (B) of section 1395x(ddd)(3) of this title that are \
             appropriate for the individual and, in the case of such \
             services described in subparagraph (A), are recommended with a \
             grade of A or B by the United States Preventive Services Task \
             Force for any indication or population, the amount paid shall \
             be 100 percent of (i) except as provided in clause (ii), the \
             lesser of the actual charge for the services or the amount \
             determined under the fee schedule that applies to such services \
             under this part, and (ii) in the case of such services that are \
             covered OPD services (as defined in subsection (t)(1)(B)), the \
             amount determined under subsection (t), (Z) with respect to \
             Federally qualified health center services for which payment is \
             made under section 1395m(o) of this title, the amounts paid \
             shall be 80 percent of the lesser of the actual charge or the \
             amount determined under such section, (AA) with respect to an \
             applicable disposable device (as defined in paragraph (2) of \
             section 1395m(s) of this title) furnished to an individual \
             pursuant to paragraph (1) of such section, the amount paid \
             shall be equal to 80 percent of the lesser of the actual charge \
             or the amount determined under paragraph (3) of such section, \
             (BB) with respect to home infusion therapy, the amount paid \
             shall be an amount equal to 80 percent of the lesser of the \
             actual charge for the services or the amount determined under \
             section 1395m(u) of this title, (CC) with respect to opioid use \
             disorder treatment services furnished during an episode of \
             care, the amount paid shall be equal to the amount payable \
             under section 1395m(w) of this title less any copayment \
             required as specified by the Secretary, (DD) with respect to a \
             specified COVID\u{2013}19 testing-related service described in \
             paragraph (1) of subsection (cc) for which payment may be made \
             under a specified outpatient payment provision described in \
             paragraph (2) of such subsection, the amounts paid shall be 100 \
             percent of the payment amount otherwise recognized under such \
             respective specified outpatient payment provision for such \
             service, (EE) with respect to a part B rebatable drug (as \
             defined in paragraph (2) of section 1395w\u{2013}3a(i) of this \
             title) furnished on or after April 1, 2023, for which the \
             payment amount for a calendar quarter under paragraph \
             (3)(A)(ii)(I) of such section (or, in the case of a part B \
             rebatable drug that is a selected drug (as defined in section \
             1320f\u{2013}1(c) of this title for which, the payment amount \
             described in section 1395w\u{2013}3a(b)(1)(B) of this title) \
             for such drug for such quarter exceeds the inflation-adjusted \
             payment under paragraph (3)(A)(ii)(II) of such section for such \
             quarter, the amounts paid shall be equal to the percent of the \
             payment amount under paragraph (3)(A)(ii)(I) of such section or \
             section 1395w\u{2013}3a(b)(1)(B) of this title, as applicable, \
             that equals the difference between (i) 100 percent, and (ii) \
             the percent applied under section 1395w\u{2013}3a(i)(5)(B) of \
             this title (FF) with respect to marriage and family therapist \
             services and mental health counselor services under section \
             1395x(s)(2)(II) of this title, the amounts paid shall be 80 \
             percent of the lesser of the actual charge for the services or \
             75 percent of the amount determined for payment of a \
             psychologist under subparagraph (L), (GG) with respect to \
             lymphedema compression treatment items (as defined in section \
             1395x(mmm) of this title), the amount paid shall be equal to 80 \
             percent of the lesser of the actual charge or the amount \
             determined under the payment basis determined under section \
             1395m(z) of this title, and (HH) with respect to items and \
             services related to the administration of intravenous immune \
             globulin furnished on or after January 1, 2024, as described in \
             section 1395x(zz) of this title, the amounts paid shall be the \
             lesser of the 80 percent of the actual charge or the payment \
             amount established under section 1395u(o)(8) of this title;";
        let started = std::time::Instant::now();
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        let elapsed = started.elapsed();
        assert!(
            pointers.is_empty(),
            "a payment-formula clause carries no \"means\"/\"includes\" \
             declarative; got {pointers:?}"
        );
        assert!(
            elapsed.as_secs_f64() < 30.0,
            "regressed: 34-subparagraph enumeration took {:.3}s (was ~10s \
             post-fix, timed out past 60s pre-fix)",
            elapsed.as_secs_f64()
        );
    }

    /// PRECISION baseline: a coordinated SUBJECT list must not be swept
    /// into a bogus medial supplement (the false-positive risk
    /// `collapse_medial_comma_adjuncts`'s own doc comment names) — a
    /// grammar-level regression check mirroring
    /// `a_real_coordinated_definiendum_sample_yields_no_pointer`'s own
    /// "still honestly no coverage" discipline, here for a DIFFERENT
    /// coordination shape (subject-side, not the definiendum itself).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_coordinated_subject_is_not_mistaken_for_a_medial_supplement() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D}, the county, and the state \
             cover this benefit.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert!(
            pointers.is_empty(),
            "a coordinated subject list is not a supplement bracket; got {pointers:?}"
        );
    }

    /// UPDATED BY G7 (a genuine surprise, re-measured, not assumed): G1's
    /// own report predicted this REAL, UNMODIFIED sentence would still fail
    /// to parse — its own internal bare-plural NP object ("purposes of this
    /// subsection") needing a separate, not-yet-built bare-NP promotion
    /// capability. G4 (a LATER stage in this same pipeline) built exactly
    /// such a promotion for `defines_pointers`'s OWN scoped chart table
    /// (`definiens_cost_table`'s `bare_noun_phrase_unary_rule`, for a
    /// SEPARATE definiens-side gap) — and it turns out to ALSO cover this
    /// sentence's subject-side bare plural, closing G1's own documented
    /// residual gap as an unplanned side effect. Because "payment unit" is
    /// ITSELF not a WordNet-indexed compound (a G7 case), this had stayed
    /// masked at the OLD unconditional-drop written-form floor until G7
    /// stopped dropping it — re-verified here, not assumed from the above
    /// reasoning alone.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_for_purposes_of_this_subsection_sentence_now_mints_its_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "For purposes of this subsection, the term \u{201C}payment unit\u{201D} \
                     means a discharge.";
        let domain = mint_domain();
        let pointers = defines_pointers(text, en, en, vn, &domain);
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "payment unit");
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "\"payment unit\" is not a WordNet-indexed compound headword, so it \
             is MINTED rather than dropped — got {:?}",
            pointers[0].target
        );
    }

    /// END-TO-END: a produced `defines` pointer resolves (via the runtime
    /// `AtomResolver`) into the definiendum's `ontolex:Form` atom in
    /// `english_wordnet` — the SAME target kind `denotes` resolves to (a
    /// written form, never a sense). Mirrors
    /// `a_produced_pointer_resolves_to_a_form_atom` exactly.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_produced_defines_pointer_resolves_to_a_form_atom() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointer = defines_pointers(
            "The term \u{201C}consumer\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        )
        .into_iter()
        .next()
        .expect("the sentence defines a term");

        let archive = project_archive_with_forms(en);
        let english_root = archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "defines".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();

        let resolved = resolver
            .resolve(&pointer.target)
            .expect("the produced defines pointer resolves by content address");
        assert_eq!(
            resolved.kind, FORM_KIND,
            "the defines pointer resolves to an ontolex:Form, never a sense"
        );
        assert_eq!(resolved.name, "consumer");
    }

    /// THE GENERIC LOOP, defines edition: a content archive grounds via
    /// `ground(defines_lens(...))`, and the minted edge resolves through the
    /// GENERIC `AtomResolver` — the verbatim shape of
    /// `a_content_archive_grounds_via_the_lens_and_resolves_to_forms`,
    /// proving `defines` really is "another lens of the same shape" as
    /// `denotes`/`cites`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_content_archive_grounds_via_the_defines_lens_and_resolves() {
        use pr4xis_runtime::grounding::ground;

        let en = english_loaded();
        let vn = verbnet_classes_loaded();

        let content = Archive {
            nodes: alloc::vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t15/s6603/h/6/A".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some(
                    "The term \u{201C}consumer\u{201D} means a natural person.".to_string()
                ),
            }],
            connections: alloc::vec![],
        };

        let grounded = ground(
            &content,
            defines_lens(
                en,
                en,
                vn,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &mint_domain(),
            ),
        )
        .expect("the defines lens grounds");
        let provision = &grounded.nodes[0];
        let (kind, target) = provision
            .edges
            .iter()
            .find(|(k, _)| k == DEFINES_REL)
            .expect("the definiendum grounded");
        assert_eq!(kind, DEFINES_REL);

        let archive = project_archive_with_forms(en);
        let english_root = archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "defines".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(target)
            .expect("the grounded definiendum resolves");
        assert_eq!(resolved.name, "consumer");
    }

    /// `denotes` and `defines` COEXIST on the same defining provision —
    /// composing the two lenses independently via `ground`'s multi-lens
    /// support (each call only ADDS edges, never replaces): the provision
    /// carries both a `denotes` pointer for every content word AND a
    /// `defines` pointer for its own definiendum, proving the lenses are
    /// genuinely independent producers over the same substrate.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn denotes_and_defines_pointers_coexist_on_the_same_provision() {
        use pr4xis_runtime::grounding::ground;

        let en = english_loaded();
        let vn = verbnet_classes_loaded();

        let content = Archive {
            nodes: alloc::vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t15/s6603/h/6/A".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some(
                    "The term \u{201C}consumer\u{201D} means a natural person.".to_string()
                ),
            }],
            connections: alloc::vec![],
        };

        let denotes_grounded =
            ground(&content, denotes_lens(en)).expect("the denotes lens grounds");
        let both_grounded = ground(
            &denotes_grounded,
            defines_lens(
                en,
                en,
                vn,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &mint_domain(),
            ),
        )
        .expect("the defines lens grounds on top of the denotes-grounded archive");

        let provision = &both_grounded.nodes[0];
        let has_denotes = provision.edges.iter().any(|(k, _)| k == "denotes");
        let has_defines = provision.edges.iter().any(|(k, _)| k == DEFINES_REL);
        assert!(has_denotes, "the provision still carries denotes edges");
        assert!(
            has_defines,
            "the provision ALSO carries a defines edge, added independently"
        );
    }

    // ---- S1: the heading-shadowing fix (`shadowed_prose`) ----

    /// BEFORE the S1 fix: a subdivision whose `lexical` the projection set to
    /// its HEADING (`project_subdivision`'s `heading.or(chapeau).or(content)`,
    /// `uslm::corpus::bridge`) never reaches `defines_pointers` at all — a
    /// heading like "Respite care" (the REAL heading text of 42 U.S.C.
    /// § 300ii(7), byte-verified against
    /// `usc_title_42-pl-119-90.xml`) carries no verb, so it can never match
    /// the "means" shape regardless of what the grammar can otherwise derive.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_heading_alone_yields_no_pointer_with_no_shadowed_prose_supplied() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let heading_only = Definition {
            kind: "subdivision".to_string(),
            name: "/us/usc/t42/s300ii/7".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: Some("Respite care".to_string()),
        };
        let pointers = defines_pointers(
            heading_only.lexical.as_deref().unwrap(),
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert!(
            pointers.is_empty(),
            "a bare heading carries no verb; got {pointers:?}"
        );
    }

    /// AFTER the S1 fix: the SAME heading-shadowed node grounds its
    /// definiendum once `defines_lens` is ALSO given the shadowed prose
    /// (`shadowed_prose`, keyed by the node's own URN) — even though its
    /// OWN `lexical` is still nothing but the heading. The prose reused here
    /// is the SAME already-proven-grammar-complete real statutory sentence
    /// `a_content_archive_grounds_via_the_defines_lens_and_resolves` already
    /// grounds (real text, `/us/usc/t15/s6603/h/6/A`) — this test isolates
    /// the PROJECTION/side-channel fix (S1) from the SEPARATE, still-open
    /// grammar gaps (G1–G7) a real shadowed caregiving definition (like the
    /// 300ii(7) respite-care sentence this fixture's URN and heading are
    /// drawn from) would also have to clear.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn defines_lens_recovers_a_definition_shadowed_by_a_heading() {
        use pr4xis_runtime::grounding::ground;

        let en = english_loaded();
        let vn = verbnet_classes_loaded();

        let shadowed = Definition {
            kind: "subdivision".to_string(),
            name: "/us/usc/t42/s300ii/7".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            // The REAL heading of 42 U.S.C. § 300ii(7) — no "means" verb, so
            // `lexical` alone (checked above) grounds nothing.
            lexical: Some("Respite care".to_string()),
        };
        let content = Archive {
            nodes: alloc::vec![shadowed],
            connections: alloc::vec![],
        };

        let mut shadowed_prose = BTreeMap::new();
        shadowed_prose.insert(
            "/us/usc/t42/s300ii/7".to_string(),
            "The term \u{201C}consumer\u{201D} means a natural person.".to_string(),
        );

        let grounded = ground(
            &content,
            defines_lens(
                en,
                en,
                vn,
                &shadowed_prose,
                &BTreeMap::new(),
                &mint_domain(),
            ),
        )
        .expect("the defines lens grounds via the shadowed-prose side-channel");
        let (kind, target) = grounded.nodes[0]
            .edges
            .iter()
            .find(|(k, _)| k == DEFINES_REL)
            .expect("the shadowed node's definiendum grounded via shadowed_prose");
        assert_eq!(kind, DEFINES_REL);

        let archive = project_archive_with_forms(en);
        let english_root = archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "defines".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(target)
            .expect("the recovered defines pointer resolves by content address");
        assert_eq!(resolved.name, "consumer");
    }

    /// A node ABSENT from `shadowed_prose` (the overwhelming majority —
    /// everything not heading-shadowed) behaves EXACTLY as before the S1
    /// fix — an empty table changes nothing.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_empty_shadowed_prose_table_does_not_change_unshadowed_behavior() {
        use pr4xis_runtime::grounding::ground;

        let en = english_loaded();
        let vn = verbnet_classes_loaded();

        let content = Archive {
            nodes: alloc::vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t15/s6603/h/6/A".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some(
                    "The term \u{201C}consumer\u{201D} means a natural person.".to_string()
                ),
            }],
            connections: alloc::vec![],
        };
        let grounded = ground(
            &content,
            defines_lens(
                en,
                en,
                vn,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &mint_domain(),
            ),
        )
        .expect("the defines lens grounds from lexical alone, table absent");
        assert!(
            grounded.nodes[0]
                .edges
                .iter()
                .any(|(k, _)| k == DEFINES_REL),
            "an unshadowed node's own `lexical` still grounds with no entry in the table"
        );
    }

    // ---- TASK #18 (G5): coordinated/plural definienda ----
    //
    // Two variants, both real report-cited constructions:
    // (A) "the terms 'X' and 'Y' mean ..." — a coordinated APPOSITION set
    //     modifying a plural "the terms" (`svo::nominal_coordinator_apposition`);
    // (B) "the term 'X' and the term 'Y' mean ..." — a coordinated FULL-NP
    //     subject, bridged onto the SAME marker mechanism by eliding the
    //     repeated "the term(s)" (`tokenize::skip_the_term_prefix`).
    // Isolated on the proven "consumer"/"individual" base declaratives
    // FIRST (the SAME isolation precedent G1-G4 establish), then checked
    // against the REAL report sentences.

    /// Variant (A), isolated: two distinct WordNet-known single-word
    /// definienda coordinated under one plural "the terms".
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_terms_x_and_y_mean_z_coordinated_apposition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The terms \u{201C}consumer\u{201D} and \u{201C}individual\u{201D} mean \
             a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        let terms: alloc::collections::BTreeSet<&str> =
            pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            alloc::collections::BTreeSet::from(["consumer", "individual"]),
            "got {pointers:?}"
        );
        assert!(
            pointers
                .iter()
                .all(|p| p.exhaustiveness == DefinitionExhaustiveness::Exhaustive),
            "both coordinated terms share the ONE declarative's exhaustiveness verdict; \
             got {pointers:?}"
        );
    }

    /// Variant (A), "or" instead of "and" — proving the closed class, not
    /// just one member of it, using the REAL report-cited pair (42
    /// U.S.C. § 289b–1(f)(2)'s "assistance", G4's own proven single-word
    /// coordination-mechanism definiendum, paired with "consumer").
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_terms_x_or_y_mean_z_coordinated_apposition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The terms \u{201C}consumer\u{201D} or \u{201C}assistance\u{201D} mean \
             a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        let terms: alloc::collections::BTreeSet<&str> =
            pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            alloc::collections::BTreeSet::from(["consumer", "assistance"]),
            "got {pointers:?}"
        );
    }

    /// Variant (A), the REAL report-cited pair — 15 U.S.C. § 70(b)'s
    /// "fiber"/"textile fiber" (the SAME real definienda
    /// `a_real_coordinated_definiendum_sample_yields_no_pointer`'s own
    /// unmodified sentence carries), recombined with the proven "means a
    /// natural person" definiens to isolate the COORDINATION mechanism
    /// from that sentence's SEPARATE, still-open G4 relative-clause gap.
    /// The MECHANISM extracts BOTH coordinated candidates ("fiber" AND
    /// "textile fiber") — confirmed directly against
    /// `montague::interpret`'s own output (both survive as `Sem::Concept`s
    /// in the coordinated `Sem::Func`, matching
    /// `coordinated_close_apposition_definienda_drop_the_terms_and_keep_both_concepts`'s
    /// unit proof). UPDATED BY G7: "fiber" is a genuine WordNet headword and
    /// grounds into `english_wordnet`; "textile fiber" is NOT WordNet-indexed
    /// as a compound headword, so (since G7) it is MINTED into the
    /// statute-local domain instead of being dropped — BOTH conjuncts now
    /// produce a pointer, each at a DIFFERENT target ontology.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_terms_fiber_or_textile_fiber_mean_a_natural_person_isolated() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let domain = mint_domain();
        let pointers = defines_pointers(
            "The term \u{201C}fiber\u{201D} or \u{201C}textile fiber\u{201D} means \
             a natural person.",
            en,
            en,
            vn,
            &domain,
        );
        let terms: alloc::collections::BTreeSet<&str> =
            pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            alloc::collections::BTreeSet::from(["fiber", "textile fiber"]),
            "the COORDINATION mechanism finds both candidates, and (since G7) \
             neither is dropped for being out of WordNet's coverage — got {pointers:?}"
        );
        let fiber = pointers.iter().find(|p| p.term == "fiber").unwrap();
        let textile_fiber = pointers.iter().find(|p| p.term == "textile fiber").unwrap();
        assert!(
            matches!(
                &fiber.target,
                EdgeTarget::Grounded { ontology, .. } if ontology == ENGLISH_ONTOLOGY
            ),
            "\"fiber\" IS a WordNet headword — got {:?}",
            fiber.target
        );
        assert!(
            matches!(
                &textile_fiber.target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "\"textile fiber\" is NOT a WordNet-indexed compound headword, so it \
             is MINTED — got {:?}",
            textile_fiber.target
        );
    }

    /// Variant (B), isolated: "the term 'X' and the term 'Y' mean ..." —
    /// the repeated "the term" prefix bridged
    /// (`tokenize::skip_the_term_prefix`) rather than a direct quote-to-
    /// quote adjacency.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_and_the_term_y_mean_z_coordinated_full_np() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D} and the term \u{201C}individual\u{201D} \
             mean a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        let terms: alloc::collections::BTreeSet<&str> =
            pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            alloc::collections::BTreeSet::from(["consumer", "individual"]),
            "got {pointers:?}"
        );
    }

    /// Variant (B), the REAL report-cited pair — 42 U.S.C. §
    /// 1395x(aa)(5)(A) "physician assistant"/"nurse practitioner",
    /// recombined with the proven definiens for the SAME isolation reason.
    /// The MECHANISM extracts BOTH coordinated candidates. UPDATED BY G7:
    /// "nurse practitioner" clears the written-form floor and grounds into
    /// `english_wordnet`; WordNet does NOT index "physician assistant" as a
    /// compound headword (the SAME G7 gap the "fiber"/"textile fiber"
    /// isolation test above measures for the OTHER real report-cited pair),
    /// so (since G7) it is MINTED into the statute-local domain instead —
    /// BOTH conjuncts now produce a pointer, each at a DIFFERENT target
    /// ontology.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_physician_assistant_and_the_term_nurse_practitioner_isolated() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let domain = mint_domain();
        let pointers = defines_pointers(
            "The term \u{201C}physician assistant\u{201D} and the term \
             \u{201C}nurse practitioner\u{201D} mean a natural person.",
            en,
            en,
            vn,
            &domain,
        );
        let terms: alloc::collections::BTreeSet<&str> =
            pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            alloc::collections::BTreeSet::from(["nurse practitioner", "physician assistant"]),
            "the COORDINATION mechanism finds both candidates, and (since G7) \
             neither is dropped for being out of WordNet's coverage — got {pointers:?}"
        );
        let nurse_practitioner = pointers
            .iter()
            .find(|p| p.term == "nurse practitioner")
            .unwrap();
        let physician_assistant = pointers
            .iter()
            .find(|p| p.term == "physician assistant")
            .unwrap();
        assert!(
            matches!(
                &nurse_practitioner.target,
                EdgeTarget::Grounded { ontology, .. } if ontology == ENGLISH_ONTOLOGY
            ),
            "\"nurse practitioner\" IS a WordNet headword — got {:?}",
            nurse_practitioner.target
        );
        assert!(
            matches!(
                &physician_assistant.target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "\"physician assistant\" is NOT a WordNet-indexed compound headword, \
             so it is MINTED — got {:?}",
            physician_assistant.target
        );
    }

    /// HONEST BASELINE: the REAL, UNMODIFIED § 3002(18)(A) sentence
    /// (byte-verified against `usc_title_42-pl-119-90.xml`): "The terms
    /// 'exploitation' and 'financial exploitation' mean the fraudulent or
    /// otherwise illegal, unauthorized, or improper act or process of an
    /// individual, including a caregiver or fiduciary, that uses the
    /// resources of an older individual for monetary or personal
    /// benefit, profit, or gain, or that results in depriving an older
    /// individual of rightful access to, or use of, benefits, resources,
    /// belongings, or assets." Coordination stopped being the blocker with
    /// the isolated variant-(A) tests above; what remained was its OWN
    /// definiens complexity beyond G4's scope (an "including X" appositive
    /// parenthetical, and a relative clause "that uses..."). Both are
    /// definiens-internal attachment gaps, and both stopped being fatal under
    /// the partial-parse goal
    /// (`AnUnattachableAdjunctNeverHidesItsClause`) — the appositive and
    /// relative-clause capabilities themselves are still unbuilt.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_exploitation_sentence_recovers_both_definienda() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The terms \u{201C}exploitation\u{201D} and \u{201C}financial \
                     exploitation\u{201D} mean the fraudulent or otherwise illegal, \
                     unauthorized, or improper act or process of an individual, \
                     including a caregiver or fiduciary, that uses the resources of \
                     an older individual for monetary or personal benefit, profit, or \
                     gain, or that results in depriving an older individual of \
                     rightful access to, or use of, benefits, resources, belongings, \
                     or assets.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        let terms: BTreeSet<&str> = pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            ["exploitation", "financial exploitation"]
                .into_iter()
                .collect(),
            "got {pointers:?}"
        );
    }

    /// HONEST BASELINE: the REAL, UNMODIFIED § 1395x(aa)(5)(A) sentence
    /// (byte-verified against `usc_title_42-pl-119-90.xml`): "The term
    /// 'physician assistant' and the term 'nurse practitioner' mean, for
    /// purposes of this subchapter, a physician assistant or nurse
    /// practitioner who performs..." — coordination stopped being the
    /// blocker with the isolated variant-(B) test above; what remained was
    /// its own definiens, a long relative-clause chain with nested
    /// PP/coordination structure G4 does not scaffold. Definiens-internal,
    /// so no longer fatal under the partial-parse goal
    /// (`AnUnattachableAdjunctNeverHidesItsClause`); the relative-clause
    /// chain itself is still unbuilt.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_unmodified_physician_assistant_sentence_recovers_both_definienda() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}physician assistant\u{201D} and the term \u{201C}nurse \
                     practitioner\u{201D} mean, for purposes of this subchapter, a physician \
                     assistant or nurse practitioner who performs such services as such \
                     individual is legally authorized to perform (in the State in which the \
                     individual performs such services) in accordance with State law (or the \
                     State regulatory mechanism provided by State law), and who meets such \
                     training, education, and experience requirements (or any combination \
                     thereof) as the Secretary may prescribe in regulations.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        let terms: BTreeSet<&str> = pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            ["nurse practitioner", "physician assistant"]
                .into_iter()
                .collect(),
            "got {pointers:?}"
        );
    }

    /// PRECISION REGRESSION CHECK: an ORDINARY coordinated NP subject
    /// (only ONE conjunct quoted) is NOT mistaken for coordinated
    /// definienda — `definiendum_words`'s own doc has the full story of
    /// the REAL regression this guards
    /// (`a_coordinated_subject_is_not_mistaken_for_a_medial_supplement`
    /// covers the medial-supplement angle of the SAME sentence; this test
    /// covers the coordinated-subject-extraction angle directly).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_ordinary_coordinated_subject_with_one_quoted_conjunct_yields_no_pointer() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}consumer\u{201D}, the county, and the state cover this benefit.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert!(
            pointers.is_empty(),
            "only one conjunct is a genuine quoted definiendum; got {pointers:?}"
        );
    }

    // ---- TASK #19 (G6): renvoi (definition-by-reference) ----

    /// [`renvoi_predicate_start`] recognizes every REAL variant this
    /// module's own test suite carries (byte-verified against
    /// `usc_title_42-pl-119-90.xml`): "has the meaning given such term in"
    /// (skilled nursing facility, § 1395x(j)), "has the same meaning given
    /// the term in" (family violence, § 3002(19)), "have the meaning
    /// given to them by" (State/United States, § 1395x(x)).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn renvoi_predicate_start_locates_the_closed_idiom_variants() {
        let en = english_loaded();
        for text in [
            "The term \u{201C}skilled nursing facility\u{201D} has the meaning given \
             such term in section 1395i\u{2013}3(a) of this title.",
            "The term \u{201C}family violence\u{201D} has the same meaning given the \
             term in the Family Violence Prevention and Services Act.",
            "The terms \u{201C}State\u{201D} and \u{201C}United States\u{201D} have the \
             meaning given to them by section 410 of this title.",
        ] {
            let (tokens, _) = tokenize::tokenize_with_alternatives(text, en);
            let start = renvoi_predicate_start(&tokens);
            assert!(
                start.is_some_and(|i| tokens[i].word == "has" || tokens[i].word == "have"),
                "{text:?} must locate its own has/have; got {start:?}"
            );
        }
    }

    /// HONEST: an ordinary "means" declarative carries no renvoi predicate
    /// at all — the two frames are genuinely disjoint.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn renvoi_predicate_start_returns_none_for_an_ordinary_means_sentence() {
        let en = english_loaded();
        let (tokens, _) = tokenize::tokenize_with_alternatives(
            "The term \u{201C}consumer\u{201D} means a natural person.",
            en,
        );
        assert_eq!(renvoi_predicate_start(&tokens), None);
    }

    /// The REAL, byte-verified headline renvoi predicate/citation from 42
    /// U.S.C. § 1395x(j), recombined with the proven WordNet-known
    /// "consumer" definiendum (the SAME isolation precedent G1-G5
    /// establish throughout this file): "The term 'consumer' has the
    /// meaning given such term in section 1395i–3(a) of this title."
    /// Same-archive (`Local`) resolution — the cited section is in the
    /// SAME title.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_has_the_meaning_given_such_term_in_y_isolated() {
        let en = english_loaded();
        let text = "The term \u{201C}consumer\u{201D} has the meaning given such term \
                     in section 1395i\u{2013}3(a) of this title.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s1395i\u{2013}3/a".to_string(),
            text: "section 1395i\u{2013}3(a) of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s1395i\u{2013}3/a".to_string());
        let peers = BTreeMap::new();

        let pointers = renvoi_pointers(text, en, &refs, &own_names, &peers)
            .expect("renvoi never fails closed");
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        assert_eq!(
            pointers[0].target,
            EdgeTarget::Local("/us/usc/t42/s1395i\u{2013}3/a".to_string())
        );
    }

    /// HONEST BASELINE: the REAL, UNMODIFIED § 1395x(j) sentence
    /// (byte-verified against `usc_title_42-pl-119-90.xml`): "The term
    /// 'skilled nursing facility' has the meaning given such term in
    /// section 1395i–3(a) of this title." The RENVOI mechanism itself is
    /// no longer the blocker (see the isolated test above, using the
    /// IDENTICAL predicate/citation structure); this REAL sentence still
    /// yields no pointer because "skilled nursing facility" is not a
    /// WordNet-indexed compound headword — the SAME G7 written-form-floor
    /// gap this file's G4 tests already document, here applying to a
    /// RENVOI definiendum rather than a "means" one.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_unmodified_skilled_nursing_facility_renvoi_still_yields_no_pointer() {
        let en = english_loaded();
        let text = "The term \u{201C}skilled nursing facility\u{201D} has the meaning \
                     given such term in section 1395i\u{2013}3(a) of this title.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s1395i\u{2013}3/a".to_string(),
            text: "section 1395i\u{2013}3(a) of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s1395i\u{2013}3/a".to_string());
        let pointers = renvoi_pointers(text, en, &refs, &own_names, &BTreeMap::new())
            .expect("renvoi never fails closed");
        assert!(
            pointers.is_empty(),
            "\"skilled nursing facility\" is not a WordNet-indexed compound headword — \
             a separately-scoped G7 gap; got {pointers:?}"
        );
    }

    /// The REAL, byte-verified renvoi predicate from 42 U.S.C. § 3002(19)
    /// — "has the SAME meaning given the term in" (a DIFFERENT surface
    /// than "has the meaning given such term in") — proving the closed
    /// idiom's "same" variant, recombined with "consumer" for the SAME
    /// isolation reason.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_has_the_same_meaning_given_the_term_in_y_isolated() {
        let en = english_loaded();
        let text = "The term \u{201C}consumer\u{201D} has the same meaning given the \
                     term in the Family Violence Prevention and Services Act.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s10401".to_string(),
            text: "42 U.S.C. 10401".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s10401".to_string());
        let peers = BTreeMap::new();

        let pointers = renvoi_pointers(text, en, &refs, &own_names, &peers)
            .expect("renvoi never fails closed");
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        assert_eq!(
            pointers[0].target,
            EdgeTarget::Local("/us/usc/t42/s10401".to_string())
        );
    }

    /// HONEST BASELINE: the REAL, UNMODIFIED § 3002(19) sentence: "The
    /// term 'family violence' has the same meaning given the term in the
    /// Family Violence Prevention and Services Act [42 U.S.C. 10401 et
    /// seq.]." — the SAME G7 written-form-floor gap
    /// (`a_real_unmodified_skilled_nursing_facility_renvoi_still_yields_no_pointer`'s
    /// own doc has the full citation) — "family violence" is not a
    /// WordNet-indexed compound headword either.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_unmodified_family_violence_renvoi_still_yields_no_pointer() {
        let en = english_loaded();
        let text = "The term \u{201C}family violence\u{201D} has the same meaning given \
                     the term in the Family Violence Prevention and Services Act.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s10401".to_string(),
            text: "42 U.S.C. 10401".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s10401".to_string());
        let pointers = renvoi_pointers(text, en, &refs, &own_names, &BTreeMap::new())
            .expect("renvoi never fails closed");
        assert!(
            pointers.is_empty(),
            "\"family violence\" is not a WordNet-indexed compound headword; got {pointers:?}"
        );
    }

    /// The REAL, byte-verified CROSS-TITLE renvoi predicate from 42
    /// U.S.C. § 247b–15(c) — "has the meaning given THAT term in" (a
    /// THIRD closed-idiom variant), AND a fronted "In this section,"
    /// adjunct this mechanism never needs to parse (it only reads the
    /// definiendum span BEFORE the renvoi predicate, whatever precedes
    /// it) — recombined with "consumer" for the SAME isolation reason.
    /// END-TO-END through the GENERIC `AtomResolver` — mirroring
    /// `a_cross_title_citation_resolves_and_the_edge_resolves_through_atom_resolver`'s
    /// own proof for `cites`, here for `definesByReference`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_x_has_the_meaning_given_that_term_in_y_cross_title_isolated() {
        let en = english_loaded();
        let text = "In this section, the term \u{201C}consumer\u{201D} has the meaning \
                     given that term in section 5304 of title 25.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t25/s5304".to_string(),
            text: "section 5304 of title 25".to_string(),
        }];
        let own_names = BTreeSet::new(); // the citing archive does NOT declare it

        let cited = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t25/s5304".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: Some("Definitions".to_string()),
        };
        let peer_archive = Archive {
            nodes: alloc::vec![cited],
            connections: alloc::vec![],
        };
        let peer_root = peer_archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert("usc_title_25".to_string(), peer_archive);

        let pointers = renvoi_pointers(text, en, &refs, &own_names, &peers)
            .expect("renvoi never fails closed");
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "consumer");
        let target = pointers[0].target.clone();
        assert!(
            matches!(&target, EdgeTarget::Grounded { ontology, .. } if ontology == "usc_title_25"),
            "resolves against the usc_title_25 peer, by the title parsed from the href"
        );

        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: "usc_title_25".to_string(),
            root: peer_root,
            role: "definesByReference".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(&target)
            .expect("the minted Grounded edge resolves by content address");
        assert_eq!(resolved.name, "/us/usc/t25/s5304");
        assert_eq!(resolved.kind, "Section");
    }

    /// HONEST BASELINE: the REAL, UNMODIFIED § 247b–15(c) sentence: "In
    /// this section, the term 'Indian tribe' has the meaning given that
    /// term in section 5304 of title 25." — the SAME G7 gap; "Indian
    /// tribe" is not a WordNet-indexed compound headword either.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_unmodified_indian_tribe_renvoi_still_yields_no_pointer() {
        let en = english_loaded();
        let text = "In this section, the term \u{201C}Indian tribe\u{201D} has the \
                     meaning given that term in section 5304 of title 25.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t25/s5304".to_string(),
            text: "section 5304 of title 25".to_string(),
        }];
        let peers = BTreeMap::new();
        let pointers = renvoi_pointers(text, en, &refs, &BTreeSet::new(), &peers)
            .expect("renvoi never fails closed");
        assert!(
            pointers.is_empty(),
            "\"Indian tribe\" is not a WordNet-indexed compound headword; got {pointers:?}"
        );
    }

    /// Isolated COORDINATED renvoi: G5's coordinated-definienda extraction
    /// and G6's renvoi frame compose — TWO WordNet-known definienda
    /// coordinated under "the terms", sharing ONE renvoi predicate, EACH
    /// gets its own pointer at the SAME cited section.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_coordinated_terms_have_the_meaning_given_to_them_by_y_isolated() {
        let en = english_loaded();
        let text = "The terms \u{201C}consumer\u{201D} and \u{201C}individual\u{201D} have \
                     the meaning given to them by section 410 of this title.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s410".to_string(),
            text: "section 410 of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s410".to_string());
        let peers = BTreeMap::new();

        let pointers = renvoi_pointers(text, en, &refs, &own_names, &peers)
            .expect("renvoi never fails closed");
        let terms: BTreeSet<&str> = pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            BTreeSet::from(["consumer", "individual"]),
            "got {pointers:?}"
        );
        assert!(
            pointers
                .iter()
                .all(|p| p.target == EdgeTarget::Local("/us/usc/t42/s410".to_string())),
            "both coordinated terms honestly share the SAME section-level target; \
             got {pointers:?}"
        );
    }

    /// HONEST BASELINE (partial): the REAL, UNMODIFIED § 1395x(x)
    /// sentence: "The terms 'State' and 'United States' have the meaning
    /// given to them by subsections (h) and (i), respectively, of section
    /// 410 of this title." The COORDINATION+RENVOI mechanism itself finds
    /// BOTH candidates (see the isolated test above); "state" clears the
    /// written-form floor and grounds (a section-level edge — this does
    /// NOT attempt the "...respectively..." per-term subsection split,
    /// `renvoi_pointers`'s own doc names that as a deliberate, separate,
    /// unaddressed capability), but "united states" does NOT — WordNet
    /// does not index it as a compound headword either (the SAME G7 gap).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_unmodified_state_and_united_states_renvoi_partially_grounds() {
        let en = english_loaded();
        let text = "The terms \u{201C}State\u{201D} and \u{201C}United States\u{201D} have \
                     the meaning given to them by subsections (h) and (i), respectively, of \
                     section 410 of this title.";
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s410".to_string(),
            text: "section 410 of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s410".to_string());
        let peers = BTreeMap::new();

        let pointers = renvoi_pointers(text, en, &refs, &own_names, &peers)
            .expect("renvoi never fails closed");
        let terms: BTreeSet<&str> = pointers.iter().map(|p| p.term.as_str()).collect();
        assert_eq!(
            terms,
            BTreeSet::from(["state"]),
            "\"united states\" is not a WordNet-indexed compound headword — a \
             separately-scoped G7 gap; got {pointers:?}"
        );
        assert_eq!(
            pointers[0].target,
            EdgeTarget::Local("/us/usc/t42/s410".to_string())
        );
    }

    /// HONEST: a renvoi predicate with NO citation `<ref>` supplied for
    /// the node yields no pointer — the SAME "no coverage → no pointer"
    /// discipline `cites_pointers`/`denotes_pointers` already establish.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn renvoi_pointers_yields_nothing_with_no_refs_supplied() {
        let en = english_loaded();
        let text = "The term \u{201C}skilled nursing facility\u{201D} has the meaning \
                     given such term in section 1395i\u{2013}3(a) of this title.";
        let pointers = renvoi_pointers(text, en, &[], &BTreeSet::new(), &BTreeMap::new())
            .expect("renvoi never fails closed");
        assert!(
            pointers.is_empty(),
            "a renvoi predicate with no citation to resolve grounds nothing; got {pointers:?}"
        );
    }

    /// HONEST: an ordinary "means" declarative — even with a `<ref>`
    /// present on the SAME node (a provision can cite something unrelated
    /// to its own definition) — never fires the renvoi path.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn renvoi_pointers_yields_nothing_for_an_ordinary_means_sentence_even_with_a_ref_present() {
        let en = english_loaded();
        let refs = alloc::vec![UsCodeRef {
            href: "/us/usc/t42/s410".to_string(),
            text: "section 410 of this title".to_string(),
        }];
        let mut own_names = BTreeSet::new();
        own_names.insert("/us/usc/t42/s410".to_string());
        let pointers = renvoi_pointers(
            "The term \u{201C}consumer\u{201D} means a natural person.",
            en,
            &refs,
            &own_names,
            &BTreeMap::new(),
        )
        .expect("renvoi never fails closed");
        assert!(pointers.is_empty(), "got {pointers:?}");
    }

    /// The TWO frames are genuinely disjoint in the OTHER direction too:
    /// `defines_pointers` (the "means" pipeline) never fires on a real
    /// renvoi sentence — it correctly finds no complete `Sem::Prop`
    /// (`renvoi_pointers` composes with `cites_pointers` instead, never a
    /// definiens parse — this module's own doc has the full rationale).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn defines_pointers_does_not_fire_on_a_real_renvoi_sentence() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}skilled nursing facility\u{201D} has the meaning \
                     given such term in section 1395i\u{2013}3(a) of this title.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert!(
            pointers.is_empty(),
            "the means-pipeline correctly finds no \"means\"/\"includes\" declarative \
             here; got {pointers:?}"
        );
    }

    /// G4(d), the REDUCED PASSIVE RELATIVE — CCGbank's own worked example
    /// (Hockenmaier & Steedman 2005, *CCGbank User's Manual* MS-CIS-05-09,
    /// §3.8 p. 55, (54)a "workers [exposed to it]") recombined into a
    /// definitional frame, plus the two REAL HCBS definienda whose definiens
    /// is exactly this shape: 42 U.S.C. § 1396b(l)(5)(B) VERBATIM, and the
    /// participial core of (l)(5)(C).
    ///
    /// The lexical gap this closes was measurable as a pure minimal pair:
    /// "services DESCRIBED under a State plan" already parsed (by the
    /// accident that WordNet independently lists "described" as an
    /// adjective) while "services PROVIDED under a State plan" did not
    /// (WordNet lists no adjective "provided", so the surface had only
    /// FINITE verb readings). Both now parse for the SAME reason — the
    /// participial category — not by lexical luck.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_a_definiens_with_a_reduced_passive_relative() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        for (text, want) in [
            (
                "The term \u{201C}worker\u{201D} means workers exposed to it.",
                "worker",
            ),
            (
                "The term \u{201C}home health care services\u{201D} means services \
                 described in section 1396d(a)(7) of this title provided under a State \
                 plan under this subchapter (or under a waiver of the plan).",
                "home health care services",
            ),
            (
                "The term \u{201C}personal care services\u{201D} means personal care \
                 services provided under a State plan under this subchapter (or under a \
                 waiver of the plan).",
                "personal care services",
            ),
        ] {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert!(
                pointers.iter().any(|p| p.term == want),
                "expected the definiendum {want:?} from {text:?}; got {pointers:?}"
            );
        }
    }

    /// The use/mention closure ([`ADefiniendumIsMentionedNeverUsed`]) holds
    /// over the SEMANTIC representation, with no lexicon or corpus load.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_definiendum_is_mentioned_never_used_axiom_holds() {
        assert!(ADefiniendumIsMentionedNeverUsed.verify().is_ok());
    }

    /// An adjunct the grammar cannot attach — anywhere in the provision —
    /// does not hide the definition it sits beside.
    ///
    /// The syntactic half of this closure is stated and proved without any
    /// load as
    /// [`AnUnattachableAdjunctNeverHidesItsClause`](crate::cognitive::linguistics::lambek::reduce::AnUnattachableAdjunctNeverHidesItsClause);
    /// this is the same claim end-to-end, over the REAL statutory shapes that
    /// motivated it, through the full lexicon + VerbNet pipeline.
    ///
    /// Each row is a MINIMAL PAIR against the plain frame in the first row:
    /// the definitional clause is character-for-character the same, and the
    /// only difference is adjunct material the chart has no attachment for.
    /// Under the whole-string parse goal every row but the first yielded
    /// nothing.
    ///
    /// - FRONTED reduced participial preamble — "As used in this section," /
    ///   "When used in this title,", 1,610 provisions corpus-wide. (The
    ///   finite-clause preambles "In this section," and "For purposes of this
    ///   section," always parsed; that asymmetry is what showed the failure
    ///   was attachment, not the definition.)
    /// - TRAILING infinitival purpose clause — 1 U.S.C. § 8's shape, "any
    ///   person authorized by law to perform the duties thereof": the
    ///   reduced passive relative already parsed ("authorized by law"), and
    ///   the infinitival complement hanging off it destroyed the whole
    ///   derivation.
    /// - BOTH at once, which is the ordinary case in running statutory text.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_unattachable_adjunct_does_not_hide_the_definition_beside_it() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        for text in [
            "The term \u{201C}vessel\u{201D} means a public official.",
            "As used in this section, the term \u{201C}vessel\u{201D} means a public official.",
            "When used in this title, the term \u{201C}vessel\u{201D} means a public official.",
            "The term \u{201C}vessel\u{201D} means any person authorized by law to perform \
             the duties thereof.",
            "As used in this section, the term \u{201C}vessel\u{201D} means any person \
             authorized by law to perform the duties thereof.",
        ] {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert!(
                pointers.iter().any(|p| p.term == "vessel"),
                "expected the definiendum \"vessel\" from {text:?}; got {pointers:?}"
            );
        }
    }

    /// Dropping the whole-string requirement does NOT drop the mention
    /// requirement: the sub-span goal is allowed to look inside a provision
    /// for a definitional clause, and still finds none where the provision
    /// defines nothing.
    ///
    /// Each sentence below embeds a quoted span AND a VerbNet-confirmed
    /// definitional predicate, so a partial parse has every opportunity to
    /// assemble a spurious definition out of a sub-span; the mention gate
    /// ([`definiendum_words`]) is what refuses, exactly as it does for the
    /// whole string.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_sub_span_goal_still_refuses_a_used_subject() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        for text in [
            // A citation-naming rule, 1 U.S.C. § 204(c) verbatim: the quoted
            // span is the name to cite BY, and the subject is used.
            "The Code of the District of Columbia may be cited as \u{201C}D.C. Code\u{201D}.",
            // The section HEADING of 1 U.S.C. § 1, which `usc_archive`
            // exposes as that section's own `lexical`.
            "Words denoting number, gender, and so forth",
        ] {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert!(
                pointers.is_empty(),
                "no sub-span of {text:?} declares a definition; got {pointers:?}"
            );
        }
    }

    /// …and end-to-end, through the FULL pipeline, over REAL operative prose
    /// that carries no quoted definiendum at all.
    ///
    /// Every sentence below reduces to a complete two-argument `Sem::Prop`
    /// under a predicate VerbNet confirms, and every one of them minted a
    /// `defines` edge before the use/mention check existed — these are the
    /// measured fabrications, verbatim:
    ///
    /// - the 1 U.S.C. § 1 section HEADING, which `usc_archive` exposes as
    ///   that section's `lexical`. It parses as `denote(words, …)` — a real
    ///   `[Theme, Co-Theme]` frame — and put `(/us/usc/t1/s1, "words")` in
    ///   the committed overlay, where § 1 in fact defines "person",
    ///   "whoever", "officer", "signature", "oath" and "writing" and never
    ///   "words".
    /// - 5 U.S.C. § 5569(g), whose derivation puts the participle
    ///   `Concept{"provided"}` in the subject slot under the passive
    ///   auxiliary "be" — the shape the reduced-relative coverage
    ///   (`recognizes_a_definiens_with_a_reduced_passive_relative`) made
    ///   reachable corpus-wide.
    /// - 5 U.S.C. § 409(b), same auxiliary shape, subject `Concept{"may"}`.
    /// - 1 U.S.C. § 204(c), a CITATION-NAMING rule: it quotes “D.C. Code”
    ///   as the name to cite BY, and its subject ("The Code of the District
    ///   of Columbia") is used, not mentioned — so the quote alone is not
    ///   what this check keys on, the SUBJECT's mention is.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_used_subject_in_real_operative_prose_yields_no_definiendum() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        for text in [
            // 1 U.S.C. § 1, section heading.
            "Words denoting number, gender, and so forth",
            // 5 U.S.C. § 5569(g).
            "Any benefit provided under subsection (c) or (d) may, under regulations \
             prescribed by the President, be provided to a family member of an \
             individual if\u{2014}",
            // 5 U.S.C. § 409(b).
            "In addition to the officers and employees provided for in section \
             406(a)(7) of this title, members of the Foreign Service may, at the \
             request of the Inspector General of the Agency for International \
             Development, be assigned as employees of the Inspector General.",
            // 1 U.S.C. § 204(c).
            "The Code of the District of Columbia may be cited as \u{201C}D.C. \
             Code\u{201D}.",
        ] {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert!(
                pointers.is_empty(),
                "no word in {text:?} is MENTIONED, so the provision defines nothing; \
                 got {pointers:?}"
            );
        }
    }

    /// The converse, over both mention shapes the corpus actually uses — so
    /// the check above is a use/mention discrimination, not a blanket refusal.
    ///
    /// `the term “X”` (a close-apposition mention promoted over its head NP)
    /// AND a BARE mention subject with no "the term"/"the word" head, which
    /// is how 42 U.S.C. § 10003 and 15 U.S.C. § 1352 write every one of their
    /// definitions. Both must survive; both are verbatim corpus prose.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_mentioned_subject_still_names_its_definiendum_in_both_corpus_shapes() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        for (text, want) in [
            // 1 U.S.C. § 2 — close-apposition mention under "The word".
            (
                "The word \u{201C}county\u{201D} includes a parish, or any other \
                 equivalent subdivision of a State or Territory of the United States.",
                "county",
            ),
            // 42 U.S.C. § 10003(1) — BARE mention subject, no head noun.
            (
                "\u{201C}radiation\u{201D} means ionizing and nonionizing radiation in \
                 amounts beyond normal background levels from sources such as medical \
                 and dental radiologic procedures;",
                "radiation",
            ),
            // 42 U.S.C. § 10003(6) — bare mention, capitalized definiendum.
            (
                "\u{201C}Secretary\u{201D} means the Secretary of Health and Human \
                 Services; and",
                "secretary",
            ),
        ] {
            let pointers = defines_pointers(text, en, en, vn, &mint_domain());
            assert!(
                pointers.iter().any(|p| p.term == want),
                "expected the mentioned definiendum {want:?} from {text:?}; \
                 got {pointers:?}"
            );
        }
    }

    /// The mention marking is the TOKENIZER's, and it survives the chart: a
    /// quoted span comes back [`ExpressionUse::Mentioned`] from
    /// `tokenize_with_alternatives`, and `reduce`'s re-typing carries it
    /// through rather than resetting every token to `Used`. Without the
    /// carry-through the whole check would silently reject every real
    /// definition instead of every fabrication — a failure mode no
    /// pointer-level assertion distinguishes from "the grammar got worse".
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_mention_marking_survives_tokenization_and_the_chart() {
        let en = english_loaded();
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives(
            "The word \u{201C}county\u{201D} includes a parish.",
            en,
        );
        let mentioned: Vec<&str> = tokens
            .iter()
            .filter(|t| t.expression_use == ExpressionUse::Mentioned)
            .map(|t| t.word.as_str())
            .collect();
        assert_eq!(
            mentioned,
            alloc::vec!["county"],
            "exactly the quoted span is mentioned; got {tokens:?}"
        );
        let reduced = reduce_with_alternatives_and_table_and_width(
            &tokens,
            &alternatives,
            &definiens_cost_table(),
            DEFINES_MAX_CHART_WIDTH,
        );
        assert_eq!(reduced.remaining.len(), tokens.len());
        let after: Vec<&str> = reduced
            .remaining
            .iter()
            .filter(|t| t.expression_use == ExpressionUse::Mentioned)
            .map(|t| t.word.as_str())
            .collect();
        assert_eq!(after, mentioned, "the chart must not erase the marking");
    }

    /// THE PARTICIPIAL READING IS SCOPED, and provably so: the extra
    /// category `defines_pointers` offers comes from `participle_alternatives`
    /// alone, and the LOADED OLiA→CCG functor — the one every live chat turn
    /// consults — still projects NOTHING for the `PastParticiple` class. So
    /// no chat derivation can see it, structurally, not just by convention
    /// (the same guarantee `with_extra_unary` gives the type-changing half).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn the_participial_category_never_reaches_the_shared_grammar() {
        use crate::cognitive::linguistics::lambek::category_projection::categories_for_class;
        let class = past_participle_form_class().expect("the participle marks a form class");
        assert!(
            categories_for_class(class).is_empty(),
            "the shared OLiA→CCG functor must project no category for {class:?} \
             until a corpus-gate measurement admits one"
        );
    }

    /// The mark itself IS produced by the loaded morphology — so the scoped
    /// projection above has something real to key on. A regular syncretic
    /// `-ed` ("provided") and an irregular participle ("given") both carry
    /// the OLiA `PastParticiple` form class; a bare finite form does not.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_loaded_morphology_marks_real_participle_surfaces() {
        use crate::cognitive::linguistics::language::Language;
        let en = english_loaded();
        let class = past_participle_form_class().expect("the participle marks a form class");
        let marked = |w: &str| {
            en.lexical_lookup_all(w)
                .iter()
                .any(|e| e.olia_class() == Some(class))
        };
        for w in ["provided", "performed", "delivered", "exposed", "given"] {
            assert!(marked(w), "{w:?} must carry the participle form class");
        }
        for w in ["provide", "service", "plan"] {
            assert!(!marked(w), "{w:?} is not a participle surface");
        }
    }

    /// THE GENERIC LOOP, renvoi edition: a content archive grounds via
    /// `ground(renvoi_lens(...))`, and the minted edge resolves through
    /// the GENERIC `AtomResolver` — mirroring
    /// `a_content_archive_grounds_via_the_cites_lens_and_resolves` exactly,
    /// proving `definesByReference` really is "another lens of the same
    /// shape" as `cites`/`defines`. Uses the § 1395x(j) renvoi predicate
    /// and citation VERBATIM, recombined with "consumer" (the SAME
    /// isolation reason `recognizes_the_term_x_has_the_meaning_given_such_term_in_y_isolated`
    /// gives — WordNet does not index the real "skilled nursing facility"
    /// as a compound headword, a separately-scoped G7 gap, not this
    /// lens-wiring proof's concern).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_content_archive_grounds_via_the_renvoi_lens_and_resolves() {
        use pr4xis_runtime::grounding::ground;

        let en = english_loaded();

        let citing = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t42/s1395x/j".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: Some(
                "The term \u{201C}consumer\u{201D} has the meaning given such term in \
                 section 1395i\u{2013}3(a) of this title."
                    .to_string(),
            ),
        };
        let content = Archive {
            nodes: alloc::vec![citing],
            connections: alloc::vec![],
        };
        let own_names: BTreeSet<String> = content.nodes.iter().map(|n| n.name.clone()).collect();

        let cited = Definition {
            kind: "Section".to_string(),
            name: "/us/usc/t42/s1395i\u{2013}3/a".to_string(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: None,
        };
        let peer_archive = Archive {
            nodes: alloc::vec![cited],
            connections: alloc::vec![],
        };
        let peer_root = peer_archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert("usc_title_42".to_string(), peer_archive);

        let mut refs_by_urn = BTreeMap::new();
        refs_by_urn.insert(
            "/us/usc/t42/s1395x/j".to_string(),
            alloc::vec![UsCodeRef {
                href: "/us/usc/t42/s1395i\u{2013}3/a".to_string(),
                text: "section 1395i\u{2013}3(a) of this title".to_string(),
            }],
        );

        let grounded = ground(&content, renvoi_lens(en, &refs_by_urn, &own_names, &peers))
            .expect("the renvoi lens grounds");
        let provision = &grounded.nodes[0];
        let (kind, target) = provision
            .edges
            .iter()
            .find(|(k, _)| k == DEFINES_BY_REFERENCE_REL)
            .expect("the renvoi grounded");
        assert_eq!(kind, DEFINES_BY_REFERENCE_REL);

        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: "usc_title_42".to_string(),
            root: peer_root,
            role: "definesByReference".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(target)
            .expect("the grounded renvoi target resolves");
        assert_eq!(resolved.name, "/us/usc/t42/s1395i\u{2013}3/a");
    }

    // ---- G7: written-form floor — case-folded lookup + out-of-lexicon minting ----

    /// MECHANISM: `is_known_written_form`'s case-folded tier recovers a
    /// capitalized WordNet lemma the tokenizer's own lowercasing hides —
    /// "Indian" (Fellbaum 1998, *WordNet: An Electronic Lexical Database*;
    /// verified here directly against the loaded lexicon, not assumed from
    /// the citation alone). The EXACT-case lookup misses (the tokenizer
    /// lowercases every surface before this check ever runs, the SAME
    /// Slice D population `resolve_surface`'s own doc names).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_known_written_form_recovers_a_case_folded_wordnet_lemma() {
        let en = english_loaded();
        assert!(
            en.lookup("indian").is_empty(),
            "sanity: the lowercased surface misses the exact-case WordNet lemma \"Indian\""
        );
        assert!(
            is_known_written_form(en, "indian"),
            "the case-folded tier recovers the capitalized lemma"
        );
    }

    /// MECHANISM, Honest: a genuine gibberish surface (no exact,
    /// lemmatized, or case-folded WordNet entry) is honestly reported
    /// unknown — the two-tier resolution never invents a false positive.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn is_known_written_form_is_false_for_a_genuine_nonword() {
        let en = english_loaded();
        assert!(
            !is_known_written_form(en, "zzqxvbflorp"),
            "no tier should match a nonword"
        );
    }

    /// ISOLATED mechanism proof for the case-folding tier, end-to-end
    /// through `defines_pointers`: "Indian" is a genuine WordNet headword
    /// whose exact-case lookup misses (see
    /// `is_known_written_form_recovers_a_case_folded_wordnet_lemma`),
    /// recombined with this module's own proven "means a natural person"
    /// definiens for isolation — the file's own standing precedent
    /// throughout G1–G6. NOT a claim that this exact sentence appears
    /// verbatim in Title 42: Title 42's own real "Indian" definitions this
    /// module's test suite carries (`a_real_unmodified_indian_tribe_renvoi_still_yields_no_pointer`)
    /// are RENVOI declarations — a separate frame `renvoi_pointers` alone
    /// handles, deliberately unchanged by this gap (see
    /// `is_known_written_form`'s own doc) — not a "means" declarative this
    /// function reduces. Before this fix: the tokenizer's lowercasing made
    /// "indian" invisible to a bare `lang.lookup`, dropping the pointer.
    /// After: the case-folded tier recovers it, grounding into
    /// `english_wordnet` — NOT the mint domain, since this one genuinely IS
    /// a WordNet-known written form.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_term_indian_means_y_via_the_case_folded_tier_isolated() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let pointers = defines_pointers(
            "The term \u{201C}Indian\u{201D} means a natural person.",
            en,
            en,
            vn,
            &mint_domain(),
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "indian");
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == ENGLISH_ONTOLOGY
            ),
            "\"Indian\" IS a WordNet headword (recovered via case-folding) — got {:?}",
            pointers[0].target
        );
    }

    /// ISOLATED proof for a REAL, web-verified out-of-lexicon coinage — LII
    /// / uscode.house.gov, 42 U.S.C. § 3002(3), the Older Americans Act's
    /// own definitions section (LLM-checked-via-web citation tier: I could
    /// not independently byte-verify this against the local XML corpus this
    /// module's OTHER tests use, unlike those — documented honestly, not
    /// glossed over): "The term "Assistant Secretary" means the Assistant
    /// Secretary for Aging." The REAL statutory object NP ("the Assistant
    /// Secretary for Aging") does NOT currently reach a full derivation
    /// (empirically confirmed, not assumed) — a SEPARATE, undiagnosed
    /// definiens-side gap, not this task's G7 scope — so this test isolates
    /// the DEFINIENDUM (the real "Assistant Secretary" coinage) against
    /// this module's own PROVEN PP-chain object
    /// (`recognizes_the_term_x_means_y_behind_the_real_secretary_pp_chain`'s
    /// own "the Secretary of Health and Human Services"), the file's
    /// standing isolation precedent throughout G1–G6. "assistant secretary"
    /// is a genuine statutory coinage: WordNet indexes "assistant" and
    /// "secretary" separately but NOT the two-word compound (verified
    /// directly against the loaded lexicon, both exact-case and
    /// case-folded), so it is MINTED rather than dropped.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_and_mints_the_real_assistant_secretary_coinage_isolated() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        assert!(
            en.lookup("assistant secretary").is_empty()
                && en.lookup_case_folded("assistant secretary").is_empty(),
            "sanity: \"assistant secretary\" is not a WordNet-indexed compound headword"
        );
        let domain = mint_domain();
        let pointers = defines_pointers(
            "The term \u{201C}Assistant Secretary\u{201D} means the Secretary \
             of Health and Human Services.",
            en,
            en,
            vn,
            &domain,
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "assistant secretary");
        assert_eq!(
            pointers[0].exhaustiveness,
            DefinitionExhaustiveness::Exhaustive
        );
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "a statutory coinage with no WordNet entry is MINTED, not dropped — got {:?}",
            pointers[0].target
        );
    }

    /// ISOLATED proof for a SECOND real out-of-lexicon coinage — 42 U.S.C.
    /// § 1396d(p)(1)'s "qualified medicare beneficiary" (web-verified, LII;
    /// the REAL statutory definiens is a long relative-clause chain far
    /// beyond this grammar's G4 coverage, separately out of scope —
    /// recombined with this module's own proven definiens for isolation,
    /// the same precedent as the case-fold test above).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_and_mints_the_real_qualified_medicare_beneficiary_coinage_isolated() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let domain = mint_domain();
        let pointers = defines_pointers(
            "The term \u{201C}qualified medicare beneficiary\u{201D} means a natural person.",
            en,
            en,
            vn,
            &domain,
        );
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "qualified medicare beneficiary");
        assert!(
            matches!(
                &pointers[0].target,
                EdgeTarget::Grounded { ontology, .. } if ontology == domain.as_str()
            ),
            "got {:?}",
            pointers[0].target
        );
    }

    /// ONTOLOGICAL ASSERTION: the SAME out-of-lexicon coinage, minted from
    /// TWO DIFFERENT declaring sentences (one "means", one "includes"),
    /// derives the IDENTICAL content-addressed target —
    /// `lemon::mint::mint`'s own documented determinism
    /// (`mint_is_deterministic_the_same_term_and_domain_mint_the_same_reference`),
    /// now proven through THIS module's real caller, not just `mint`'s own
    /// isolated unit tests.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn the_same_minted_coinage_derives_the_same_target_from_two_declaring_sentences() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let domain = mint_domain();
        let first = defines_pointers(
            "The term \u{201C}qualified medicare beneficiary\u{201D} means a natural person.",
            en,
            en,
            vn,
            &domain,
        );
        let second = defines_pointers(
            "The term \u{201C}qualified medicare beneficiary\u{201D} includes an individual.",
            en,
            en,
            vn,
            &domain,
        );
        assert_eq!(first.len(), 1, "got {first:?}");
        assert_eq!(second.len(), 1, "got {second:?}");
        assert_eq!(
            first[0].target, second[0].target,
            "the SAME (domain, term) mints the SAME content-addressed target \
             regardless of which sentence names it"
        );
    }

    /// `defines_pointers`'s OWN chart derivation runs under
    /// [`DEFINES_MAX_CHART_WIDTH`] (512), NOT the SHARED bound
    /// (`crate::cognitive::linguistics::lambek::reduce::reduce_with_alternatives_and_table`'s
    /// 256) — proven directly at the SAME layer `defines_pointers` itself
    /// calls (`reduce_with_alternatives_and_table_and_width`, over
    /// [`definiens_cost_table`]'s own table), using the SAME
    /// right-branching `S/S` chain
    /// (`crate::cognitive::linguistics::lambek::reduce::chart_tests::a_caller_supplied_wider_bound_accepts_what_the_shared_bound_refuses`
    /// proves the underlying `chart_reduce_with_costs`/
    /// `chart_reduce_with_costs_bounded` mechanism this composes over —
    /// Steedman (2000)'s `S/S` sentence-modifier category, needing no real
    /// lexicon). This isolates the WIDTH-BOUND WIRING itself — the report's
    /// own measured 351-token record (42 U.S.C. § 1395x(r) "physician") is
    /// NOT reproduced here (this task had no access to the local XML corpus
    /// other tests byte-verify against), and whether the G4 coordination
    /// grammar itself scales to hundreds of real tokens is a SEPARATE,
    /// untested claim this test does not make.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn defines_pointers_own_bound_accepts_what_the_shared_bound_refuses() {
        use crate::cognitive::linguistics::lambek::reduce::reduce_with_alternatives_and_table;

        let width = 300;
        let tokens: Vec<TypedToken> = (0..width)
            .map(|i| TypedToken {
                expression_use: ExpressionUse::Used,
                word: alloc::format!("w{i}"),
                lambek_type: if i + 1 < width {
                    LambekType::right_div(LambekType::s(), LambekType::s())
                } else {
                    LambekType::s()
                },
            })
            .collect();
        let alternatives: Vec<Vec<LambekType>> = alloc::vec![Vec::new(); width];
        let table = definiens_cost_table();

        let shared = reduce_with_alternatives_and_table(&tokens, &alternatives, &table);
        assert!(
            !shared.success,
            "the shared 256-token bound refuses a {width}-token derivation, \
             even though the grammar itself could derive it"
        );

        let widened = reduce_with_alternatives_and_table_and_width(
            &tokens,
            &alternatives,
            &table,
            DEFINES_MAX_CHART_WIDTH,
        );
        assert!(
            widened.success,
            "defines_pointers's own DEFINES_MAX_CHART_WIDTH derives the SAME \
             {width}-token chain the shared bound refuses"
        );
    }

    // ---- Title 1 ground truth (manual verbatim audit, this session) ----
    //
    // The committed corpus-wide ratchet floor for Title 1 is only 3 pairs
    // (`defines_pointers_corpus_ratchet.rs`'s `TITLE_1_FLOOR`), and Title 1
    // is the U.S. Code's smallest title ("General Provisions") — small
    // enough to audit BY HAND against the real, current, on-disk XML
    // (`crates/domains/data/legal/uscode/usc_title_1/
    // usc_title_1-pl-119-90.xml`) rather than trust the extracted count.
    // That manual read found 10 real, CURRENT, operative "the term X
    // means/includes/has the meaning" declaratives (excluding several
    // "means" hits that are historical `<quotedContent>` from expired
    // 1990s joint resolutions about enrollment procedures — NOT current
    // Title 1 law, correctly never extracted). A live decode of the
    // freshly-regenerated `.defines.cprx.gz` (this session,
    // `probe_title1_defines_overlay_content`) found only 3 pairs total for
    // the WHOLE title, and one of those three is itself a FALSE POSITIVE
    // (`/us/usc/t1/s204/c`, see below) — i.e. only 1 of the 10 real
    // definitions currently extracts. The tests below pin each real
    // definition individually with its own real text, so future
    // improvement is provable pair-by-pair, not just as one aggregate
    // count — and per explicit instruction, a currently-missing case is
    // left RED, not adjusted to match today's behavior.

    /// 1 U.S.C. § 7(b) — real text. Complete, single-sentence "the term X
    /// means Y" declarative, no dangling chapeau, no medial adjunct — the
    /// simplest possible shape. Was MISSING from the real corpus extraction;
    /// root cause found via direct chart instrumentation
    /// (`probe_title1_state_chart_divergence_point`,
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`) and confirmed
    /// NOT to be the N-ary coordination machinery at all: `collapse_
    /// capitalized_runs` (`tokenize.rs`) collapsed "United States" into a
    /// single token typed bare `NP` with only an `NP\NP` alternative — no
    /// `N` reading — so "the United States" (`NP/N` + `NP`) had no
    /// application-only reduction and the whole sentence never reached `S`.
    /// FIXED by `NpForcing::ProperNounRun` adding the missing `N`
    /// alternative reading (`tokenize.rs`). Generalization past the exact
    /// bisected arity (4-item) confirmed adversarially — 5/6/8-item
    /// coordinations and a mid-list (not last-conjunct) proper-noun run all
    /// extract correctly (`probe_the_proper_noun_run_fix_generalizes_
    /// past_arity_four`, same file).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_title1_state_definition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "In this section, the term \u{201C}State\u{201D} means a State, \
             the District of Columbia, the Commonwealth of Puerto Rico, or any \
             other territory or possession of the United States.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(
            pointers.len(),
            1,
            "real 1 U.S.C. § 7(b) definition not extracted — a genuine, \
             confirmed gap, not a regression; got {pointers:?}"
        );
        assert_eq!(pointers[0].term, "state");
    }

    /// 1 U.S.C. § 112b(k)(2) — real text. Also a complete, single-sentence
    /// declarative with no dangling chapeau — structurally the SAME shape
    /// as the sibling § 112b(k)(6) "Secretary" definition four paragraphs
    /// later in the SAME subsection, which DOES extract correctly. Was
    /// MISSING — a genuinely SEPARATE gap from the "State" definition above
    /// (not the N-ary coordination machinery, and not the same root cause):
    /// the embedded subject relative clause "that negotiates and enters
    /// into a qualifying non-binding instrument" needed (1) a
    /// transitive-verb-level coordinator, `svo::transitive_verb_coordinator`
    /// = `(TV\TV)/TV` with `TV = (NP\S)/NP` — Steedman (2000) *The Syntactic
    /// Process*, Ch. 4's general `(X\X)/X` coordination schema instantiated
    /// at the transitive-verb level, mirroring [`svo::nominal_coordinator_np`]
    /// and [`svo::sentential_coordinator_wq`]'s own scoping precedent — and
    /// (2) a prepositional-verb particle, `svo::transitive_verb_particle` =
    /// `(TV\TV)`, so "enters" + "into" combine BEFORE the shared object is
    /// absorbed (Huddleston & Pullum 2002, *CGEL* Ch. 7 §3 "Prepositional
    /// verbs"; Hockenmaier & Steedman 2005, *CCGbank User's Manual*,
    /// Appendix A.2). Both are additive alternative readings (`tokenize.rs`),
    /// never a hand-listed word set — offered to every "and"/"or"-or-marker
    /// coordinator and every ordinary preposition respectively, gated on
    /// type shape, and arbitrated by the existing Viterbi cost/completeness
    /// ranking. Generalization confirmed adversarially — a different
    /// prepositional verb ("relies on") and 3-way verb coordination
    /// ("drafts, negotiates, and enters into") both extract correctly
    /// (`probe_the_transitive_verb_coordinator_fix_generalizes`,
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_title1_appropriate_department_or_agency_definition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}appropriate department or agency\u{201D} means \
             the department or agency of the United States Government that \
             negotiates and enters into a qualifying non-binding instrument on \
             behalf of itself or the United States.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(
            pointers.len(),
            1,
            "real 1 U.S.C. § 112b(k)(2) definition not extracted — a genuine, \
             confirmed gap, not a regression; got {pointers:?}"
        );
        assert_eq!(pointers[0].term, "appropriate department or agency");
    }

    /// 1 U.S.C. § 112b(k)(6) — the ONE real Title 1 definition confirmed
    /// currently extracting correctly (`probe_title1_defines_overlay_content`,
    /// this session). Locked in as a regression guard alongside its two
    /// currently-failing siblings above, so a future fix to those two
    /// cannot silently break this one.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_real_title1_secretary_definition() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}Secretary\u{201D} means the Secretary of State.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "secretary");
    }

    /// FIXED, was a KNOWN FALSE POSITIVE: 1 U.S.C. § 204(c) — real text.
    /// "The Code of the District of Columbia may be cited as "D.C. Code"."
    /// is a CITATION-CONVENTION statement (how one specific referent may be
    /// abbreviated), never a "the term X means Y" declarative — it names no
    /// term and defines nothing. It nonetheless minted a pointer with term
    /// "code", confirmed both by decoding the real title-1 corpus cache
    /// (`probe_title1_defines_overlay_content`) and by re-running this exact
    /// isolated sentence.
    ///
    /// The use/mention closure ([`ADefiniendumIsMentionedNeverUsed`]) is what
    /// closed it, and this witness is the sharpest statement of what that
    /// closure actually keys on: the sentence DOES contain a quoted span
    /// (“D.C. Code”), so "the text has quotes in it" would not have
    /// discriminated. What discriminates is that the quoted span is the
    /// citation FORM, sitting in an adjunct, while the SUBJECT — "The Code
    /// of the District of Columbia" — is an ordinary used NP. A definiendum
    /// is the MENTIONED SUBJECT of a definitional predicate, nothing less
    /// specific.
    ///
    /// Kept (not deleted) so a regression back to the false positive is
    /// still caught, exactly as the assertion this replaces demanded.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_title1_dc_code_citation_sentence_no_longer_false_positives() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text =
            "The Code of the District of Columbia may be cited as \u{201C}D.C. Code\u{201D}.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert!(
            pointers.is_empty(),
            "a citation-naming rule defines nothing: its subject is used, not \
             mentioned, however many quoted spans sit elsewhere in it; got {pointers:?}"
        );
    }

    /// 1 U.S.C. § 112b(k)(1) — real text, the ALL-joined reassembly shape
    /// (chapeau + its two lettered subparagraphs)
    /// `dangling_chapeau_reassembly_index` generates for this real
    /// dangling-chapeau node. A structural sibling of G3/S2 (the
    /// 2026-07-15 coverage report's dangling-chapeau-enumeration gap
    /// class) — re-verified this session (after the footnote/heading fix
    /// landed) that this real title's dangling-chapeau definition DOES now
    /// extract, grounded into the `usc_t42_coinages` bucket the reassembly
    /// mechanism already uses for other titles' enumerations. Locked in as
    /// a regression guard.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_real_title1_appropriate_congressional_committees_enumeration_extracts() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}appropriate congressional committees\u{201D} \
             means\u{2014} the Committee on Foreign Relations of the Senate; \
             and the Committee on Foreign Affairs of the House of \
             Representatives.";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert_eq!(pointers.len(), 1, "got {pointers:?}");
        assert_eq!(pointers[0].term, "appropriate congressional committees");
    }

    /// KNOWN GAP, documented not fixed: 1 U.S.C. § 112b(k)(3) — real text, a
    /// RENVOI (definition-by-reference, G6) construction: "has the meaning
    /// given that term in section 3(4) of the National Security Act of
    /// 1947 (50 U.S.C. 3003(4))." The existing renvoi tests in this module
    /// (`recognizes_the_term_x_has_the_meaning_given_such_term_in_y_isolated`
    /// etc.) cover the general shape via a SHORTER cross-reference target;
    /// this REAL Title 1 witness — with its real, longer statutory
    /// cross-reference ("section 3(4) of the National Security Act of 1947
    /// (50 U.S.C. 3003(4))") — was confirmed this session to still produce
    /// zero pointers: renvoi is a DIFFERENT frame (a cites-substrate
    /// composition, not a definiens parse) per this module's own G6 doc,
    /// and that composition is not yet wired for this real target shape.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_real_title1_intelligence_community_renvoi_definition_is_not_yet_confirmed() {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let text = "The term \u{201C}intelligence community\u{201D} has the meaning \
             given that term in section 3(4) of the National Security Act of \
             1947 (50 U.S.C. 3003(4)).";
        let pointers = defines_pointers(text, en, en, vn, &mint_domain());
        assert!(
            pointers.is_empty(),
            "if this now returns a pointer, the renvoi/cites-substrate gap for \
             this real Title 1 cross-reference has closed — update this test's \
             assertion (and doc comment) to lock in the fix rather than \
             deleting it; got {pointers:?}"
        );
    }
}
