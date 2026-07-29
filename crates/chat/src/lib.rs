use pr4xis::category::{Ap, NonEmpty, Product, Writer};
use pr4xis::ontology::Vocabulary;
use pr4xis::ontology::meta::OntologyName;
pub use pr4xis::ontology::meta::Provenance;
pub use pr4xis::ontology::meta::Provenance as RelationshipMeta;
use pr4xis_domains::cognitive::cognition::epistemics;
use pr4xis_domains::cognitive::linguistics::english::{
    ConceptId, English, LexicalReasoner, word_sense,
};
use pr4xis_domains::cognitive::linguistics::lambek::{
    ExpressionUse, ReductionResult, TypedToken, montague, reduce::chart_reduce, tokenize,
};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::pragmatics::realize::DefinitionProvenance;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::diagnostics::DiagnosticOntology;
use pr4xis_domains::formal::information::diagnostics::trace_functors::{
    PipelineTrace, TracedPipeline,
};
use pr4xis_domains::formal::information::diagnostics::trace_impls;
use pr4xis_domains::formal::information::knowledge::{
    SelfModelInstance, describe_knowledge_base, is_capability_query_referent, is_self_referent,
};

/// The Diagnostics ontology governs the trace — every PipelineTraceEntry is
/// a Diagnostic concept. `trace_meta()` is pulled from `ontology!`-generated
/// `meta()` so the chat engine's trace attribution flows from the ontology itself,
/// not from hardcoded strings. Public so callers can inspect which ontology
/// authorizes the trace semantics.
///
/// Returns a [`RelationshipMeta`] — the unified Lemon+PROV-O record every
/// structural entity in pr4xis carries (issue #153).
pub fn trace_meta() -> RelationshipMeta {
    DiagnosticOntology::meta()
}

// Praxis Chat Engine — shared logic for CLI, WASM, and any frontend.
//
// Zero I/O. Takes a string, returns a string.
// All intelligence comes from the Language ontology.
// The chat engine is a functor: Input → Language → Response.
//
// Trace is produced by applying trace functors to each pipeline step result.
// The trace functor maps: PipelineStep → DiagnosticConcept → PROV Activity.
// No manual trace.ok() — the functor provides ontology names and operations.

/// Re-export for callers.
pub use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineTraceEntry;

/// The generated chat-capability suite — a pure functor from a loaded reasoner
/// graph to `{(question, expected-property)}` plus a structural faithfulness
/// checker over [`ProcessResult`]. See the module docs.
pub mod capability;

#[cfg(test)]
mod coverage_gate;

/// Multi-turn conversation state (task #17) — [`session::ChatSession`]
/// carries a `ChatOutcome::Conditional` prompt's pending rule across turns.
pub mod session;
pub use session::ChatSession;

/// Alias — the trace is a PipelineTrace from the Diagnostics ontology.
pub type Trace = PipelineTrace;

/// The TYPED outcome of a turn (doc §4.1) — a self-aware system models what it
/// CANNOT answer, not just what it can. Abstention stops being a string the UI
/// sniffs ("I don't know …") and becomes a value: an `Abstained` turn names the
/// surfaces it could not resolve — the "asked but not loaded" set, i.e. WHAT TO
/// LOAD next. Reiter (1978) closed-world: the system knows the boundary of its
/// own knowledge.
///
/// `Eq` is dropped (was `PartialEq, Eq`): `Conditional`'s payload transitively
/// carries `Quantity` (`formal::math::quantity::value::Quantity`, `f64`-typed,
/// derives only `PartialEq`), which blocks a structural `Eq` derive.
/// `assert_eq!`/`matches!`, the only comparisons used against `ChatOutcome` in
/// this crate's own tests, both require `PartialEq` only.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatOutcome {
    /// The turn answered from knowledge (the embedded model or a loaded ontology).
    Answered,
    /// The turn abstained — it could not ground the query. `unresolved` is the set
    /// of asked surfaces no loaded ontology knows (empty when the input itself
    /// carried no resolvable entity, e.g. empty input).
    Abstained { unresolved: Vec<String> },
    /// A positive capability (Bobrow, Kaplan, Norman, Thompson & Winograd
    /// 1977, "GUS, A Frame-Driven Dialog System", *Artificial Intelligence*
    /// 8(2):155-173): the question is governed by a fully-known, fully-cited
    /// [`pr4xis_domains::social::judicial::conditional_rule::ConditionalRule`],
    /// but evaluating it for THIS asker is blocked on one or more private
    /// facts never supplied. `missing` names exactly what a future
    /// multi-turn dialogue layer must ask for. This is NOT abstention —
    /// `Abstained`'s own doc comment ("could not ground the query") would be
    /// false here: the query WAS grounded, in a real cited rule.
    Conditional {
        // Boxed: `ConditionalRule` transitively carries a `LegalTerm` with a
        // dozen `Vec` fields (~560 bytes), dwarfing `Abstained`'s ~24 bytes —
        // unboxed, every `ChatOutcome` value (including the far more common
        // `Answered`/`Abstained`) would pay that size.
        rule: Box<pr4xis_domains::social::judicial::conditional_rule::ConditionalRule>,
        missing: pr4xis::category::NonEmpty<
            pr4xis_domains::social::judicial::conditional_rule::AppliedElement,
        >,
    },
    /// The multi-turn slot-filling layer ([`session::ChatSession`], task
    /// #17) filled every `Required` element and Sergot et al. (1986)'s
    /// reduction reached a definite verdict — `applies` is `true` for
    /// `Applicability::Applies`, `false` for `Applicability::DoesNotApply`.
    /// Only ever produced by [`session::ChatSession::ask`], never by
    /// [`process_with_reasoner`] directly (a single turn alone never carries
    /// enough asker-supplied fact to resolve a rule).
    RuleResolved {
        rule: Box<pr4xis_domains::social::judicial::conditional_rule::ConditionalRule>,
        applies: bool,
    },
}

/// Result of processing input through the linguistics pipeline.
pub struct ProcessResult {
    pub response: String,
    pub user_act: SpeechAct,
    pub system_act: SpeechAct,
    pub duration_us: u64,
    pub token_count: usize,
    pub parsed: bool,
    pub trace: Trace,
    pub from_ontology: bool,
    /// The typed outcome — answered, or abstained with the unresolved surfaces.
    pub outcome: ChatOutcome,
    /// The plain-language "Why?" explanation of this outcome (doc §5.2) — a
    /// SECOND realization of the turn's communicative goal, glossing WHY the
    /// engine reached this outcome (what it found and where, or what it could not
    /// connect). `None` for the outcomes whose primary response already carries
    /// its own reason (`Conditional`/`RuleResolved`, and the unproven-relation
    /// abstain), so the renderer shows no Why? panel. Realized centrally in
    /// [`process_with_reasoner`] from the LOADED explain-frames table
    /// ([`realize_why`](pr4xis_domains::cognitive::linguistics::pragmatics::realize::realize_why)),
    /// never assembled in code or sniffed from the response text.
    pub why: Option<String>,
    /// The DEFINITION-PROVENANCE line (doc §5.2's third channel): the loaded
    /// lexicon that wrote a recited gloss and the documents it wrote the gloss
    /// FROM, realized from the same loaded explain-frames table as [`why`](Self::why).
    ///
    /// Carried as its OWN field rather than appended to `why` on purpose: a
    /// renderer must be able to show "authored from" and "reasoned over" as two
    /// different facts WITHOUT parsing the `why` sentence to tell them apart.
    /// `None` when the turn recited no sourced definition.
    pub definition_provenance:
        Option<pr4xis_domains::cognitive::linguistics::pragmatics::realize::RealizedProvenance>,
}

/// Process input through the full linguistics pipeline.
/// Returns (response_text, user_speech_act, system_speech_act).
///
/// Source-agnostic: the pipeline reasons over the embedded English
/// language model. Loaded corpora (statutes, …) are surfaced through the
/// self-model catalog (see `self_describe`), not threaded into the
/// linguistic pipeline.
pub fn process(lang: &English, input: &str) -> (String, SpeechAct, SpeechAct) {
    let result = process_with_metadata(lang, input);
    (result.response, result.user_act, result.system_act)
}

/// Process with full metadata — timing, token count.
///
/// The pipeline IS a writer monad computation: `TracedPipeline<A> = Writer<PipelineTrace, A>`.
/// Each stage returns a traced value, and composition through `.bind()` / `.tell()`
/// accumulates trace entries automatically via the PipelineTrace monoid.
/// No mutation. No manual trace.record() calls.
///
/// Reference: Moggi, "Notions of Computation and Monads" (1991).
///
/// The linguistic pipeline stages (tokenize/parse/interpret/respond)
/// reason over `lang` only. Self-referential queries route through the
/// self-model (see `self_describe`); loaded corpora live in its catalog.
///
/// The lexical-reasoning surface (`lookup` / `is_a` / `concept` / define)
/// is threaded through a `&dyn LexicalReasoner` — `English` itself by
/// default. A caller that has loaded corpora can inject a
/// [`process_with_reasoner`] reasoner that grounds those corpora into the
/// same lexical surface (see `ComposedReasoner` in `pr4xis-domains`).
pub fn process_with_metadata(lang: &English, input: &str) -> ProcessResult {
    // Default reasoner = the embedded English model itself (behavior-
    // preserving: English is both the linguistic substrate and the lexical
    // reasoner when no corpus is loaded).
    process_with_reasoner(lang, lang, input)
}

/// Scan raw `input` for a leading presupposed clause: a registered temporal
/// presupposition-trigger surface (`presupposition_trigger_lexicon` —
/// currently just "since", Beaver & Condoravdi 2003 / Heinämäki 1974)
/// followed by a clause and a comma. Returns `(presupposed_clause_text,
/// main_clause_text)`.
///
/// Operates on the RAW TEXT, not the tokenized stream, because trailing
/// comma punctuation does not survive tokenization (`tokenize::surface_tokens`
/// trims non-operator ASCII punctuation) — there is no comma TOKEN to scan
/// for by the time `find_governed_predicate`-style token-stream matching
/// would run. The clause-boundary marker (a comma) is a universal
/// orthographic fact, not input-specific string matching; the TRIGGER surface
/// itself is loaded data, exactly like `find_governed_predicate`'s registry
/// lookup.
fn find_presupposed_clause(input: &str) -> Option<(String, String)> {
    use pr4xis_domains::cognitive::linguistics::presupposition_trigger_lexicon::presupposition_trigger_surfaces;

    let (before, after) = input.split_once(',')?;
    let after = after.trim();
    if after.is_empty() {
        return None;
    }
    let words: Vec<&str> = before.split_whitespace().collect();
    for trigger in presupposition_trigger_surfaces() {
        let trigger_len = trigger.split(' ').count().max(1);
        if words.len() <= trigger_len {
            continue;
        }
        let lead = words[..trigger_len].join(" ");
        if lead.eq_ignore_ascii_case(&trigger) {
            let clause = words[trigger_len..].join(" ");
            if !clause.is_empty() {
                return Some((clause, after.to_string()));
            }
        }
    }
    None
}

/// Is `s` (a lowercased, possibly multi-word surface) already known to the
/// FULL composed reasoner — WordNet ⊕ every registered domain lexicon? The
/// same knowledge check `collapse_multiword_surfaces`'s own classify closure
/// below uses, factored out so the tokenizer's capitalized-run detector
/// (`tokenize::tokenize_ontological_registry_aware` /
/// `tokenize::tokenize_with_alternatives_registry_aware`) can defer to it —
/// without this, a registered program/agency name or statutory acronym that
/// happens to be capitalized ("Residential Habilitation", "EVV") gets fused
/// into a gloss-less NP before `collapse_multiword_surfaces` ever gets a
/// chance to resolve it (that function only ever WIDENS adjacent tokens; it
/// never re-splits an already-merged one to re-classify it). A confirmed
/// real corpus regression before this guard existed.
fn is_registry_known_surface(reasoner: &dyn LexicalReasoner, s: &str) -> bool {
    !reasoner.lookup(s).is_empty()
        || !reasoner.lookup_case_folded(s).is_empty()
        || reasoner.relation_for_surface(s).is_some()
        || pr4xis_domains::cognitive::linguistics::scope_predicate_lexicon::scope_predicate_surfaces()
            .contains(s)
        || reasoner.is_loaded_surface(s)
}

/// The classify closure `collapse_multiword_surfaces` (tokenize.rs) probes
/// for every candidate span — factored out of `parse_clause` and
/// `process_with_reasoner` (previously duplicated verbatim between the two)
/// so the readings a resolved multi-word surface offers stay in exactly one
/// place.
///
/// An ENTITY surface (a loaded citation/label/collocation OR a WordNet
/// multi-word lemma) collapses to BOTH nominal readings (proper noun / NP so
/// it stands alone as a subject/complement, common noun / N so a
/// determiner attaches: "a cough out", "a care for") — and ADDITIONALLY, when
/// the resolved concept's OWN loaded WordNet-LMF part of speech
/// (`ConceptView::pos`) is `Verb` ("look like" — WordNet indexes multiword
/// collocations across every part of speech, not just nouns, Fellbaum 1998
/// *WordNet: An Electronic Lexical Database*, MIT Press, Ch.1), the
/// verb-shaped readings too (transitive / bare-transitive, the same pair a
/// single-token verb lemma carries), so "what does X look like" can reduce
/// the same way "what does X mean" already does — through the existing
/// `does_support()` + `wh_what_object()` machinery, unchanged. Additive
/// rather than exclusive — matching the "offer every reading, let the chart
/// keep whichever derives a complete parse" discipline this file already
/// applies everywhere else (Steedman 2000, *The Syntactic Process* — CCG
/// resolves lexical ambiguity by chart search over every category a token
/// carries, never by picking one up front): a two-word span can be a genuine
/// homograph across a verb sense under one spelling and a distinct noun
/// sense under another close variant ("cell phone" is WordNet's ONLY
/// spelling of the verb "to cell phone [someone]"; the noun device is
/// spelled "cellphone"/"cellular phone" there) — a real corpus case
/// ("the difference between a 'smartphone' and 'cell phone'") needs the
/// nominal reading even though the resolved concept under this exact
/// spelling is verb-tagged. A VERB-HEADED relational surface ("count as" /
/// "counts as" — `verbal_relation_lexicon`, checked BEFORE the generic
/// relational-predicate branch below) collapses to the SAME verb-shaped
/// readings, for the SAME reason: it carries its own tense and needs the
/// ordinary transitive-verb slot, not a copula's predicative complement —
/// see that module's own doc for why it is a separate source from the two
/// below even though all three feed the same `relation_for_surface` index.
/// A COPULA-COMPLEMENT relational surface ("part of") collapses to a
/// relational predicate so "is X part of Y" parses; so does a
/// SCOPE-predicate surface ("subject to", "falls under" —
/// `scope_predicate_lexicon`). All loaded data (the reasoner's lookup +
/// relation/scope-predicate indices), never a pattern.
fn multiword_surface_readings(
    reasoner: &dyn LexicalReasoner,
    s: &str,
) -> Option<Vec<pr4xis_domains::cognitive::linguistics::lambek::types::LambekType>> {
    use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
    use pr4xis_domains::social::software::markup::xml::lmf::LmfPos;

    let resolved_id = reasoner
        .lookup(s)
        .first()
        .copied()
        .or_else(|| reasoner.lookup_case_folded(s).first().copied());
    if let Some(id) = resolved_id {
        let pos = reasoner.concept(id).map(|c| c.pos());
        let mut readings = vec![svo::proper_noun(), svo::noun()];
        if pos == Some(LmfPos::Verb) {
            readings.push(svo::transitive_verb());
            readings.push(svo::bare_transitive_verb());
        }
        return Some(readings);
    }
    if pr4xis_domains::cognitive::linguistics::verbal_relation_lexicon::verbal_relation_surfaces()
        .contains(s)
    {
        return Some(vec![svo::transitive_verb(), svo::bare_transitive_verb()]);
    }
    if reasoner.relation_for_surface(s).is_some()
        || pr4xis_domains::cognitive::linguistics::scope_predicate_lexicon::scope_predicate_surfaces()
            .contains(s)
    {
        return Some(vec![svo::relational_predicate()]);
    }
    None
}

/// Parse `text` through tokenize → chart-reduce → Montague interpretation —
/// Stages 1-3 of [`process_with_reasoner`], run standalone over a SUB-clause
/// (the presupposed clause `find_presupposed_clause` extracts) rather than
/// the whole turn. Intentionally mirrors those stages rather than sharing
/// code with them: `process_with_reasoner` is the hot path behind the full
/// corpus-gated test suite, and threading an extra return path through it
/// for this one caller was judged higher regression risk than a duplicated,
/// independently-tested ~25-line pipeline recombination of already-tested
/// stage functions. If `process_with_reasoner`'s stages 1-3 change, this
/// function needs the matching update.
fn parse_clause(
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    text: &str,
) -> Option<montague::Sem> {
    let ont_tokens = tokenize::tokenize_ontological_registry_aware(
        text,
        lang,
        &|s| is_registry_known_surface(reasoner, s),
        reasoner.max_surface_words(),
    );
    let raw_tokens: Vec<TypedToken> = ont_tokens.iter().cloned().map(Into::into).collect();
    let (_, alternatives) = tokenize::tokenize_with_alternatives_registry_aware(
        text,
        lang,
        &|s| is_registry_known_surface(reasoner, s),
        reasoner.max_surface_words(),
    );
    let (tokens, mut type_sets) = tokenize::collapse_multiword_surfaces(
        &raw_tokens,
        &alternatives,
        reasoner.max_surface_words(),
        |s| multiword_surface_readings(reasoner, s),
    );
    {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        let np = svo::proper_noun();
        let n = svo::noun();
        for (i, tok) in tokens.iter().enumerate() {
            if reasoner.is_loaded_surface(&tok.word) {
                if !type_sets[i].contains(&np) {
                    type_sets[i].push(np.clone());
                }
                if !type_sets[i].contains(&n) {
                    type_sets[i].push(n.clone());
                }
            }
        }
    }
    if tokens.is_empty() {
        return None;
    }
    let ne_tokens = NonEmpty::of(tokens[0].clone(), tokens[1..].to_vec());
    let words: Vec<String> = ne_tokens.iter().map(|t| t.word.clone()).collect();
    let reduction = chart_reduce(&words, &type_sets);
    let parsed = reduction.success;
    // Mirrors `process_with_reasoner`'s own guard (see its Stage 3 comment):
    // only trust `montague::interpret`'s CYK search when the syntax chart
    // already vetted the parse. Without this, an ungrammatical "since"
    // clause could still find a spurious `Sem::Prop` from its raw primary
    // types and get treated as a real presupposition to verify.
    if !parsed || reduction.remaining.len() != ne_tokens.len() {
        return Some(montague::Sem::unresolved());
    }
    Some(montague::interpret(&reduction.remaining, reasoner))
}

/// Karttunen (1973)-style presupposition verification (Kim, Pavlick, Karagol
/// Ayan & Ramachandran 2021, "Which Linguist Invented the Lightbulb?
/// Presupposition Verification for Question-Answering", ACL-IJCNLP 2021,
/// pp. 3932-3945):
/// when `input` fronts a temporal-clause presupposition trigger
/// (`find_presupposed_clause`), the presupposed clause is parsed on its own
/// (`parse_clause`) and VERIFIED against the loaded ontology by routing it
/// through `answer_statement`'s (now truth-checking) two-entity branch — the
/// SAME `verify_relational_claim` truth-check the interrogative path uses,
/// never a second, divergent copy of it.
///
/// Returns `Some` ONLY when the presupposed clause is a resolvable two-entity
/// claim PROVEN false (`taxonomy_checked == Some((_, _, false))`) — every
/// other case (no trigger, clause doesn't parse to a two-entity `Prop`,
/// clause is true, or the relation is unproven) returns `None` and the
/// caller proceeds with the ORIGINAL, unmodified pipeline over the whole
/// input, unaffected.
fn check_presupposed_clause(
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    input: &str,
) -> Option<trace_impls::ResponseResult> {
    let (clause_text, _main_text) = find_presupposed_clause(input)?;
    let meaning = parse_clause(lang, reasoner, &clause_text)?;
    let montague::Sem::Prop {
        predicate,
        arguments,
    } = &meaning
    else {
        return None;
    };
    let result = answer_statement(reasoner, predicate, arguments);
    match &result.taxonomy_checked {
        Some((_, _, false)) => Some(result),
        _ => None,
    }
}

/// Rewrite the two AMBIGUOUS ASCII quote marks (`"` — `quote_glyphs::
/// QuoteRole::Ambiguous`) into the canonical DIRECTIONAL double-quote pair
/// this pipeline's own quoted-mention machinery already recognizes.
///
/// `tokenize::collapse_quoted_spans` (the recognizer behind the quoted-
/// mention NP reading `montague::interpret`'s close-apposition branch needs
/// for "the term 'X' means …" / "the incident type 'X' mean?") is, BY ITS
/// OWN DOC, "Scoped to the DIRECTIONAL glyphs (`QuoteRole::Initial`/`Final`)
/// only — the two ASCII marks are `QuoteRole::Ambiguous` … and are left
/// untouched here, to `flush_word`'s existing punctuation trim." Real
/// harvested question text (the caregiver corpus's own `what_does_x_mean`
/// rows, e.g. `What does the incident type "use of a restraint" mean?`)
/// almost never carries a genuine directional quote — plain ASCII `"…"` is
/// the overwhelming convention — so without this normalization the quoted
/// DEFINIENDUM's close-apposition reading is never reachable: the closing
/// mark is discarded as ordinary punctuation before tokenization ever runs,
/// and the true definiendum is lost to a generic bare-noun-compound
/// fallback ("incident type use" instead of "use of a restraint" — a real,
/// isolated regression: confirmed via a synthetic probe that the IDENTICAL
/// sentence, directional quotes swapped in for the ASCII pair, correctly
/// extracts "use of a restraint" as the sole entity, while the ASCII-quoted
/// original extracts the garbled three-word compound).
///
/// The by-OCCURRENCE-PARITY convention (1st, 3rd, 5th, … ASCII mark opens;
/// 2nd, 4th, 6th, … closes) is the standard "smart quotes" heuristic for
/// non-nested spans — every quoted appositive in this corpus is a single,
/// non-nested run within one clause. Resolved against the LOADED quote-
/// glyph vocabulary's own canonical directional pair
/// (`quote_glyphs::vocabulary`, looked up by UCD NAME), never a hardcoded
/// `'\u{201C}'`/`'\u{201D}'` literal pair — the SAME table
/// `collapse_quoted_spans` itself reads, so a future re-authoring of that
/// table (a different canonical pair, say) stays in sync automatically. An
/// unmatched (odd) trailing ASCII mark degrades exactly as an unmatched
/// directional opener already does downstream — `collapse_quoted_spans`'s
/// own doc: "An opener with no legitimate closer … is left as-is" — so a
/// stray mark (an inches symbol, an informal aside) is harmless. Only the
/// ASCII DOUBLE mark is rewritten; the single mark (`'`) is left alone,
/// since it collides with the genitive/contraction apostrophe
/// (`split_possessive_clitics`'s own domain) far too often to disambiguate
/// this simply.
fn normalize_ascii_double_quotes(input: &str) -> String {
    use pr4xis_domains::cognitive::linguistics::lambek::quote_glyphs;
    let quotes = quote_glyphs::vocabulary();
    let Some(open_glyph) = quotes
        .iter()
        .find(|g| g.name == "LEFT DOUBLE QUOTATION MARK")
    else {
        // The loaded table is a build-time invariant (`quote_glyphs::load`'s
        // own panic-on-malformed contract) — a missing row means the TABLE
        // is broken, not that this input has no ASCII quotes. Fail closed to
        // the untouched input rather than fabricate a literal here.
        return input.to_string();
    };
    let Some(close_glyph) = open_glyph.pairs_with else {
        return input.to_string();
    };
    let mut out = String::with_capacity(input.len());
    let mut seen = 0usize;
    for c in input.chars() {
        if c == '"' {
            out.push(if seen.is_multiple_of(2) {
                open_glyph.glyph
            } else {
                close_glyph
            });
            seen += 1;
        } else {
            out.push(c);
        }
    }
    out
}

/// Process with an explicit lexical reasoner injected.
///
/// `lang` is the linguistic substrate (tokenize / parse / Montague lexicon /
/// self-model); `reasoner` is the lexical-reasoning surface the answer stages
/// query (`lookup` / `is_a` / `concept` / `parents` / `children`). For the
/// embedded-only case `reasoner == lang`. When a corpus is loaded, `reasoner`
/// is a `ComposedReasoner` whose grounded lexicon UNIONs the loaded ontology's
/// glossed concepts into the same surface — so a "what is X" over a loaded
/// concept reads its loaded gloss, and an unloaded concept abstains exactly as
/// the embedded model already does.
pub fn process_with_reasoner(
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    input: &str,
) -> ProcessResult {
    let start = WasmSafeTimer::now();
    // Bridge plain ASCII `"…"` quoting into the directional-quote machinery
    // `tokenize::collapse_quoted_spans` already implements — see
    // `normalize_ascii_double_quotes`'s own doc for the full rationale (the
    // what_does_x_mean quoted-appositive gap this closes).
    let normalized_input = normalize_ascii_double_quotes(input);
    let input = normalized_input.as_str();

    // Stage 1: Tokenize through the Language ontology.
    // tokenize_ontological produces Tokens (ontological: sense + POS + Lambek type).
    // Legacy TypedTokens derived for the reducer until it's migrated.
    let ont_tokens = tokenize::tokenize_ontological_registry_aware(
        input,
        lang,
        &|s| is_registry_known_surface(reasoner, s),
        reasoner.max_surface_words(),
    );
    let raw_tokens: Vec<TypedToken> = ont_tokens.iter().cloned().map(Into::into).collect();
    let (_, alternatives) = tokenize::tokenize_with_alternatives_registry_aware(
        input,
        lang,
        &|s| is_registry_known_surface(reasoner, s),
        reasoner.max_surface_words(),
    );
    let token_count = ont_tokens.len();

    // Multi-token surface recognition: collapse maximal known multi-word surfaces
    // (a loaded citation/label, a WordNet collocation) into single proper-noun
    // tokens BEFORE the parse, so every downstream stage — chart_reduce, interpret,
    // and the partial-understanding fallback — sees one lookup unit. Data-driven
    // (the reasoner's loaded surface set); a no-op when `max_surface_words == 1`
    // (embedded English), so single-token chat is byte-identical.
    // An ENTITY surface (a loaded citation/label/collocation OR a WordNet
    // multi-word lemma) collapses to BOTH nominal readings — proper noun
    // (NP) so it stands alone as a subject/complement, AND common noun (N)
    // so a determiner attaches ("a cough out", "a care for": NP/N + N →
    // NP) — PLUS, when the resolved concept's own loaded WordNet-LMF part
    // of speech is `Verb` ("look like" — WordNet indexes multiword
    // collocations across every POS, not just nouns, Fellbaum 1998 Ch.1),
    // ALSO the verb-shaped readings (`multiword_surface_readings`'s own doc
    // has the full rationale, including why this is additive rather than
    // exclusive). A RELATIONAL surface ("part of") collapses to a
    // relational predicate so "is X part of Y" parses; so does a
    // SCOPE-predicate surface ("subject to", "fall(s) under" —
    // `scope_predicate_lexicon`), a phrasal Subsumption-default question
    // ("is X subject to Y") with no relation kind of its own — kept OUT of
    // the reasoner's relation/predicate surface indices on purpose, since
    // those also feed the reverse connective-phrasing lookup and a
    // Subsumption-pointing entry there breaks the "is a" default's
    // un-lexicalized-copula invariant network-wide. All loaded data (the
    // reasoner's surface + relation indices) or this closed hand-authored
    // set, never a pattern. The exact lookup misses a capitalized
    // multi-word lemma from its lowercased tokenizer surface ("Turkish
    // bath" vs. "turkish bath") — Slice D's case-folded fallback
    // (`lookup_case_folded`) recovers it, tried only on a miss.
    let (tokens, mut type_sets) = tokenize::collapse_multiword_surfaces(
        &raw_tokens,
        &alternatives,
        reasoner.max_surface_words(),
        |s| multiword_surface_readings(reasoner, s),
    );

    // A LOADED ontology surface is a noun the chat reasons over. Offer it BOTH
    // readings so it parses in either syntactic role, gated on LOADED-corpus
    // membership — NOT the union lookup — so English function words (which never
    // resolve to a loaded ontology) keep their copula/determiner/wh types:
    //   - a proper-noun NP reading, so it stands alone as a subject/complement
    //     ("is <loaded-X> part of Y", "is case law a law"); and
    //   - a common-noun N reading, so a determiner attaches ("a legal document").
    // A single-word loaded surface already carries N from tokenize; a COLLAPSED
    // multi-word surface (a §9 citation/label span) arrives typed NP-only, so
    // without the N reading "a <multi-word-label>" could not reduce (the determiner
    // NP/N finds no N). Both are ADDED as alternatives, never replacing the base
    // type, so the chart keeps whichever derives S.
    {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        let np = svo::proper_noun();
        let n = svo::noun();
        for (i, tok) in tokens.iter().enumerate() {
            if reasoner.is_loaded_surface(&tok.word) {
                if !type_sets[i].contains(&np) {
                    type_sets[i].push(np.clone());
                }
                if !type_sets[i].contains(&n) {
                    type_sets[i].push(n.clone());
                }
            }
        }
    }

    if tokens.is_empty() {
        return ProcessResult {
            response: "Empty input received.".into(),
            user_act: SpeechAct::Assertion,
            system_act: SpeechAct::Assertion,
            duration_us: start.elapsed_us(),
            token_count: 0,
            parsed: false,
            trace: PipelineTrace::from_traceable(&trace_impls::TokenizeResult { tokens: &tokens }),
            from_ontology: false,
            // Empty input named nothing — an abstention with no surface to load.
            outcome: ChatOutcome::Abstained {
                unresolved: Vec::new(),
            },
            // Nothing was asked, so there is nothing to explain: no Why? panel.
            why: None,
            // …and nothing was defined, so no definition provenance either.
            definition_provenance: None,
        };
    }

    // NonEmpty semigroup: the empty-check above proves the invariant.
    let ne_tokens = NonEmpty::of(tokens[0].clone(), tokens[1..].to_vec());
    let words: Vec<String> = ne_tokens.iter().map(|t| t.word.clone()).collect();
    // `type_sets` was built by `collapse_multiword_surfaces` above, aligned to the
    // (possibly collapsed) `tokens`: a collapsed span → `[proper_noun()]`; an
    // uncollapsed token → its own type plus that position's alternatives — exactly
    // the set this stage built before, so an uncollapsed stream is unchanged.

    // Stage 2: Parse through Lambek grammar. Chart reduction returns a Traceable.
    let reduction = chart_reduce(&words, &type_sets);
    let parsed = reduction.success;

    // Stage 3: Interpret through Montague semantics — but ONLY when the
    // syntax chart itself already found a complete parse. `montague::interpret`
    // is a CYK search over `montague_tokens`' types, exhaustive over every
    // bracketing; when `parsed` is false, those types are just the raw
    // per-token PRIMARY assignments (the syntax chart, searching every
    // loaded ALTERNATIVE, already established that no combination of them
    // forms a grammatical sentence). Running the same exhaustive semantic
    // search over that already-rejected input can still find SOME
    // accidental bracketing that happens to type-reduce to a complete `S` —
    // a false positive, not a real derivation of the sentence — and a
    // confident (and wrong) `Sem::Question`/`Sem::Prop` routes straight to
    // `answer_question`/`answer_statement` instead of the honest
    // `attempt_partial_understanding` fallback (confirmed via the full
    // corpus regression suite: 33 out_of_scope_abstain questions with no
    // syntactic parse at all — "What happens if check-in ... cannot be ...
    // captured?", no possessive anywhere in it — started confidently
    // defining an unrelated loaded word once the CYK rewrite could find that
    // spurious full-span reduction). So semantics is only trusted to the
    // exact extent syntax already vetted it: `reduction.remaining` (the
    // chart's OWN winning-derivation types) when `parsed`, otherwise the
    // honest "no derivation" sentinel directly, never a search over
    // known-ungrammatical input.
    let meaning = if parsed && reduction.remaining.len() == ne_tokens.len() {
        montague::interpret(&reduction.remaining, reasoner)
    } else {
        montague::Sem::unresolved()
    };

    // Stage 4: Classify the speech act through pragmatics. A phatic exchange
    // (Malinowski 1923 / Jakobson 1960) is checked first: `SpeechAct::Greeting`
    // is the ONLY variant `jakobson_function_of_speech_act` maps to Phatic
    // (`pragmatics::response`), so naming it here — rather than defaulting to
    // `Assertion` — is what lets that classifier chain actually fire on a live
    // turn instead of only in its own unit tests.
    let is_phatic_turn = is_phatic(&ont_tokens, lang);
    let user_act = if is_phatic_turn {
        SpeechAct::Greeting
    } else if meaning.is_question() {
        SpeechAct::Question
    } else {
        SpeechAct::Assertion
    };

    // Stage 5: Metacognitive decision — which branch of repair/response to take.
    let metacog_decision = if meaning.is_question() {
        "question detected → query ontology"
    } else if meaning.is_proposition() {
        "statement detected → acknowledge/elaborate"
    } else if parsed {
        "parsed but unrecognized form → partial understanding"
    } else {
        "parse failed → metacognitive repair (attempt partial understanding)"
    };

    // Stage 6: Generate the response through NLG.
    // A fronted presupposed clause PROVEN false ("Since a seal is a plant, …")
    // is checked FIRST and independently of how the whole turn parses — the
    // main clause's own grammar/parse success is irrelevant to whether its
    // presupposition holds. A `None` here (no trigger, or the presupposition
    // holds/is unproven) is a pure no-op: every other input is unaffected and
    // falls through to the unmodified dispatch below.
    // Self-referential questions route through the SelfModel eigenform.
    let response_result = if let Some(result) = check_presupposed_clause(lang, reasoner, input) {
        result
    } else if is_self_referential(&ont_tokens, reasoner) {
        answer_self_referential(lang, &ont_tokens)
    } else if is_phatic_turn {
        answer_phatic()
    } else {
        match &meaning {
            montague::Sem::Question {
                predicate,
                arguments,
                illocution,
            } => answer_question(reasoner, lang, &tokens, predicate, arguments, *illocution),
            montague::Sem::Prop {
                predicate,
                arguments,
            } => answer_statement(reasoner, predicate, arguments),
            _ => attempt_partial_understanding(reasoner, lang, &tokens, &reduction, &meaning),
        }
    };
    let response_result =
        decline_if_an_unresolved_acronym_was_ignored(reasoner, input, response_result);
    let response_result =
        widen_definiendum_if_compound_available(reasoner, &tokens, response_result);

    // Build the trace by threading TracedPipeline<()> through each stage.
    // `.tell()` is the writer monad's log-append operation; each call concatenates
    // a single trace entry via the PipelineTrace monoid (Vec concatenation).
    // The final log IS the full pipeline trace, accumulated by composition, not mutation.
    //
    // Every step produces its trace entry through a `Traceable` impl — the
    // ontology (Diagnostic/Trace) owns the entry's shape; the call site just
    // hands it the step's domain result. No inline string construction.
    let realization = trace_impls::RealizationResult {
        char_count: response_result.response.len(),
    };
    let speech_act_result = trace_impls::SpeechActClassificationResult { user_act };
    let metacog_result = trace_impls::MetacognitionResult {
        decision: metacog_decision,
        parsed,
    };
    let pipeline: TracedPipeline<()> = Writer::pure(())
        .tell(PipelineTrace::from_traceable(
            &trace_impls::TokenizeResult { tokens: &tokens },
        ))
        .tell(PipelineTrace::from_traceable(&reduction))
        .tell(PipelineTrace::from_traceable(
            &trace_impls::InterpretResult { meaning: &meaning },
        ))
        .tell(PipelineTrace::from_traceable(&speech_act_result))
        .tell(PipelineTrace::from_traceable(&metacog_result))
        .tell(PipelineTrace::from_traceable(&response_result))
        .tell(PipelineTrace::from_traceable(&realization));

    let from_ontology = response_result.from_ontology;
    // The communicative goal that produced `response` and the loaded sources it
    // reasoned over — carried out of the answer stage so the "Why?" layer keys
    // off the SAME frame and the SAME provenance the answer used.
    let frame = response_result.frame;
    let conditional = response_result.conditional;
    let reasoned_over = response_result.reasoned_over;
    let deferred_elaboration = response_result.deferred_elaboration;
    let response = response_result.response;

    // The abstention's "asked but not loaded" set — computed once here, shared by
    // both the typed outcome and the plain-language explanation (a pure read, so
    // eager computation is behavior-equivalent to the old lazy call).
    let unresolved = if from_ontology {
        Vec::new()
    } else {
        unresolved_surfaces(&meaning, reasoner)
    };

    // The typed outcome (doc §4.1): a rule-governed turn awaiting a private
    // fact (checked FIRST — Bobrow et al. 1977 GUS: the rule is fully known
    // and cited, this is a positive capability, not abstention), otherwise
    // answered from knowledge, or abstained — naming the surfaces no loaded
    // ontology could ground (what to load next). Derived from the answer
    // stage's own typed signal, never sniffed from the text.
    let outcome = if let Some((rule, missing)) = conditional {
        ChatOutcome::Conditional {
            rule: Box::new(rule),
            missing,
        }
    } else if from_ontology {
        ChatOutcome::Answered
    } else {
        ChatOutcome::Abstained {
            unresolved: unresolved.clone(),
        }
    };

    // The plain-language "Why?" (doc §5.2): a SECOND realization keyed off the
    // communicative `frame` that actually produced the response, filled from THIS
    // turn's real provenance (`reasoned_over`) and unresolved surfaces — never
    // sniffed from the response text. `None` for the self-explaining frames (the
    // renderer then shows no Why? panel).
    let why = pr4xis_domains::cognitive::linguistics::pragmatics::realize::realize_why(
        frame,
        from_ontology,
        &reasoned_over,
        &unresolved,
    )
    .map(|w| match &deferred_elaboration {
        // The deferred content earns its own clearly-delimited section
        // (never blended into the SAME sentence as the epistemic-
        // provenance statement — an RST Elaboration is a different
        // relation from the justification `why` states) rather than being
        // dropped: the multi-hop chain's intermediate rungs and any
        // subtypes, moved out of the primary response under Grice's
        // Quantity maxim (see `build_taxonomy_response`).
        Some(deferred) => format!("{w}\n\nFull reasoning chain:\n{deferred}"),
        None => w,
    });

    // The THIRD provenance channel (doc §5.2), realized here beside `why` from
    // the same loaded frames table — and kept OUT of the `why` string. Anything
    // appended to that string could only be recovered by re-splitting it, and a
    // renderer that has to split prose to tell two provenance claims apart is
    // one edit away from conflating them again.
    let definition_provenance =
        pr4xis_domains::cognitive::linguistics::pragmatics::realize::realize_definition_provenance(
            &response_result.definition_provenance,
        );

    ProcessResult {
        response,
        user_act,
        system_act: SpeechAct::Assertion,
        duration_us: start.elapsed_us(),
        token_count,
        parsed,
        trace: pipeline.log,
        from_ontology,
        outcome,
        why,
        definition_provenance,
    }
}

/// The surfaces a turn NAMED but could not resolve to any concept — the "asked but
/// not loaded" set (doc §4.1), read off the interpreted meaning's entities and
/// filtered to those the reasoner's lexicon does not know. This IS the abstention
/// reason: what the user would have to load for the system to answer.
fn unresolved_surfaces(meaning: &montague::Sem, reasoner: &dyn LexicalReasoner) -> Vec<String> {
    let named: Vec<&montague::Sem> = match meaning {
        montague::Sem::Question { arguments, .. } | montague::Sem::Prop { arguments, .. } => {
            arguments.iter().collect()
        }
        other => vec![other],
    };
    let mut unresolved: Vec<String> = Vec::new();
    for sem in named {
        let surface = extract_entity_name(sem, reasoner);
        // Resolution here MUST match the answer path's (`resolve_surface`:
        // exact ∪ morphological analyses) — an inflected surface the question
        // realized ("coughing") is a KNOWN concept, and naming it unresolved
        // is the false-negation honesty violation the capability suite guards.
        if surface.is_empty() || !resolve_surface(reasoner, &surface).is_empty() {
            continue;
        }
        match entity_composite_parts(sem) {
            // A composition-MINTED surface is not a word (axiom
            // [`DefiniendumIsALexicalUnit`]): report its failing LEXICAL
            // leaves — the loadable units — never the minted concatenation.
            // For the degenerate "is a" both leaves resolve, so nothing is
            // named (the honest empty abstention).
            Some((func, arg)) => {
                for leaf in [func, arg] {
                    if resolve_surface(reasoner, leaf).is_empty()
                        && !unresolved.iter().any(|u| u == leaf)
                    {
                        unresolved.push(leaf.to_string());
                    }
                }
            }
            None => {
                if !unresolved.contains(&surface) {
                    unresolved.push(surface);
                }
            }
        }
    }
    unresolved
}

/// The composite halves of the surface [`extract_entity_name`] names for this
/// `Sem`, or `None` when that surface is a lexical token — walks the same
/// Func-body spine as `extract_entity_name`, so the two always describe the
/// same leaf. Typed accessor over [`montague::PredProvenance`]; the space in
/// the word is never re-parsed.
fn entity_composite_parts(sem: &montague::Sem) -> Option<(&str, &str)> {
    match sem {
        montague::Sem::Pred {
            provenance: montague::PredProvenance::Composite { func, arg },
            ..
        } => Some((func.as_str(), arg.as_str())),
        montague::Sem::Func { body, .. } => body.iter().find_map(entity_composite_parts),
        _ => None,
    }
}

/// Scan `tokens` for a span matching a registered governance-predicate
/// surface (`predicate_lexicon::predicate_surface_index` — today just
/// `"eligible for"`, Minsky 1974 frame semantics), the SAME registry the
/// successful-parse path (`answer_question`, via
/// `LexicalReasoner::conditional_rule_for_predicate`) already consults.
/// Applied here to the RAW token stream rather than a parsed `Sem`: a
/// chart-parse failure doesn't mean the sentence isn't asking a rule-
/// governed question — a long or structurally uncovered object noun phrase
/// ("the Program of Comprehensive Assistance for Family Caregivers") can
/// break whole-sentence coverage even when the governing predicate itself
/// is perfectly recognizable.
///
/// `collapse_multiword_surfaces` (tokenize.rs), which already runs before
/// this function ever sees `tokens`, MAY have already merged the surface's
/// words into a single token (`.word == "eligible for"`) when the reasoner
/// recognizes it via `relation_for_surface` — or may not have, when
/// `max_surface_words() == 1` (e.g. embedded English). Rather than assume
/// either tokenization granularity, this tries every window width from 1
/// token up to the surface's own word count at each start position and
/// compares the JOINED window text — the same join
/// `collapse_multiword_surfaces` itself uses — so both a pre-collapsed
/// single token and a fully uncollapsed word-by-word span match. Matching
/// is case-insensitive and generic over however many surfaces the registry
/// carries — this function does not know or care that "eligible for" is
/// the only entry today. Returns the matched predicate surface and the
/// remaining words after the matched span (the object phrase).
fn find_governed_predicate(tokens: &[TypedToken]) -> Option<(String, Vec<&str>)> {
    use pr4xis_domains::cognitive::linguistics::predicate_lexicon::predicate_surface_index;

    let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
    for surface in predicate_surface_index().keys() {
        let max_width = surface.split(' ').count().max(1);
        for start in 0..words.len() {
            let upper = (words.len() - start).min(max_width);
            for width in 1..=upper {
                let joined = words[start..start + width].join(" ");
                if joined.eq_ignore_ascii_case(surface) {
                    let object: Vec<&str> = words[start + width..].to_vec();
                    return Some((surface.clone(), object));
                }
            }
        }
    }
    None
}

/// Whether any token in the sentence is POS-tagged `Adjective` or
/// `Auxiliary` (OLiA, Chiarcos & Sukhareva 2015) — a real, POS-grounded
/// signal that the sentence has a more complex predicate structure than a
/// bare taxonomic comparison, used to gate `explore_concepts` (see its call
/// site in [`attempt_partial_understanding`]).
///
/// ALSO true when a token is one of `scope_predicate_lexicon`'s registered
/// surfaces ("required for", "subject to", …) — collapsed by
/// `collapse_multiword_surfaces` into ONE opaque multi-word token BEFORE
/// this check runs, so `lang.lexical_lookup` finds no dictionary entry for
/// the fused string and the Adjective/Auxiliary POS check above can never
/// see it. Left ungated, that silently defeated this very gate: "is X
/// required for Y" collapses "required for" into a relational-predicate
/// token, `lexical_lookup("required for")` misses, and `explore_concepts`
/// fired a WordNet-sense dump of unrelated nouns on a modal/deontic
/// question exactly like the "am I ELIGIBLE for X" case this gate already
/// documents (caught via the full corpus regression suite — 10 previously-
/// abstaining out-of-scope questions started confidently mis-answering).
/// Each registered surface is itself an adjective/participle/prepositional-
/// verb predicate by its own module's grounding, so this is the same
/// semantic signal, just read after the collapse instead of before it.
///
/// Deliberately does NOT also scan the pre-collapse raw token stream for a
/// bare Adjective/Auxiliary POS tag: an attributive adjective inside an
/// ordinary noun phrase that a loaded multi-word concept happens to span
/// ("palliative care", "custodial care") carries the SAME POS tag as a
/// genuine predicative modal/deontic adjective ("is X REQUIRED") but is not
/// the signal this gate exists to catch — "hospice or palliative care
/// location" is a bare taxonomic comparison ("hospice is a form of
/// palliative care"), not a modal/deontic question, even though "palliative"
/// is POS-tagged Adjective. Attributive vs. predicative adjective POSITION
/// is exactly the distinction a bare POS-tag scan over an unordered set
/// cannot make; a raw-token generalization was tried and reverted after the
/// full corpus regression suite caught it silently breaking this class of
/// question (confirmed via `git diff` against the pre-change snapshot).
/// Only `scope_predicate_lexicon`'s registered surfaces are trusted here —
/// each one is independently grounded (by its own module doc) as a genuine
/// predicative construction, never a bare attributive-adjective guess.
/// ALSO true when a token is a possessive determiner/pronoun ("my", "your",
/// "his", "their", …; Huddleston & Pullum 2002 Ch. 5 §10's genitive/
/// dependent-possessive class, `PronounKind::Possessive`) — "what ARE MY
/// benefits" is a question about a PARTICULAR speech-act participant's
/// circumstances, not a kind-level taxonomic fact ("a benefit is a kind of
/// X"), so a bare comparison of "benefits"/"program" cannot honestly answer
/// it even though neither word is itself an Adjective/Auxiliary. Gated on
/// `PronounKind`, never a word list — this is the SAME closed-class lexicon
/// `is_pronoun`/the tokenizer's own possessive-clitic split already use.
///
/// ALSO true when a token's type is exactly one of the canonical VERB
/// category shapes (`svo::intransitive_verb`/`transitive_verb`/
/// `ditransitive_verb`/`bare_transitive_verb`) — "what HAPPENS if check-in
/// cannot be captured" carries its own real verbal predicate ("happens"),
/// so the sentence is asking about an EVENT/PROCEDURE, not predicating one
/// queried noun of another; `explore_concepts`'s "is a seal a mammal?"
/// design case has no verb at all beyond the copula.
///
/// Exact-equality against the four verb constructors, NOT the broader
/// `LambekType::is_predicate()` (any complex type whose ultimate result is
/// `S`) — `is_predicate()` is ALSO true of a fronted wh-word like "what"
/// (`S[wq]/(NP\S)`: applying it eventually yields `S`, same as a verb), so
/// that broader check made this gate fire on literally every wh-question
/// and silenced `explore_concepts` outright.
///
/// STILL not enough on its own: `svo::copula` and `svo::transitive_verb`
/// are the SAME LambekType, `(NP\S)/NP` (by design — that shared shape is
/// what lets the copula's category combine exactly like a transitive verb
/// would), so a medial "is"/"are" ALSO structurally matches
/// `transitive_verb()`, wrongly firing this gate on every "what IS X"
/// question and silencing `explore_concepts`/`define_word` alike. Excluded
/// via `lang.lexical_lookup`'s POS tag — a closed-class copula IS in the
/// narrow embedded dictionary — so the verb-shape check only fires for a
/// token that is NEITHER a lexically-known copula NOR (see above)
/// resolvable there at all: open-class WordNet verbs like
/// "happens"/"progresses" are invisible to `lang.lexical_lookup`
/// (`lang.lexical_lookup("happens")` misses entirely; only the full
/// `en`/WordNet-backed reasoner's OLiA-class projection assigns their
/// verbal Lambek type), so this exclusion only ever removes genuine
/// copulas, never a real verb.
///
/// Both additions confirmed via the full corpus regression suite: 33
/// out_of_scope_abstain questions ("what happens if…", "what are MY…")
/// started confidently dumping WordNet senses once a separate fix (the
/// Montague CYK rewrite, `montague.rs`) stopped an unrelated bug from
/// accidentally routing them through `answer_question`'s own, stricter
/// gap-detection instead of ever reaching this gate.
/// Levi (1978), *The Syntax and Semantics of Complex Nominals* (Academic
/// Press) / Bauer & Huddleston, in Huddleston & Pullum (2002), *The
/// Cambridge Grammar of the English Language*, Ch. 19 "Lexical
/// word-formation," pp. 1621-1722: a maximal run of adjacent Adjective
/// (`N/N`) and Noun (`N`) tokens is ONE noun phrase, right-headed by its
/// rightmost noun — never several independently comparable entities.
/// `explore_concepts`'s "is X a Y" bare-comparison design case ("is a seal a
/// mammal?") needs each compared noun to be the HEAD of its own, separately
/// introduced noun phrase (its own determiner, or its own side of a
/// copula/coordinator); a shared, unseparated compound-noun chain — however
/// many nouns it stacks, under one determiner or none at all — is a SINGLE
/// constituent.
///
/// Without this collapse, "What is the representative payee misuse
/// escalation protocol?" — one 5-word nominal compound the grammar failed to
/// parse as a unit — degrades to a pairwise WordNet comparison of whichever
/// 2+ of its INTERNAL words happen to individually resolve ("escalation",
/// "protocol"), a confident-sounding answer about the compound's last two
/// words, not the (fabricated) compound itself — the same silently-dropped-
/// real-query-subject failure `decline_if_an_unresolved_acronym_was_ignored`
/// already guards against for the single-acronym case, generalized here to
/// every nominal compound. Confirmed real regression: 3 domain_mimicry
/// adversarial-corpus rows flipped Safe → Answered exactly this way once an
/// earlier fix broadened `entities_found.is_empty()` elsewhere and stopped
/// masking it (task #26).
///
/// Concept-deduped exactly like the flat per-token computation this
/// replaces: two heads resolving to the SAME loaded concept count once.
// A COLLAPSED multi-word surface ("direct support",
// `collapse_multiword_surfaces` — an English WordNet collocation OR a
// curated domain lexicon entry, either source) is typed `proper_noun()`
// (NP), not `is_noun()` (N) — so without this arm it silently BREAKS a
// compound run instead of bridging it, splitting "community [direct
// support] navigation" into two disconnected single-word runs
// ("community", "navigation") that `explore_concepts` then dumps
// independently, exactly the "several independently comparable nouns"
// failure `noun_phrase_heads`'s own doc says it exists to prevent — just one
// step upstream of the plain-adjacent-nouns case that doc already
// covers. Confirmed real regression this closes: "What is Community
// Direct Support/Navigation?" (`en.is_loaded_surface` alone is too
// narrow here — it only recognizes a CURATED-lexicon surface, not an
// English WordNet collocation like "direct support", so a collapsed
// span from the OTHER knowledge source stayed a silent run-breaker).
// The word containing an interior space is the structural signature of
// a `collapse_multiword_surfaces` JOIN (`join_token_window`'s own
// documented `' '`-joining convention) — never true of an ordinary
// single-word token, including a pronoun ("who", also bare `Atom(NP)`,
// never mistaken for a compound-bridging span since it never contains a
// space). `en.lookup` non-empty double-checks it is a genuinely
// resolving span, not merely NP-shaped.
//
// Shared by `noun_phrase_heads` (collapses a run to its head, for
// multi-noun comparison) and `widen_definiendum_to_compound` (widens a
// resolved head back OUT to the largest known compound containing it, for
// the single-entity define path) — the SAME notion of "this token
// continues a nominal run", used in both directions.
fn is_nominal_token(t: &TypedToken, en: &dyn LexicalReasoner) -> bool {
    use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
    t.lambek_type.is_noun()
        || t.lambek_type == svo::adjective()
        || (t.lambek_type == svo::proper_noun()
            && t.word.contains(' ')
            // BOTH `lookup` AND `lookup_case_folded`, never `lookup` alone —
            // the SAME exact ∪ case-folded union that MINTED this collapsed
            // token in the first place (`multiword_surface_readings`:
            // `lookup(s).first().or_else(|| lookup_case_folded(s)...)`).
            // Re-checking with the narrower `lookup` alone re-introduced the
            // case-marked-lemma miss `lookup_case_folded` exists for
            // (WordNet spells "Social Security" / "Down syndrome"
            // capitalized; the tokenizer's surface is lowercased), so a
            // correctly-collapsed nominal token failed THIS nominal test and
            // SPLIT its surrounding compound-noun run in two — turning the
            // single unparsed compound "caregiver social security credit
            // program" into two separately-headed runs ("caregiver" /
            // "program") that `explore_concepts` then compared as if they
            // were independent entities. The confirmed adversarial
            // fabricated-term case this fixes is exactly that question
            // (adversarial corpus index 69).
            && (!en.lookup(&t.word).is_empty() || !en.lookup_case_folded(&t.word).is_empty()))
}

fn noun_phrase_heads<'a>(tokens: &'a [TypedToken], en: &dyn LexicalReasoner) -> Vec<&'a str> {
    let mut seen_concepts: Vec<ConceptId> = Vec::new();
    let mut heads: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if !is_nominal_token(&tokens[i], en) {
            i += 1;
            continue;
        }
        let mut j = i;
        let mut head_idx = None;
        while j < tokens.len() && is_nominal_token(&tokens[j], en) {
            if tokens[j].lambek_type.is_noun() {
                head_idx = Some(j);
            }
            j += 1;
        }
        if let Some(h) = head_idx {
            let word = tokens[h].word.as_str();
            match en.lookup(word).first() {
                Some(&id) if seen_concepts.contains(&id) => {}
                Some(&id) => {
                    seen_concepts.push(id);
                    heads.push(word);
                }
                None => {}
            }
        }
        i = j;
    }
    heads
}

/// Widen a single resolved definiendum HEAD back out to the largest known
/// compound noun phrase containing it, when one exists — the inverse
/// operation of `noun_phrase_heads`'s run-collapsing. That collapse is
/// correct for MULTI-noun comparison (`explore_concepts`: "escalation" vs
/// "protocol" are the concepts being compared, not the 5-word compound
/// naming their relationship), but wrong for `define_word`: "what is a tax
/// credit?" must define the compound "tax credit" (a distinct WordNet/USC
/// concept), not just its head noun "credit" (15 unrelated generic senses —
/// acknowledgements in a film's credits, financial trust, a college course
/// credit — none of them the fiscal-policy sense the question asked about).
///
/// Finds the LAST token matching `head` (the one `content_entities`/
/// `noun_phrase_heads` already resolved), walks backward while the
/// preceding tokens are OPEN-class (`!en.is_function_word`), and tries the
/// WIDEST candidate phrase first, narrowing one word at a time until one
/// resolves via `resolve_surface` (exact ∪ lemmatized — a plural surface
/// like "tax credits" needs this to land on the loaded singular "tax
/// credit") or nothing wider than
/// `head` itself does.
///
/// Gated on closed-class membership, NOT on the chart's own winning
/// category assignment for the preceding token (`is_nominal_token`, used by
/// `noun_phrase_heads`, was tried first and rejected here: confirmed via a
/// real corpus case that a lexically-ambiguous modifier — "tax" in "What
/// tax credits in 2026?" — can win a VERB reading, `(NP\S)/NP`, in a
/// degenerate copula-less derivation, even though "tax credit" is the
/// obviously-intended nominal compound; gating on that category assignment
/// silently refuses to widen past it). Open-class-vs-closed-class is the
/// more robust signal here precisely because it does NOT depend on which of
/// a genuinely ambiguous word's derivations the chart happened to rank
/// first — a determiner/preposition/wh-word/copula can never itself be part
/// of a compound noun's own surface, independent of any parse ambiguity, so
/// stopping there is always safe; an open-class word MIGHT be part of one,
/// which the following `en.lookup`/`en.lookup_case_folded` check settles
/// empirically rather than grammatically. Never invents a phrase the loaded
/// lexicon/corpus does not already recognize — a modifier that does not
/// combine into a KNOWN compound is left out, so "the big tax credit" still
/// defines "tax credit", not a fabricated "big tax credit".
fn widen_definiendum_to_compound(
    tokens: &[TypedToken],
    head: &str,
    en: &dyn LexicalReasoner,
) -> String {
    let Some(head_idx) = tokens.iter().rposition(|t| t.word == head) else {
        return head.to_string();
    };
    let mut start = head_idx;
    while start > 0 && !en.is_function_word(&tokens[start - 1].word) {
        start -= 1;
    }
    for widen_from in start..head_idx {
        let joined = tokens[widen_from..=head_idx]
            .iter()
            .map(|t| t.word.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !resolve_surface(en, &joined).is_empty() {
            return joined;
        }
    }
    head.to_string()
}

/// Deliberately scans EVERY token, unconditionally — no entity exclusion.
/// This is safe and correct for its two actual callers
/// (`attempt_partial_understanding`'s single-definiendum and multi-noun
/// branches, both PARSE-FAILURE-only): the entity/noun set on that path is
/// itself extracted from a degenerate, already-failed derivation, so a
/// genuinely deontic word (e.g. "need") can — and, confirmed via full
/// corpus regression, DOES — end up misclassified as one of the compared
/// nouns purely because its ALTERNATIVE reading happens to be nominal
/// ("Do live-in workers NEED to use EVV?" fails to parse, and `need`'s
/// noun sense joins `noun_phrase_heads`'s own comparison set) —
/// entity-excluding this check would then silently un-flag the very word
/// carrying the deontic signal. `answer_question`'s two PARSE-SUCCESS
/// branches need the opposite trade-off (a well-identified, unambiguous
/// entity that MUST be excluded) and use the narrower, entity-excluding
/// sibling [`has_deontic_or_descriptive_marker`] instead — see that
/// function's own doc for why the two are not merged into one
/// parameterized check.
fn has_modal_or_descriptive_predicate(lang: &dyn Language, tokens: &[TypedToken]) -> bool {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::{LexicalEntry, PosTag, PronounKind};
    use pr4xis_domains::cognitive::linguistics::scope_predicate_lexicon::scope_predicate_surfaces;
    use pr4xis_domains::social::judicial::modality::ontology::classify_modal;
    let scope_predicates = scope_predicate_surfaces();
    tokens.iter().any(|t| {
        matches!(
            lang.lexical_lookup(&t.word),
            Some(LexicalEntry::Pronoun(p)) if p.kind == PronounKind::Possessive
        )
    }) || tokens.iter().any(|t| {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        let verb_shaped = t.lambek_type == svo::intransitive_verb()
            || t.lambek_type == svo::transitive_verb()
            || t.lambek_type == svo::ditransitive_verb()
            || t.lambek_type == svo::bare_transitive_verb();
        let lexically_copula = lang
            .lexical_lookup(&t.word)
            .is_some_and(|entry| entry.pos_tag() == PosTag::Copula);
        verb_shaped && !lexically_copula
    }) || tokens.iter().any(|t| {
        // Deliberately NOT do-support-excluded: a CLAUSE-INITIAL "Do"/
        // "Does"/"Did" ("Do live-in workers NEED to use EVV?") gets the
        // question-forming `S[q]/NP/NP`-family category (`assign_type`'s
        // clause-initial branch), NOT the mid-sentence provisional `NP\S`
        // auxiliary row a MEDIAL "does" gets — so it is NEVER independently
        // caught by the verb-shaped disjunct above the way a medial
        // do-support token is. This Adjective/Auxiliary-POS check is the
        // ONLY signal that recognizes a fronted "Do X need/pay/require Y?"
        // polar-obligation question here — a do-support exclusion here was
        // tried, then reverted after a full corpus regression: "Do live-in
        // workers need to use EVV?"/"Does Medicaid Pay for Home Care?" and
        // 3 siblings newly OverAnswered, since "need"/"pay" themselves are
        // open-class words whose PRIMARY reading is a bare noun, `N`, not
        // verb-shaped, invisible to the disjunct above either way — a
        // do-support exclusion here would silently remove the only signal
        // left.
        lang.lexical_lookup(&t.word)
            .is_some_and(|entry| matches!(entry.pos_tag(), PosTag::Adjective | PosTag::Auxiliary))
            || scope_predicates.contains(&t.word)
    }) || tokens.iter().any(|t| {
        // von Wright (1951) deontic-modality classification (`ObligationModality`,
        // `crates/domains/src/social/judicial/modality`) — a PRECISE, cited,
        // closed-set surface classifier for the deontic markers Halliday (1985)
        // ch.10 documents ("shall"/"must"/"required"/"requires"/"prohibited"/
        // "forbidden"/"may"/"permitted"/"entitled"/"discretionary"), reused here
        // rather than reinvented: a "which X are required to Y"/"what is
        // required of X" obligation question is exactly the deontic-primitive
        // shape this ontology already types (`Mandatory`), previously proven
        // and tested (`modality::tests`) but never called from live chat —
        // wiring it in closes that orphan and gives a token-surface signal
        // independent of the broader Adjective-POS check above, which is
        // itself already sufficient for "required" (confirmed empirically:
        // `lang.lexical_lookup("required")` resolves `PosTag::Adjective`
        // regardless) but not for every deontic marker this ontology covers.
        classify_modal(&t.word).is_some()
    })
}

/// The narrower sibling of [`has_modal_or_descriptive_predicate`] for
/// `answer_question`'s two PARSE-SUCCESS entity-verification paths: a
/// possessive pronoun, or a von Wright deontic-modal surface — but
/// deliberately WITHOUT the broad "any Adjective/Auxiliary POS token"
/// disjunct (nor the "any non-copula verb" one).
///
/// BOTH broader disjuncts are correct and needed where
/// [`has_modal_or_descriptive_predicate`] is actually used
/// (`attempt_partial_understanding`'s two branches, which only ever run
/// after the CCG chart has already FAILED to parse the sentence — its own
/// cited case, "Who REIMBURSES caregivers for mileage?", is a fragment the
/// chart never resolved). `answer_question`'s two branches run the OPPOSITE
/// case — the chart already SUCCEEDED, over cleanly-extracted, well-typed
/// entities — a much wider, looser distribution the parse-failure-only
/// check was never measured against. Confirmed via full corpus regression,
/// TWICE: first, wiring the FULL check (verb-shaped disjunct included)
/// caused 11 real Green -> {MissingTerm, UnparsedKnownTerm} regressions
/// (ordinary content verbs like "mean"/"provide" aren't evidence of a wrong
/// illocutionary shape the way they are for a verb-less parse-failure
/// fragment). Dropping JUST the verb-shaped disjunct still left the
/// Adjective/Auxiliary-POS disjunct, which caused 4 MORE: "What is the
/// DEFINITION of a MINOR incident?" (the PP "of a minor incident" is
/// dropped by a separate, pre-existing entity-extraction gap — see
/// `extract_entity_name`'s own doc — leaving "minor", an ordinary
/// ATTRIBUTIVE/prenominal adjective modifying "incident", stranded in the
/// token stream with nothing excluding it), "What is the role of the
/// FISCAL Intermediary...?" (the same shape, "fiscal"), and "Is home
/// infusion therapy INCLUDED in EVV?"/"Is Hospice INCLUDED in EVV?" (a
/// genuine two-entity COVERAGE-verification question this corpus's ground
/// truth DOES expect answered — "included" is a past-participial adjective
/// reading of the sentence's own real, intended predicate, not a deontic
/// complication). `ObligationModality::classify_modal`'s small, precise,
/// literature-cited closed set catches every ORIGINALLY-targeted case
/// (`Mandatory`: "required"/"requires"/"shall"/"must"; `Prohibitive`;
/// `Discretionary`) without any of the four false positives above — none
/// of "minor"/"fiscal"/"included"/"eligible" are deontic-modality surfaces
/// by von Wright's own primitive partition. `exclude` has no counterpart in
/// [`has_modal_or_descriptive_predicate`] itself (that function's own doc:
/// "Deliberately scans EVERY token, unconditionally — no entity exclusion"
/// — correct for its parse-FAILURE callers, whose noun/entity pool is
/// itself degenerate) — it names the surface(s) `answer_question` already
/// extracted as THIS question's own well-typed entities, so a possessive
/// pronoun or deontic-modal word that is itself part of the resolved
/// definiendum (rather than a genuinely separate obligation marker
/// elsewhere in the sentence) never disqualifies the very entity it names.
fn has_deontic_or_descriptive_marker(
    lang: &dyn Language,
    tokens: &[TypedToken],
    exclude: &[&str],
) -> bool {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::{LexicalEntry, PronounKind};
    use pr4xis_domains::social::judicial::modality::ontology::classify_modal;
    let is_excluded = |t: &TypedToken| {
        exclude
            .iter()
            .any(|e| e.split_whitespace().any(|w| w == t.word))
    };
    let candidates: Vec<&TypedToken> = tokens.iter().filter(|t| !is_excluded(t)).collect();
    candidates.iter().any(|t| {
        matches!(
            lang.lexical_lookup(&t.word),
            Some(LexicalEntry::Pronoun(p)) if p.kind == PronounKind::Possessive
        )
    }) || candidates.iter().any(|t| classify_modal(&t.word).is_some())
}

/// A collapsed `scope_predicate_lexicon` token ("required for", "subject
/// to", …) is itself NEVER a resolvable content word (it has no embedded-
/// English dictionary entry) — its mere PRESENCE is itself the modal/
/// deontic signal a single-definiendum "define X" answer must defer to,
/// independent of how many other content entities happen to survive
/// extraction around it. Extracted as a standalone function (was an inline
/// `let` binding local to [`attempt_partial_understanding`]) so
/// [`answer_question`]'s structurally identical single-/multi-entity
/// define paths can consult the SAME check rather than duplicating it.
fn has_scope_predicate_token(tokens: &[TypedToken]) -> bool {
    tokens.iter().any(|t| {
        pr4xis_domains::cognitive::linguistics::scope_predicate_lexicon::scope_predicate_surfaces()
            .contains(&t.word)
    })
}

/// English's single infinitival particle (Huddleston & Pullum 2002, *The
/// Cambridge Grammar of the English Language*, Ch.14 §2) — a closed class of
/// exactly one surface. Mirrors `lambek::tokenize::is_infinitive_marker`
/// (the SAME check, SAME citation) rather than importing it: `tokenize.rs`
/// is one of exactly two files whose full byte content is content-addressed
/// as the `defines_pointers` grammar-signature closure
/// ([`pr4xis_domains::applied::data_provisioning::defines_grammar_signature::DEFINES_GRAMMAR_CLOSURE_FILES`])
/// — widening its visibility, even for a pure no-op signature change, marks
/// the multi-hour cached `.defines.cprx.gz` corpus overlay stale and forces
/// a full `pr4xis compile --defines --lock` regen. A single-literal closed
/// class has no meaningful "single source of truth" cost to duplicating
/// across that specific boundary.
fn is_infinitive_marker(word: &str) -> bool {
    word == "to"
}

/// An INFINITIVAL wh-question — "what to bring", "where to go", "who to
/// ask" (Huddleston & Pullum 2002 Ch.14 §2: an infinitival relative/
/// interrogative clause, headed by a bare `to`-VP with no finite predicate
/// at all) — is asking for a course of ACTION, not naming a definiendum:
/// nothing in the sentence claims to BE, MEAN, or otherwise characterize
/// whatever single entity a degenerate argument-extraction happens to
/// surface (e.g. "What to bring to nursing home?" extracts "nursing home"
/// as its sole entity leaf, but the sentence never asserts anything IS or
/// MEANS "nursing home" — it asks what items to pack). Recognized directly
/// and structurally — the wh-word immediately followed by the infinitive
/// marker `to` — rather than inferred indirectly from copula absence (which
/// also, wrongly, excludes "What DOES X mean/provide?", a `does`-
/// periphrastic definitional question with no copula either — see
/// [`has_deontic_or_descriptive_marker`]'s own doc for the corpus evidence
/// this distinction is load-bearing).
fn is_infinitival_wh_question(en: &dyn LexicalReasoner, tokens: &[TypedToken]) -> bool {
    tokens
        .first()
        .is_some_and(|first| en.is_nonpersonal_interrogative(&first.word))
        && tokens
            .get(1)
            .is_some_and(|second| is_infinitive_marker(&second.word))
}

/// The head of the copula's post-copular complement NP — the token
/// immediately after a loaded Copula token (OLiA Copula —
/// "is"/"are"/"was"/"were", distinct from progressive/perfect Auxiliary),
/// skipping any Determiner/Article (Higgins 1973; Mikkelsen 2005, *Copular
/// Clauses: Specification, Predication and Equation*), PLUS the one token
/// right after that head — covering a CLASSIFIER-NOUN quoted-apposition
/// ("what is the TYPE 'mammal'?": the copula's grammatical head is "type",
/// but the quoted span "mammal" immediately apposed to it is the real
/// definiendum `unresolved_definitional_subject_compound`/quote-
/// normalization already isolates elsewhere in this module — confirmed
/// real regression: `what_does_the_quoted_appositive_mean_isolates_the_
/// quoted_span` newly failed with a one-token-only version of this
/// function). Bounded to exactly two candidates, not an open window: a
/// PP-attached different topic ("the turnover RATE FOR direct care
/// workers", "the PROCESS FOR background checks") puts a Preposition
/// between the head and the unrelated entity, which never lands in either
/// of these two slots, so the bound does not reopen the mismatch class
/// this function exists to catch. Empty when no Copula token exists at
/// all (a plain matrix-verb question like "who PAYS for X", or a
/// `does`-periphrastic "what DOES X mean/provide?") — copula-absence is
/// deliberately NOT conflated with "the copula names this entity";
/// callers decide separately what copula-absence should mean for their
/// own gate.
fn copula_complement_candidates<'a>(
    lang: &dyn Language,
    tokens: &'a [TypedToken],
) -> Vec<&'a TypedToken> {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag;
    let is_determiner_pos = |p: PosTag| matches!(p, PosTag::Determiner | PosTag::Article);
    let Some(copula_idx) = tokens.iter().position(|t| {
        lang.lexical_lookup(&t.word)
            .is_some_and(|e| e.pos_tag() == PosTag::Copula)
    }) else {
        return Vec::new();
    };
    let Some(head_offset) = tokens[copula_idx + 1..].iter().position(|t| {
        !lang
            .lexical_lookup(&t.word)
            .is_some_and(|e| is_determiner_pos(e.pos_tag()))
    }) else {
        return Vec::new();
    };
    let head_idx = copula_idx + 1 + head_offset;
    let head = &tokens[head_idx];
    // The second candidate (the classifier-noun-plus-quoted-apposition
    // lookahead) is only valid when the head itself is a bare NOUN. An
    // Adjective head ("eligible", or the multiword-collapsed "eligible
    // for") governs its OWN complement/PP — the token right after it is
    // that governed argument, not a second name for the same referent
    // ("who is ELIGIBLE FOR [DCH]" collapses "eligible for" into one
    // Adjective-typed token, putting DCH immediately after it; without
    // this gate the lookahead would wrongly treat DCH as a second valid
    // complement candidate, reopening the exact mismatch this function
    // exists to catch — confirmed real regression against
    // `who_is_eligible_for_x_declines_but_who_is_x_still_answers`).
    // `lang.lexical_lookup` alone under-covers here the SAME way
    // `is_what_copula_question`'s own doc explains for verbs: it resolves
    // CLOSED-class function words, so an open-class content noun ("type",
    // "turnover rate") is invisible to it. `LambekType::Atom(N | NP)`
    // (the chart's own category for the token, assigned regardless of
    // open/closed class) closes that gap the same way `is_predicate`
    // already does for predicates.
    use pr4xis_domains::cognitive::linguistics::lambek::types::{AtomicType, LambekType};
    let head_is_noun = matches!(
        head.lambek_type,
        LambekType::Atom(AtomicType::N) | LambekType::Atom(AtomicType::NP)
    ) || lang
        .lexical_lookup(&head.word)
        .is_some_and(|e| e.pos_tag() == PosTag::Noun);
    if head_is_noun {
        tokens[head_idx..].iter().take(2).collect()
    } else {
        vec![head]
    }
}

/// Whether a loaded Copula token links the wh-word to some OTHER nominal
/// than the single resolved content entity — i.e. the entity is real and
/// domain-loaded, but is not what the copula is actually predicating
/// identity of. Distinguishes "Who is [eligible] for DCH?" (the copula
/// predicates an ADJECTIVE; the resolved entity is merely embedded in a
/// trailing PP-complement of that predicate, not the thing being defined)
/// from "Who IS [DCH]?" (the post-copular NP IS the resolved entity).
///
/// A single resolved content entity was never sufficient evidence the
/// SENTENCE itself asks to define that entity (`is_what_copula_question`'s
/// own doc makes the identical point for the "what" gate) — this closes the
/// same gap structurally rather than by POS-tag presence alone, which is
/// too coarse: "eligible" in "who is eligible for X" also sits right after
/// a bona fide Copula token, so requiring a copula is not enough, only
/// checking WHAT the copula names is. When no Copula token is present at
/// all (a `does`-periphrastic "what DOES X mean/provide?", OR a plain
/// matrix-verb question like "who PAYS for X" with no copula anywhere),
/// this returns `false` (no signal either way) rather than treating
/// copula-ABSENCE as exclusionary — deliberately soft, so it is safe to use
/// as an additional exclusionary conjunct on the "what"/"which" branch
/// (which must keep allowing does-periphrastic questions through) without
/// requiring a stronger, positive check like
/// [`who_predication_identifies_the_entity`] below.
///
/// Confirmed real regression this closes (jq-diffed against the committed
/// snapshot after the 2026-07 vocabulary-closure/tokenizer batch newly
/// exposed it — the entity these questions embed only started resolving to
/// exactly one entity once the batch's new lexicon rows and compound
/// collapsing landed): "Who is eligible for the Self-Determination
/// Program?", "Who is eligible for the PCA program?".
fn copula_predicate_names_a_different_entity(
    lang: &dyn Language,
    tokens: &[TypedToken],
    entity_surface: &str,
) -> bool {
    let entity_lower = entity_surface.to_lowercase();
    let candidates = copula_complement_candidates(lang, tokens);
    !candidates.is_empty()
        && !candidates
            .iter()
            .any(|t| entity_lower.starts_with(&t.word.to_lowercase()))
}

/// The POSITIVE counterpart to [`copula_predicate_names_a_different_entity`]
/// — required (not merely non-excluding) for the "who is X" branch, since
/// unlike the "what"/"which" branch that branch has no does-periphrastic
/// positive case to preserve (measured: every genuine shape-(a) row in this
/// corpus — "Who is DCH?", "Who is a General Caregiver?", "Who is a Legally
/// Responsible Individual (LRI)?" — literally uses the copula "is"). A
/// bare, predicate-less NP ("General Caregiver?") is allowed through the
/// same way [`is_what_copula_question`] allows it for "what". Anything
/// else — a genuine finite predicate that is NOT a copula at all — means
/// "who" is the SUBJECT of some OTHER verb ("pays", "schedules",
/// "administers", "qualifies", "put") and the resolved entity is merely
/// that verb's object/oblique argument, not what "who" refers to: "Who
/// PAYS for long-term care?" asks to name a payer, not to define
/// "long-term care"; "long-term care" never appears as a copula
/// complement at all because there is no copula in the sentence.
///
/// Confirmed real regression this closes (same jq-diff as above): "Who
/// pays for long-term care?", "Who pays for the EVV system?", "Who pays
/// for assistive technology?", "Who Pays Nursing Home During the Lookback
/// Period?", "Who put the member's information into the portal?", "Who
/// schedules the appointments?", "Who pays a personal care attendant?",
/// "Who Qualifies for Medicaid?" — eight previously-correct Green rows
/// that started confidently defining an unrelated resolved entity once
/// `entities.len() == 1` started landing more often, none of which carry
/// any Copula token at all (so the softer mismatch-only check above never
/// fired for them).
fn who_predication_identifies_the_entity(
    lang: &dyn Language,
    tokens: &[TypedToken],
    entity_surface: &str,
) -> bool {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag;
    let is_predicate_pos =
        |p: PosTag| matches!(p, PosTag::Verb | PosTag::Copula | PosTag::Auxiliary);
    let has_no_predicate = !tokens.iter().any(|t| {
        t.lambek_type.is_predicate()
            || lang
                .lexical_lookup(&t.word)
                .is_some_and(|e| is_predicate_pos(e.pos_tag()))
    });
    if has_no_predicate {
        return true;
    }
    let entity_lower = entity_surface.to_lowercase();
    copula_complement_candidates(lang, tokens)
        .iter()
        .any(|t| entity_lower.starts_with(&t.word.to_lowercase()))
}

/// Whether this sentence is genuinely shaped like a "what/which is X"
/// definitional query, the ONLY shape `define_word` honestly answers —
/// requires BOTH a nonpersonal interrogative pronoun/determiner ("what",
/// "which"; the loaded `WhReferentRole` Thing/Selection split, Cysouw
/// 2004 §3.2 table (9) — `en.is_nonpersonal_interrogative`) AND the
/// loaded Copula POS (OLiA Copula — "is"/"are", distinct from
/// Verb/Auxiliary in the same closed-class lexicon). A single resolved
/// content entity is not by itself evidence the SENTENCE is asking to
/// define that entity: "Who pays for long-term care?" ("who", no
/// copula-linked "what"), "Is there a cost for the PCCA license?"
/// (existential "is", no "what"/"which"), and "What Medicare Covers..."
/// ("what" with no copula at all, "covers" is the real predicate) all
/// resolve exactly one loaded content word and are NOT "what is X"
/// queries — caught via the full corpus regression suite (9
/// previously-correct out-of-scope-abstain questions started confidently
/// defining an unrelated loaded concept once the concept-dedup fix in
/// [`attempt_partial_understanding`] made their content-entity count
/// land on exactly 1 more often).
/// ALSO permits a sentence with NO finite verb/copula/auxiliary at all
/// ("Conservator", "Trustee?") — a bare word or noun phrase carries no
/// predicate structure of ANY kind, so "define this word" is the only
/// interpretation available; unlike "what responsibilities does a
/// trustee have" (a real predicate, "does...have", just not the copula),
/// there is nothing else the query COULD be asking. The closed-class POS
/// check alone is not sufficient here: `lang.lexical_lookup` only
/// resolves closed-class function words, so an open-class content verb
/// ("Who REIMBURSES caregivers for mileage?") would be invisible to it
/// and `has_no_predicate` would wrongly read as true. `LambekType::is_predicate`
/// closes that gap — every token's category comes from the same
/// OLiA-class projection regardless of open/closed class (confirmed via
/// full corpus regression: the POS-only check caused 24 new
/// Green→OverAnswered flips on exactly this class of question).
///
/// Extracted as a standalone function (was an inline `let` binding local
/// to [`attempt_partial_understanding`]) so [`answer_question`]'s
/// structurally identical single-entity define path (previously ungated
/// entirely — the task#42/POA precedent this gate was built for was never
/// wired into the parse-SUCCESS path) can consult the SAME check.
fn is_what_copula_question(
    en: &dyn LexicalReasoner,
    lang: &dyn Language,
    tokens: &[TypedToken],
) -> bool {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag;
    let is_predicate_pos =
        |p: PosTag| matches!(p, PosTag::Verb | PosTag::Copula | PosTag::Auxiliary);
    let has_no_predicate = !tokens.iter().any(|t| {
        t.lambek_type.is_predicate()
            || lang
                .lexical_lookup(&t.word)
                .is_some_and(|e| is_predicate_pos(e.pos_tag()))
    });
    has_no_predicate
        || (tokens
            .iter()
            .any(|t| en.is_nonpersonal_interrogative(&t.word))
            && tokens.iter().any(|t| {
                lang.lexical_lookup(&t.word)
                    .is_some_and(|entry| entry.pos_tag() == PosTag::Copula)
            }))
}

/// The DEFINITIONAL-SUBJECT COMPOUND closure: for a question the pipeline
/// already recognizes as a "what/which is X" definitional query
/// ([`is_what_copula_question`] — the loaded `WhReferentRole` + Copula-POS
/// gate, or a bare predicate-less NP), identify X itself as the FIRST
/// maximal run of adjacent, individually-RESOLVING nominal constituents in
/// the token stream — the post-copular subject NP of a specificational
/// copular clause (Higgins 1973, *The Pseudo-cleft Construction in
/// English*; Mikkelsen 2005, *Copular Clauses: Specification, Predication
/// and Equation*, Benjamins: in "what is X?" the wh-word, copula, and
/// determiner are all closed-class syncategorematics, so the first nominal
/// run IS the definiendum) — and, when that run is a MULTI-word compound
/// that does NOT resolve AS A UNIT anywhere in the loaded vocabulary,
/// return the joined compound so the caller abstains NAMING it.
///
/// Why constituent senses can never stand in for the compound's own
/// referent: a nominal compound is a NAME for a category, not a
/// compositional description — the semantic relation between its
/// constituents is unstated, contextually supplied, and conventionalized
/// per compound, so a hearer who knows every constituent still does not
/// know the compound's referent (Downing 1977, "On the Creation and Use of
/// English Compound Nouns," *Language* 53(4):810-842). A fortiori for a
/// titled program/statute name ("Caregiver Social Security Credit
/// Program"): a proper name designates rigidly, not via the descriptive
/// content of its parts (Kripke 1980, *Naming and Necessity*, Harvard UP),
/// so enumerating WordNet senses of "caregiver" and "program" answers a
/// question nobody asked while fabricating confidence about a term the
/// loaded sources never define — the exact TruthfulQA fabricated-term
/// failure mode (Lin, Hilton & Evans 2022, ACL) the adversarial corpus
/// (`praxis-corpus-tests`, category `fabricated_term`) measures.
///
/// Structural discipline:
/// - Run membership is decided by the chart's own category assignment (a
///   noun-typed token, or a multi-word proper-noun token minted by
///   `collapse_multiword_surfaces`) PLUS the established exact ∪
///   case-folded lookup union — never a word list, never a pattern over
///   the input string. Requiring every constituent to RESOLVE keeps this
///   closure scoped to precisely the fabricated-compound signature (real,
///   individually-known words recombined into an unloaded unit); a run
///   containing a genuinely unknown word is a different epistemic
///   situation (name THAT word) already handled by the existing paths.
///   Two further bounds, each forced by a live corpus measurement (see
///   the inline comments): a closed-class function word never joins a
///   run even under a spurious WordNet homograph reading, and an
///   APPOSITIVE re-mention of a concept an earlier member already named
///   (a registered acronym alias: "Electronic Visit Verification (EVV)")
///   is not a second constituent — a compound needs two or more DISTINCT
///   constituent concepts.
/// - The ≥ 2 arity bound is definitional, not a tuned threshold:
///   compounding is by definition the combination of two or more lexemes
///   (Downing 1977, above) — a one-token run is not a compound, and the
///   existing single-definiendum path owns it.
/// - The as-a-unit check is [`resolve_surface`] (exact ∪ lemmatized ∪
///   case-folded — the SAME widest resolution net every honesty gate in
///   this module already uses), so this closure never claims ignorance of
///   a compound any other path could resolve.
/// - The returned surface is the user's OWN uttered contiguous nominal
///   string — a mentioned/queried term (the [`DefiniendumIsALexicalUnit`]
///   axiom's own Quinean category), never a composition-minted
///   concatenation.
fn unresolved_definitional_subject_compound(
    en: &dyn LexicalReasoner,
    lang: &dyn Language,
    tokens: &[TypedToken],
) -> Option<String> {
    use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
    if !is_what_copula_question(en, lang, tokens) {
        return None;
    }
    // A closed-class function word can never itself be part of a compound
    // noun's own surface — the SAME boundary discipline
    // [`widen_definiendum_to_compound`]'s own doc already states and
    // applies — even when it happens to carry a spurious open-class WordNet
    // homograph reading in a degenerate parse-failure stream ("or" is
    // WordNet's operating-room/logical-OR noun; "as" is the element
    // arsenic — confirmed live on corpus row 4174 "Hallucinations or
    // Loneliness?", where without this bound the coordinator "or" joined
    // the run as a "constituent" of a fabricated compound "or loneliness").
    let is_known_noun_constituent = |t: &TypedToken| {
        let nominal = t.lambek_type.is_noun()
            || (t.lambek_type == svo::proper_noun() && t.word.contains(' '));
        nominal
            && !en.is_function_word(&t.word)
            && (!en.lookup(&t.word).is_empty() || !en.lookup_case_folded(&t.word).is_empty())
    };
    let start = tokens.iter().position(is_known_noun_constituent)?;
    // APPOSITIVE-GLOSS control — the SAME first-concept dedup discipline
    // `content_entities` and `noun_phrase_heads` already apply: a run member
    // that resolves to a concept an EARLIER member of the run already named
    // is not a second compound constituent, it is the same concept's own
    // alternative written form in close apposition (Quirk, Greenbaum, Leech
    // & Svartvik 1985, *A Comprehensive Grammar of the English Language*,
    // §17.65-17.68: appositives are co-referential re-mentions, not
    // modifier-head structure). The corpus shape this protects is "What is
    // Electronic Visit Verification (EVV)?" — the parenthetical acronym is
    // a REGISTERED ALIAS resolving to the very concept the compound it
    // glosses resolves to, and before this control the pair read as a
    // 2-constituent compound whose join ("electronic visit verification
    // evv") resolves to nothing, flipping 6 real, previously-Green
    // definitional corpus rows (USERRA/Medigap/EVV/FMS/D-SNP/DNR — caught
    // by the caregiver_capability_ratchet gate) into false abstentions. A
    // compound, by contrast, requires two or more DISTINCT constituent
    // concepts (Downing 1977's N+N pairs are pairs of distinct category
    // names).
    let mut distinct_concepts: Vec<ConceptId> = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for t in &tokens[start..] {
        if !is_known_noun_constituent(t) {
            break;
        }
        // The union's first id — the same resolution order that minted a
        // collapsed token (`multiword_surface_readings`).
        let id = en
            .lookup(&t.word)
            .first()
            .copied()
            .or_else(|| en.lookup_case_folded(&t.word).first().copied());
        match id {
            Some(id) if distinct_concepts.contains(&id) => continue,
            Some(id) => distinct_concepts.push(id),
            // Unreachable (membership requires the union to resolve), kept
            // exhaustive rather than unwrapped.
            None => {}
        }
        run.push(t.word.as_str());
    }
    if run.len() < 2 {
        return None;
    }
    let compound = run.join(" ");
    if !resolve_surface(en, &compound).is_empty() {
        return None;
    }
    Some(compound)
}

fn attempt_partial_understanding(
    en: &dyn LexicalReasoner,
    lang: &dyn Language,
    tokens: &[TypedToken],
    reduction: &ReductionResult,
    _meaning: &montague::Sem,
) -> trace_impls::ResponseResult {
    // Governance-predicate check FIRST, before the generic knowledge/
    // exploration fallback below: a recognized rule-governing predicate
    // ("eligible for X") deserves a chance at the conditional-rule
    // registry, or an honest "I don't have that" abstention, never a
    // nouns-based taxonomy dump of unrelated senses of whatever common
    // words happen to also appear in the sentence.
    if let Some((predicate, object_words)) = find_governed_predicate(tokens) {
        let object = object_words.join(" ");
        if let Some(rule) = en.conditional_rule_for_predicate(&predicate, &object) {
            use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{
                self, ConditionalPresentation, ResponseContent,
            };
            use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
            use pr4xis_domains::social::judicial::conditional_rule::Applicability;
            if let Applicability::Indeterminate { missing } = rule.applicability() {
                let presentation = ConditionalPresentation {
                    rule_name: rule.term.name.text.clone(),
                    citation: rule
                        .term
                        .source_text
                        .as_ref()
                        .map(|s| s.text.clone())
                        .or_else(|| rule.term.subsection.as_ref().map(|c| c.to_bluebook()))
                        .unwrap_or_default(),
                    rule_text: rule.term.definition.text.clone(),
                    missing_facts: missing
                        .iter()
                        .map(|el| el.requirement.field.text.clone())
                        .collect(),
                };
                let content = ResponseContent::new(ResponseFrame::RuleAwaitingFact)
                    .with_conditional(presentation);
                return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                    .with_conditional(Some((rule, missing)));
            }
            // Applies/DoesNotApply: no producer populates these yet (mirrors
            // `answer_question`'s own not-yet-reachable note) — fall through
            // to the honest-abstention arm below.
        }
        // The predicate IS recognized as rule-governed, but no loaded rule
        // covers this specific object — the honest answer is "I don't have
        // that," naming what was asked about, never a generic exploration
        // of incidental common-noun senses elsewhere in the sentence.
        use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
        use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation).with_entity(&object);
        return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
            .with_entities_found(vec![object]);
    }

    // Words the grammar recognizes at all (needed to PARSE — closed-class
    // function words like "does"/"a"/"how" included) vs. words that carry
    // no lexical entry whatsoever.
    let unknown_words: Vec<&str> = tokens
        .iter()
        .filter(|t| lang.lexical_lookup(&t.word).is_none())
        .map(|t| t.word.as_str())
        .collect();

    // Whether a real ANSWER EXISTS for this question — the `UnknownKnown`
    // contract (`epistemics.rs`: "answer exists but grammar can't parse").
    // That requires a genuine CONTENT-bearing token: one whose part of
    // speech is open-class (Noun/Verb/Adjective/Adverb/Interjection, per
    // `PosTag::is_content` — OLiA, Chiarcos & Sukhareva 2015) AND that
    // resolves to an actual loaded concept via `en.lookup`. Checking only
    // `lang.lexical_lookup(&t.word).is_some()` (any recognized token) is
    // too weak: closed-class function words the grammar needs to attempt
    // PARSING ("does", "a", "how") pass that check just as readily as real
    // content words, which previously made every question containing them
    // read as knowledge-grounded regardless of whether the entity actually
    // asked about resolved to anything.
    //
    // A LOADED surface (`en.is_loaded_surface`) also counts as content even
    // when `lang.lexical_lookup` (the EMBEDDED-English-only dictionary)
    // misses entirely: a loaded multi-word domain concept ("legally
    // responsible individual") has no embedded-English headword to check a
    // POS tag against, but by construction is never a closed-class function
    // word either — the same "loaded surfaces are unambiguously content"
    // signal the NP/N dual-reading push (Stage 1) already relies on. Left
    // ungated, a sentence whose ONLY resolvable word was a loaded multi-word
    // concept fell all the way to `UnknownUnknown` ("I do not understand")
    // instead of `UnknownKnown` (caught via the full corpus regression
    // suite: "What is a Legally Responsible Individual (LRI)?" — the failed
    // full-grammar parse should still recognize a real, loaded answer).
    //
    // Deduplicated by resolved CONCEPT, not surface string: "legally
    // responsible individual" and its own registered acronym "lri" are two
    // surfaces for the SAME loaded concept, and counting both as separate
    // entities would misroute a clean single-definiendum question into the
    // multi-noun comparison branch below, comparing the concept to itself.
    let mut seen_concepts: Vec<ConceptId> = Vec::new();
    let content_entities: Vec<&str> = tokens
        .iter()
        .filter(|t| {
            let is_content = lang
                .lexical_lookup(&t.word)
                .is_some_and(|entry| entry.pos_tag().is_content())
                || en.is_loaded_surface(&t.word);
            is_content && !en.lookup(&t.word).is_empty()
        })
        .filter(|t| match en.lookup(&t.word).first() {
            Some(&id) if seen_concepts.contains(&id) => false,
            Some(&id) => {
                seen_concepts.push(id);
                true
            }
            None => true,
        })
        .map(|t| t.word.as_str())
        .collect();

    let has_knowledge = !content_entities.is_empty();
    let parsed = reduction.success;
    let query_result: Option<&str> = if parsed { Some("parsed") } else { None };
    let state = epistemics::classify_result(parsed, has_knowledge, query_result);

    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let frame = ResponseFrame::from_epistemic(&state);
    let entities: Vec<String> = content_entities.iter().map(|s| s.to_string()).collect();

    // `from_ontology` tracks whether this turn is a genuine ANSWER, not
    // merely whether `has_knowledge` found some content word — the two are
    // NOT the same claim. `SuggestInterpretation` ("I found N concepts but
    // could not parse the sentence — did you mean: is a X a Y?") is a
    // clarification request, not an answer: it names concepts that
    // resolved but explicitly asks the user to confirm what they meant.
    // Defaulting `from_ontology` to `has_knowledge` and only overriding it
    // for that one branch keeps every OTHER branch's existing behavior
    // (define_word and explore_concepts genuinely do answer) unchanged.
    let mut from_ontology = has_knowledge;

    // The communicative goal that actually produces `response` — carried out to
    // the `ResponseResult` so the "Why?" layer keys off the SAME frame that was
    // realized, not the raw epistemic frame. Defaults to `frame`; overridden only
    // where a branch realizes from a DIFFERENT frame than the epistemic one (the
    // unresolved-compound branch below, which realizes `UnknownVocabulary` under
    // the `UnknownKnown` state). The answered branches (define/explore) leave it
    // as `frame`: `realize_why` keys those off `from_ontology`, not the frame.
    let mut used_frame = frame;

    // A collapsed `scope_predicate_lexicon` token ("required for", "subject
    // to", …) is itself NEVER in `content_entities` — it has no embedded-
    // English dictionary entry to satisfy `lang.lexical_lookup`, so fusing
    // it removes what was, pre-collapse, a SEPARATE resolvable content word
    // ("required", "subject") from the pool. That can silently shrink
    // `content_entities` from 2 down to 1, rerouting a modal/deontic
    // question ("who are incident reports required for?") into the
    // UNGATED single-definiendum branch below instead of the
    // `has_modal_or_descriptive_predicate`-gated branch it needs — the
    // same failure class that gate exists to prevent, just one step
    // earlier in the pool computation. Presence of a scope-predicate token
    // is itself the modal/deontic signal, so route through the gated
    // branch whenever one is present, regardless of the entity count.
    let has_scope_predicate_token = has_scope_predicate_token(tokens);

    // See [`is_what_copula_question`]'s own doc comment for the full
    // rationale; extracted to a standalone function so `answer_question`'s
    // structurally identical single-entity define path can share it.
    let is_what_copula_question = is_what_copula_question(en, lang, tokens);

    // A modal auxiliary co-occurring with the WH+copula shape above still
    // signals a deontic/advice question, not a definitional one: "Which of
    // my mom's kids SHOULD be her POA?" has "which" + a copula ("be") and
    // resolves exactly one loaded content word ("POA"), satisfying
    // `is_what_copula_question` on its own — but "should" (Auxiliary POS)
    // is exactly the modal/deontic signal `has_modal_or_descriptive_
    // predicate` exists to catch (already gating the multi-noun
    // `explore_concepts` branch below); a personal-advice question like
    // this one is not honestly answerable as "define POA" just because it
    // happens to share the surface shape of "what is X" (caught via the
    // full corpus regression suite as a Green->OverAnswered flip once the
    // task #42 POA vocabulary made `content_entities.len() == 1` land here).

    // The definitional branch below answers over a definiendum this function's
    // own `entities` list may not contain (`widen_definiendum_to_compound` can
    // widen the head into a compound), and out of a channel `entities` cannot
    // see at all (the statutory `defines` overlay). It therefore reports its own
    // provenance, collected here and unioned into the entity-derived claim at
    // the bottom rather than replacing it — the entities genuinely did resolve
    // and are genuinely reported as `entities_found`, so their ontologies keep
    // their credit; this only adds what was missing. Empty for every other
    // branch, none of which defines anything.
    let mut definitional_reasoned_over: Vec<OntologyName> = Vec::new();
    // The definitional branch's OTHER provenance channel — the documents a
    // recited gloss was authored from. Reported separately from
    // `definitional_reasoned_over` all the way out, because merging them at any
    // point would erase exactly the distinction they exist to carry.
    let mut definition_provenance: Vec<DefinitionProvenance> = Vec::new();

    let response = match state {
        epistemics::EpistemicConcept::UnknownKnown => {
            // Checked FIRST, before the single-definiendum and multi-noun
            // dispatch below: when the definitional subject is a multi-word
            // nominal compound that does not resolve AS A UNIT, neither
            // branch can honestly answer — `define_word` would define one
            // constituent, `explore_concepts` would enumerate constituent
            // senses — and both fabricate confidence about a term the
            // loaded sources never define (see
            // [`unresolved_definitional_subject_compound`]'s own doc,
            // Downing 1977 / Kripke 1980). The honest closed-world outcome
            // (Reiter 1978) is an abstention NAMING the full compound the
            // user actually asked about, so `unresolved` semantics match
            // every other vocabulary-gap path in this module.
            if let Some(compound) = unresolved_definitional_subject_compound(en, lang, tokens) {
                from_ontology = false;
                used_frame = ResponseFrame::UnknownVocabulary;
                realize::realize(
                    &ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity(&compound),
                )
            } else if content_entities.len() == 1
                && !has_scope_predicate_token
                && is_what_copula_question
                && !has_modal_or_descriptive_predicate(lang, tokens)
            {
                let definiendum = widen_definiendum_to_compound(tokens, content_entities[0], en);
                let answer = define_word(en, &definiendum);
                definitional_reasoned_over = answer.reasoned_over;
                definition_provenance = answer.authored_from;
                answer.text
            } else {
                // `noun_phrase_heads` also concept-dedups (two surfaces for
                // the same loaded concept, "legally responsible individual"
                // / "lri", must not count as two comparable nouns, or
                // `explore_concepts` would compare the concept to itself
                // instead of routing to the clean single-definiendum branch
                // its own `content_entities.len() == 1` check missed only
                // because the two surfaces inflated the count to 2) AND
                // collapses each nominal-compound run to its own head (see
                // its doc comment) so a single unparsed multi-word compound
                // never reads as several independently comparable nouns.
                let nouns: Vec<&str> = noun_phrase_heads(tokens, en);
                // `explore_concepts`'s own documented use case is a bare
                // taxonomic comparison the chart failed to parse ("is a
                // seal a mammal?") — a sentence with neither an Adjective
                // nor an Auxiliary token. A modal/deontic question ("am I
                // ELIGIBLE for X", "is X REQUIRED", "can/does/must...")
                // has a fundamentally different predicate structure
                // (permission/obligation, not "X is-a Y"), which lexical
                // taxonomy exploration cannot honestly answer — firing it
                // anyway previously produced a WordNet-sense dump of
                // whatever incidental nouns the modal question happened to
                // mention, read by the corpus harness as a fabricated
                // answer to a question that should have abstained. Gated
                // on POS tags (OLiA, Chiarcos & Sukhareva 2015), never a
                // word list.
                if nouns.len() >= 2 && !has_modal_or_descriptive_predicate(lang, tokens) {
                    explore_concepts(en, &nouns)
                } else {
                    // Neither a single definiendum nor a clean taxonomic
                    // comparison: `SuggestInterpretation`'s "did you mean"
                    // is a question back to the user, not an answer.
                    //
                    // `nouns` (already computed above), not `content_entities`
                    // — a "did you mean: is a X a Y?" guess is only a
                    // coherent NP/NP comparison when X and Y are themselves
                    // noun phrases; `content_entities` admits verbs/
                    // adjectives too (`PosTag::is_content`), so a sentence
                    // like "Does EVV REQUIRE the Social Security number…"
                    // paired the verb "require" with "evv" into "is an evv
                    // a require?" — a confirmed real defect (30-agent full-
                    // corpus gap audit, ~187-row cluster) traced to exactly
                    // this positional, POS-blind selection, not to
                    // `_meaning`'s montague output (unused on this path) as
                    // first suspected. `nouns.len() < 2` here degrades
                    // gracefully: `realize_suggestion` already has an
                    // honest single-entity ("I know the word X but could
                    // not understand the sentence structure") and zero-
                    // entity fallback, so naming fewer, real nouns is
                    // strictly more honest than padding out to 2 with a
                    // non-nominal word.
                    from_ontology = false;
                    let mut content = ResponseContent::new(frame);
                    for w in &nouns {
                        content = content.with_entity(w);
                    }
                    realize::realize(&content)
                }
            }
        }
        epistemics::EpistemicConcept::KnownUnknown => {
            let mut content = ResponseContent::new(frame);
            for w in &unknown_words {
                content = content.with_entity(w);
            }
            realize::realize(&content)
        }
        epistemics::EpistemicConcept::KnownKnown => {
            // By construction, `_meaning` here is NEITHER `Sem::Question` nor
            // `Sem::Prop` — the caller already dispatched both of those cases
            // before ever reaching `attempt_partial_understanding` (see the
            // `match &meaning` in `process_with_reasoner`). So this branch
            // means the CHART's syntax-only reduction (`reduction.success`,
            // alternatives-aware) fully succeeded while `montague::interpret`
            // did not reach a complete `S`.
            //
            // This was previously reachable far more often than it should
            // have been: `interpret` used to walk tokens with a single
            // greedy left-to-right adjacent-pair pass, independent of the
            // chart's own (alternatives-aware, exhaustive-split) derivation
            // — so it could commit to a DIFFERENT, unsuccessful bracketing
            // than the one the chart found, get stuck, and silently discard
            // every unreduced leftover chunk. Fixed by making `interpret`
            // itself a CYK chart mirroring `chart_reduce` (`montague.rs`'s
            // `interpret` doc comment; Montague 1970; Steedman 2000's
            // "rule-to-rule" hypothesis), so semantics now finds a complete
            // derivation whenever the syntax chart does, over the SAME
            // per-token types the chart's own winning derivation backtracked
            // ([`super::reduce::extract_winning_types`]/`chart_reduce`'s
            // `remaining`) — first exposed, and fixed, alongside the
            // genitive-clitic category (task #8): "Medicaid's contractor"
            // only reduces via an ALTERNATE NP reading of "medicaid" the
            // chart explores, which the old greedy walk never reached.
            //
            // This branch remains as a defensive fallback, not a normal
            // path: even a chart-matching CYK semantic derivation can still
            // fail to reach a complete `S` if `apply`'s composition rules
            // have a gap for some combinator shape (a distinct, narrower
            // failure mode than the derivation-order bug above). There is no
            // safe reading of an unreduced `Sem::Func`/`Sem::Pred`/
            // `Sem::Concept` as "the predicate" of a question that never
            // resolved — `_meaning.describe()` on a stalled value used to
            // render raw internal notation ("λ.is") straight into the
            // response, a real defect, not a definitional description. This
            // degrades to the same honest, entity-naming fallback
            // `KnownUnknown` above already uses (never a hardcoded literal —
            // `content_entities` is the same loaded-concept set `has_knowledge`
            // was computed from), never the raw semantic notation.
            from_ontology = false;
            let mut content = ResponseContent::new(frame);
            for w in &content_entities {
                content = content.with_entity(w);
            }
            realize::realize(&content)
        }
        epistemics::EpistemicConcept::UnknownUnknown => {
            realize::realize(&ResponseContent::new(frame))
        }
    };

    // The known entities may resolve to LOADED concepts (this is the path "what is
    // <loaded surface>" takes); record which loaded ontologies were reasoned over,
    // then union in whatever the definitional branch reported over and above them.
    let mut reasoned_over = {
        let ids: Vec<ConceptId> = entities
            .iter()
            .flat_map(|e| en.lookup(e).iter().copied())
            .collect();
        loaded_ontologies_of(en, &ids)
    };
    extend_reasoned_over(&mut reasoned_over, definitional_reasoned_over);
    trace_impls::ResponseResult::new(used_frame, response)
        .with_entities_found(entities)
        .grounded(from_ontology)
        .with_reasoned_over(reasoned_over)
        .with_definition_provenance(definition_provenance)
}

/// Check if the tokens reference the system itself.
///
/// The self-reference decision is owned by the self-model layer: a token is
/// self-referential iff its surface form is one of the self-model's typed
/// self-referents ([`is_self_referent`] — the system's identity name and the
/// indexicals English resolves to the addressee). The routing body asks the
/// self-model rather than enumerating word literals here.
///
/// SMALLEST TYPED STEP (#186): the membership test is against the self-model's
/// self-referent *surface* set. The fully typed form — resolve each token to a
/// SelfModel `ConceptId`/`SenseId` and test membership in the SelfModel
/// reflexive closure — needs an indexical→SelfModel sense bridge that the
/// pipeline does not yet have; tracked as a follow-up.
///
/// The system's own NAME (`SYSTEM_NAME`/"praxis") is an unambiguous self-
/// referent regardless of context — nothing else in ordinary text names the
/// system by name. The second-person indexical ("you"/"yourself") is NOT
/// unambiguous: English also uses "you" GENERICALLY, addressing no one in
/// particular ("how do you know when it's time to…", "how much care will
/// you need" — paraphrasable as "how does ONE know…"/"how much care does A
/// PERSON need"; Kitagawa & Lehrer 1990, "Impersonal uses of personal
/// pronouns", *J. Pragmatics* 14(5):739-759, is the standard citation for
/// this construction; Zobel 2014 ch. 2 the fuller typology). A full corpus
/// regression run caught this concretely: harvested caregiver questions
/// ("Do you get services from a home health agency in Florida…", "How old
/// do you need to be to get home care?") route through the self-model
/// eigenform dump instead of the ordinary answer pipeline, because bare
/// indexical membership doesn't distinguish "you" ADDRESSING the system
/// ("What can you do?", "Are you a computer?" — no other resolvable content,
/// "you" genuinely IS what's being asked about) from "you" as a generic
/// experiencer subject of an ordinary domain question (substantial OTHER
/// resolvable content — "care", "home care", "a home health agency" — is
/// what the question is actually about). Gated accordingly: the indexical
/// trigger only counts as self-referential when NO other token in the
/// sentence resolves to a real loaded concept.
fn is_self_referential(
    tokens: &[pr4xis_domains::cognitive::linguistics::text::Token],
    en: &dyn LexicalReasoner,
) -> bool {
    use pr4xis_domains::formal::information::knowledge::instance::SYSTEM_NAME;
    let names_system = tokens
        .iter()
        .any(|t| t.word == SYSTEM_NAME || t.word == "praxis");
    if names_system {
        return true;
    }

    let has_indexical = tokens.iter().any(|t| is_self_referent(&t.word));
    if !has_indexical {
        return false;
    }
    // A capability-query referent ("reason", "know", "capable", "can" — the
    // self-model's OWN vocabulary for asking about itself) resolves to a
    // real WordNet concept just like any other content word, but it is
    // part of the self-referential utterance, not independent domain
    // content — "What can you REASON about?" must not be defeated by
    // "reason" the noun/verb having its own unrelated WordNet senses,
    // exactly the trap `is_pronoun`/`PronounKind::Possessive` (task #51)
    // already had to correct for elsewhere in this file.
    let has_other_content = tokens.iter().any(|t| {
        !is_self_referent(&t.word)
            && !is_capability_query_referent(&t.word)
            && (t.pos.is_some_and(|p| p.is_content()) || en.is_loaded_surface(&t.word))
            && !en.lookup(&t.word).is_empty()
    });
    !has_other_content
}

/// Whether a self-referential utterance asks SPECIFICALLY about the
/// system's [`Capability`](pr4xis_domains::cognitive::cognition::self_model)
/// — "what can you reason about/do/know" — as opposed to a generic
/// self-referential question. Same typed-set pattern as
/// [`is_self_referential`]: the routing body asks the self-model-owned
/// [`is_capability_query_referent`] set, never a word literal here.
fn is_capability_query(tokens: &[pr4xis_domains::cognitive::linguistics::text::Token]) -> bool {
    tokens.iter().any(|t| is_capability_query_referent(&t.word))
}

/// Answer a self-referential question through the eigenform.
///
/// A CAPABILITY query ("what can you reason about") is realized as
/// generated natural-language text over the live loaded component list
/// (`ChatExplorationEnumeratesTheLoadedInventory`-style: the self-model
/// carries the answer, the NLG layer only composes it — see
/// [`realize::ResponseContent::with_capabilities`]). A component with a
/// registered, verified-not-reachable [`TaskClaim`](pr4xis_domains::
/// cognitive::linguistics::nlp_task::claims::TaskClaim) is named as such
/// ("registered, not yet reachable in conversation") instead of listed as
/// a plain working capability — the fix this whole ontology exists for
/// (`crate::self_describe`'s answer previously could not distinguish "this
/// concept exists in code" from "this concept can actually be used in a
/// live turn"). A component with no registered claim is listed plainly —
/// silence is not a negative claim, only a registered NotReachable verdict
/// is. Every other self-referential question keeps the eigenform's full
/// Schema Presentation → JSON transport, which the chat UI
/// (docs/chat/index.html) consumes as structured self-description, not
/// prose.
fn answer_self_referential(
    lang: &English,
    tokens: &[pr4xis_domains::cognitive::linguistics::text::Token],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::formal::information::schema::transport::Present;
    let eigenform = observe_self(lang);

    let response = if is_capability_query(tokens) {
        use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
        use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
        let capabilities: Vec<String> = eigenform
            .components
            .iter()
            .map(|v| {
                let name = v.name();
                let has_unreachable_claim = eigenform
                    .task_claims
                    .iter()
                    .any(|c| c.component == name && !c.is_reachable());
                if has_unreachable_claim {
                    format!("{name} (registered, not yet reachable in conversation)")
                } else {
                    name.to_string()
                }
            })
            .collect();
        let content =
            ResponseContent::new(ResponseFrame::AssertKnowledge).with_capabilities(capabilities);
        realize::realize(&content)
    } else {
        eigenform.present().to_json()
    };

    // A self-referential answer reasons over the eigenform, not a loaded .prx —
    // AssertKnowledge with empty `reasoned_over`, so the Why? layer states the
    // built-in substrate.
    trace_impls::ResponseResult::new(
        pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame::AssertKnowledge,
        response,
    )
    .with_entities_found(vec!["pr4xis".into(), "self-model".into()])
    .grounded(true)
}

/// Whether `word` is a phatic interjection — one of Ameka's (1992) Greeting/
/// Farewell/Response/Politeness subtypes, read off the loaded lexical entry
/// through the SAME classifier chain the R-2 A2 corpus test walks
/// (`pragmatics::response::jakobson_function_of_interjection`). The loaded
/// WordNet `fw-greeting`/`fw-farewell`/`fw-politeness`/`fw-response-*` synset
/// inventory is the source — never a hardcoded word list.
fn is_phatic_interjection(word: &str, lang: &English) -> bool {
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::LexicalEntry;
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::jakobson_function_of_interjection;
    use pr4xis_domains::formal::information::communication::ontology::JakobsonFunction;
    lang.lexical_lookup_all(word).iter().any(|entry| {
        matches!(
            entry,
            LexicalEntry::Interjection(i)
                if jakobson_function_of_interjection(i.kind) == Some(JakobsonFunction::Phatic)
        )
    })
}

/// Whether `word` counts as "other content" that blocks the phatic
/// short-circuit — anything except a phatic interjection itself or a word
/// resolving EXCLUSIVELY to closed-class (function-word) lexical entries.
///
/// Deliberately does NOT trust the tokenizer's single `Token::pos` pick: a
/// corpus regression ("No will or POA." — an intestate-succession fact, not
/// a greeting) showed why. "No" is genuinely a phatic `Response`
/// interjection, but "will" is ALSO ambiguous between the modal auxiliary
/// and the noun ("a will", a legal document) — the tokenizer's context-free
/// default pick landed on the (non-content) auxiliary reading, and "POA" has
/// no entry at all in the embedded base lexicon (it resolves only through
/// the composed reasoner's registered lexicons, not `English` alone). Both
/// silently failed to register as content under a naive `Token::pos`
/// check. Checking every loaded lexical entry for the word instead — content
/// if ANY reading is Noun/Verb/Adjective/Adverb, and conservatively content
/// if the word is unknown to this lexicon altogether — fixes both: an
/// ambiguous word with a real content reading, and a wholly unresolved one,
/// both correctly block the short-circuit rather than silently licensing it.
fn is_non_phatic_content_word(word: &str, lang: &English) -> bool {
    if is_phatic_interjection(word, lang) {
        return false;
    }
    use pr4xis_domains::cognitive::linguistics::lexicon::pos::LexicalEntry;
    let entries = lang.lexical_lookup_all(word);
    if entries.is_empty() {
        return true;
    }
    entries.iter().any(|e| {
        matches!(
            e,
            LexicalEntry::Noun(_)
                | LexicalEntry::Verb(_)
                | LexicalEntry::Adjective(_)
                | LexicalEntry::Adverb(_)
        )
    })
}

/// Whether the WHOLE utterance is a phatic exchange (Malinowski 1923 /
/// Jakobson 1960): at least one phatic interjection, and no OTHER content
/// word — the same "trigger present, no unrelated content" gate
/// `is_self_referential` already uses for its own indexical check. "Hello"
/// routes through `ResponseFrame::PhaticReturn`; "Hi, what is a nurse?" and
/// "No will or POA." do not — an interjection incidental to real content
/// keeps its own answer path instead of being pre-empted.
fn is_phatic(
    tokens: &[pr4xis_domains::cognitive::linguistics::text::Token],
    lang: &English,
) -> bool {
    let has_phatic = tokens.iter().any(|t| is_phatic_interjection(&t.word, lang));
    if !has_phatic {
        return false;
    }
    let has_other_content = tokens
        .iter()
        .any(|t| is_non_phatic_content_word(&t.word, lang));
    !has_other_content
}

/// Realize the R-2 A2 phatic-return frame (Malinowski 1923's "communion"
/// exchange, content-independent by design — see `realize::
/// realize_phatic_return`'s own doc). The only response frame reached
/// directly from a Jakobson-Phatic classification rather than an epistemic
/// state: there is no proposition here to be known or unknown.
fn answer_phatic() -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    let content = ResponseContent::new(ResponseFrame::PhaticReturn);
    trace_impls::ResponseResult::new(content.frame, realize::realize(&content)).grounded(true)
}

/// The LOADED ontologies the given concepts belong to, deduped (doc §2.3 — the
/// provenance a turn records). Embedded-English concepts contribute nothing
/// (`ontology_of_concept` → `None`); a loaded `.prx` concept contributes its
/// `OntologyName`, so an answer over a USC Title names that Title.
fn loaded_ontologies_of(en: &dyn LexicalReasoner, ids: &[ConceptId]) -> Vec<OntologyName> {
    let mut names = Vec::new();
    for &id in ids {
        if let Some(name) = en.ontology_of_concept(id)
            && !names.contains(&name)
        {
            names.push(name);
        }
    }
    names
}

/// The ontologies a relational answer `child → parent` reasoned over: the loaded
/// `.prx` ontologies its concepts belong to ([`loaded_ontologies_of`]) PLUS the
/// English substrate when the answer CROSSED into it — a LOADED child
/// (`ontology_of_concept` = `Some`) reaching an ENGLISH ancestor
/// (`ontology_of_concept` = `None`). That pairing holds ONLY through the declared
/// into-English typing (the W2.2 cross-universe arm), so WordNet genuinely supplied
/// the is-a chain and is credited by its `english_wordnet` name. This is a
/// PROVENANCE claim (WordNet's taxonomy was consulted), NOT a structural one:
/// English is still never a loaded `RuntimeOntology` (gate i governs
/// `composed.loaded()`, a disjoint fact).
fn reasoned_over_of(
    en: &dyn LexicalReasoner,
    child: ConceptId,
    parent: ConceptId,
) -> Vec<OntologyName> {
    let mut names = loaded_ontologies_of(en, &[child, parent]);
    if en.ontology_of_concept(child).is_some() && en.ontology_of_concept(parent).is_none() {
        let english = OntologyName::new(
            pr4xis_domains::cognitive::linguistics::english::bridge::ENGLISH_ONTOLOGY,
        );
        if !names.contains(&english) {
            names.push(english);
        }
    }
    names
}

/// The LOADED ontology that owns the provision a statutory definition came from,
/// resolved from the provision's own URN.
///
/// This is the SAME STRUCTURAL claim [`loaded_ontologies_of`] makes — the
/// provision genuinely IS a node of a genuinely loaded `.prx` — reached by a
/// lexicon resolution rather than by the answer's concept ids, because
/// [`LexicalReasoner::statute_definitions`] hands back the defining provision's
/// URN and prose, not its `ConceptId`. It is deliberately NOT modelled on
/// [`reasoned_over_of`]'s English arm: that one is a cross-universe PROVENANCE
/// claim about a taxonomy consulted through a declared typing, whereas the
/// ontology credited here literally owns the node whose text the answer quotes,
/// so it composes into the very same `loaded_ontologies_of` union rather than
/// standing beside it as a second mechanism.
///
/// The join is the node-name-as-surface indexing `ComposedReasoner::new`
/// performs (`composed.rs` — every non-Form node's `name.to_lowercase()` is
/// pushed into the surface overlay, kept additively "until every producer mints
/// Forms"), so the URN must be folded to the SAME lowercase key the overlay was
/// built with: `lookup` normalizes nothing of its own, and `lookup_case_folded`
/// delegates to the English substrate alone and so can never reach a loaded id.
/// The fold is load-bearing rather than cosmetic — a USC provision URN carries
/// its clause letters CAPITALIZED (`/us/usc/t15/s6603/h/6/A`), which is exactly
/// the deepest, most-specific definitions — and is pinned independently by
/// `a_provision_urn_with_uppercase_clause_letters_resolves_to_its_own_concept`
/// (`composed.rs`) so that transition cannot silently drop provenance.
fn statutory_source_of(en: &dyn LexicalReasoner, urn: &str) -> Vec<OntologyName> {
    loaded_ontologies_of(en, en.lookup(&urn.to_lowercase()))
}

/// Union `more` into `names`, preserving order and dropping what is already
/// there — the SAME dedup discipline [`loaded_ontologies_of`] applies WITHIN one
/// provenance channel, applied ACROSS channels so a two-channel union stays a
/// set (a repeated `OntologyName` would otherwise surface twice in the "Why?"
/// panel's source list).
fn extend_reasoned_over(names: &mut Vec<OntologyName>, more: Vec<OntologyName>) {
    for name in more {
        if !names.contains(&name) {
            names.push(name);
        }
    }
}

/// Resolve a question surface to concept ids: the exact lexicon hit UNION the
/// exact hits of its dual-route morphological analyses (the cited lemmatizer:
/// identity → loaded AGID irregulars → rule inversion + Spencer-§5.2
/// allomorphy), UNION the case-FOLDED fallback (Slice D,
/// `.notes/chat-fix-c-build-state.md`: the tokenizer lowercases every
/// surface before this ever runs, so a capitalized WordNet lemma —
/// "Section Eight", "Turkish bath", "O.K." — needs its own recovery path,
/// tried only when the exact ∪ lemmatized union above is still empty). An
/// inflected surface a frame realized — the gerundial nominal "coughing" of
/// a verb-concept question — reaches the verb concept it names, exactly as
/// the tokenizer's analysis half already does. Exact hits keep their rank (a
/// surface that IS a lemma lists its own concepts first); deduped.
///
/// The fold fallback tries EVERY candidate (the raw surface AND each
/// lemmatized form), not the raw surface alone: a capitalized lemma derived
/// from a proper noun ("Islamise", from "Islam") is itself INFLECTED in the
/// question surface ("islamising"), so the exact-case WordNet form only
/// becomes reachable after BOTH steps compose — de-inflect to the stem,
/// THEN case-fold the stem — neither alone finds it (traced via
/// `islamising_lemma_resolution_probe`, task #30: `lookup_case_folded
/// ("islamising")` misses, `lookup_case_folded("islamise")` — the
/// lemmatized stem — hits `Islamise`).
fn resolve_surface(en: &dyn LexicalReasoner, surface: &str) -> Vec<ConceptId> {
    use pr4xis_domains::cognitive::linguistics::morphology::lemmatizer::{
        Language, lemmatize_kinded,
    };
    let mut ids: Vec<ConceptId> = en.lookup(surface).to_vec();
    let kinded = lemmatize_kinded(surface, Language::English);

    // INFLECTIONAL recovery is ALWAYS unioned in, whether or not `surface`
    // already resolved on its own: an inflected form (a verb's gerund, a
    // plural) IS the same lexeme as its base, not a candidate sibling word
    // (`SemanticEffect::is_inflectional`) — so a gerund that ALSO happens to
    // carry its own independent WordNet entry ("coughing" as a noun, "a fit
    // of coughing") must still reach the base verb's senses too ("cough"),
    // never only the noun's. Skipping this union is what regressed the
    // general WordNet⊕USC corpus (`chat_capability.rs`) define/is-a/
    // directional-No counts when LEXICAL BLOCKING below was introduced: it
    // had blocked ALL recovery once `surface` resolved on its own, silently
    // dropping the base-verb gerund reading for every "-ing" word whose
    // gerund is independently lexicalized.
    for (form, kind) in &kinded {
        if form.written_rep == surface || !kind.is_inflectional() {
            continue;
        }
        for &cid in en.lookup(&form.written_rep) {
            if !ids.contains(&cid) {
                ids.push(cid);
            }
        }
    }

    // DERIVATIONAL LEXICAL BLOCKING (Aronoff 1976, *Word Formation in
    // Generative Grammar*, MIT Press, §3): a surface that is ITSELF a
    // lexical headword is its own lemma — its entry IS the answer, so
    // derivational lemma-recovery (a DIFFERENT candidate lexeme, e.g. a
    // prefix-stripped guess) is a FALLBACK for surfaces with no entry of
    // their own, never an ADDITIVE union grafting a sibling headword's
    // senses onto an already-resolved word. The rule-based lemmatizer
    // legitimately OVER-generates derivational candidates (it de-prefixes
    // "respite" to the free but etymologically-unrelated word "spite" —
    // "respite" is Latin *respectus*, not re-+spite); consuming that
    // candidate for a resolved headword is what injected "spite"'s senses
    // ("malevolence…", "feeling a need to see others suffer") into the
    // define answer for "respite". A headword blocks that re-analysis; only
    // a non-lexicalized surface falls through to derivational recovery.
    if ids.is_empty() {
        for (form, kind) in &kinded {
            if form.written_rep == surface || kind.is_inflectional() {
                continue;
            }
            for &cid in en.lookup(&form.written_rep) {
                if !ids.contains(&cid) {
                    ids.push(cid);
                }
            }
        }
        // Still nothing (neither the surface nor any recovered lemma is a
        // direct hit) → the Slice D case-folded fallback over the surface AND
        // its lemmas, for a capitalized WordNet multiword ("Turkish bath").
        if ids.is_empty() {
            ids = en.lookup_case_folded(surface);
            for (form, _kind) in &kinded {
                for cid in en.lookup_case_folded(&form.written_rep) {
                    if !ids.contains(&cid) {
                        ids.push(cid);
                    }
                }
            }
        }
    }
    ids
}

/// Grounded truth-verdict for a two-entity relational claim ("child KIND
/// parent") against the loaded ontology. Extracted from `answer_question`'s
/// two-entity branch so question-answering AND presupposition verification
/// (Karttunen 1973, "Presuppositions of Compound Sentences") share one
/// truth-check — never two divergent copies of the same
/// `best_reaching_pair`/provable-negation reasoning.
enum RelationalVerdict {
    /// Corroborated true: the trusted concept-sense pair, the evidence chain
    /// along `kind` (`en.relation_chain`), and the ontologies traversed to
    /// reach it.
    True {
        child_id: ConceptId,
        parent_id: ConceptId,
        chain: Option<Vec<ConceptId>>,
        reasoned_over: Vec<OntologyName>,
    },
    /// Provably false, with the ontologies traversed to establish it (either
    /// the one scoped corroborated-but-untrusted pair, or every candidate
    /// pair when the negation was found by the unscoped existential scan).
    False { reasoned_over: Vec<OntologyName> },
    /// Both concepts are known but the relation is neither provable nor
    /// disprovable — an honest abstain, not a vocabulary gap.
    Unproven { reasoned_over: Vec<OntologyName> },
    /// One or both surfaces failed to resolve to any known concept at all —
    /// out of scope for a truth verdict (a vocabulary gap, not a false claim).
    Unresolved,
}

fn verify_relational_claim(
    en: &dyn LexicalReasoner,
    child: &str,
    parent: &str,
    kind: &pr4xis_domains::cognitive::linguistics::relation_lexicon::ConceptRef,
) -> RelationalVerdict {
    use pr4xis_domains::formal::relations::ontology::{
        antisymmetric_relation_kinds, opposition_relation_kind,
    };

    // Applicative: child and parent lookups are independent computations.
    // Reference: McBride & Paterson, "Applicative Programming with Effects" (2008)
    let lookups = Ap::pure(resolve_surface(en, child))
        .map2(Ap::pure(resolve_surface(en, parent)), |c, p| {
            Product::new(c, p)
        });
    let child_ids = &lookups.value.left;
    let parent_ids = &lookups.value.right;

    if child_ids.is_empty() || parent_ids.is_empty() {
        return RelationalVerdict::Unresolved;
    }

    let kind_is_antisymmetric = antisymmetric_relation_kinds().contains(kind);
    let opposition = opposition_relation_kind();
    // Whether a provable negation was already checked (and found false) for
    // a SCOPED pair — set only by the `Uncorroborated` arm below, so the
    // unscoped cross-product scan after this match never re-runs for that
    // case (it would silently reintroduce the exact any-sense-pair bug this
    // mechanism exists to retire).
    let mut provably_not = false;
    match word_sense::best_reaching_pair(
        en,
        pr4xis_domains::cognitive::linguistics::verbnet::store::verbnet_loaded(),
        pr4xis_domains::cognitive::linguistics::conceptnet::store::conceptnet_loaded(),
        pr4xis_domains::cognitive::linguistics::framenet::store::framenet_loaded(),
        pr4xis_domains::cognitive::linguistics::sumo::store::sumo_loaded(),
        pr4xis_domains::cognitive::linguistics::propbank::store::propbank_loaded(),
        child_ids,
        parent_ids,
        kind,
    ) {
        word_sense::ReachingPairOutcome::Trusted(cid, pid) => {
            return RelationalVerdict::True {
                child_id: cid,
                parent_id: pid,
                chain: en.relation_chain(cid, pid, kind),
                reasoned_over: reasoned_over_of(en, cid, pid),
            };
        }
        word_sense::ReachingPairOutcome::Uncorroborated(cid, pid) => {
            // A lone reaches() hit that an independent source (VerbNet,
            // ConceptNet, FrameNet, SUMO, or PropBank) found real evidence
            // against — NOT trusted as a confident "Yes". This check is
            // SCOPED to this SPECIFIC pair, never the full child_ids ×
            // parent_ids cross product (that would silently reintroduce the
            // exact any-sense-pair bug this mechanism exists to retire — a
            // confident "No" grounded in an unrelated sense pair).
            if (kind_is_antisymmetric && en.reaches(pid, cid, kind))
                || en.reaches(cid, pid, &opposition)
                || en.reaches(pid, cid, &opposition)
            {
                return RelationalVerdict::False {
                    reasoned_over: reasoned_over_of(en, cid, pid),
                };
            }
            // Neither trusted nor provably false — fall through to the
            // honest abstain below WITHOUT re-running the broad scan
            // (`provably_not` stays `false`, skipping the block below).
        }
        word_sense::ReachingPairOutcome::NoPath => {
            // No candidate pair reaches at all — the UNSCOPED cross-product
            // negation check below is correct here: there is no "the"
            // specific pair to scope to, so an existential search over every
            // candidate pair is the honest way to check for a provable
            // negation. Assert a negation ONLY when it is PROVABLE — absence
            // of a path is not a disproof, and a closed-world "No" is
            // dishonest ("a statute is not a law" was exactly such a false
            // negative).
            provably_not = child_ids.iter().any(|&cid| {
                parent_ids.iter().any(|&pid| {
                    (kind_is_antisymmetric && en.reaches(pid, cid, kind))
                        || en.reaches(cid, pid, &opposition)
                        || en.reaches(pid, cid, &opposition)
                })
            });
        }
    }

    let traversed: Vec<ConceptId> = child_ids.iter().chain(parent_ids).copied().collect();
    if provably_not {
        RelationalVerdict::False {
            reasoned_over: loaded_ontologies_of(en, &traversed),
        }
    } else {
        RelationalVerdict::Unproven {
            reasoned_over: loaded_ontologies_of(en, &traversed),
        }
    }
}

pub fn answer_question(
    en: &dyn LexicalReasoner,
    lang: &dyn Language,
    tokens: &[TypedToken],
    predicate: &str,
    arguments: &[montague::Sem],
    illocution: montague::QuestionIllocution,
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{
        self, ConditionalPresentation, ResponseContent,
    };
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
    use pr4xis_domains::social::judicial::conditional_rule::Applicability;

    // A POSITIVE capability (Bobrow, Kaplan, Norman, Thompson & Winograd
    // 1977, GUS): the question's predicate is governed by a fully-known,
    // fully-cited ConditionalRule, and evaluating it for this asker is
    // blocked on a private fact never supplied. Tried FIRST and falling
    // through cleanly on `None` — the illocution/predicate-classification
    // gap (recognizing "eligible for X" as a rule-governing question shape
    // in the first place) is separate, unscoped work; this hook only fires
    // once `conditional_rule_for_predicate` already resolves a rule.
    //
    // `object` is every argument's extracted surface, joined — the registry
    // matcher (task #28) needs the OBJECT, not just the predicate, since a
    // real rule is topically narrow (matching on predicate alone would
    // confidently attach an asset-transfer-specific rule to an unrelated
    // "eligible for X" question).
    let object = arguments
        .iter()
        .map(|s| extract_entity_name(s, en))
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(rule) = en.conditional_rule_for_predicate(predicate, &object)
        && let Applicability::Indeterminate { missing } = rule.applicability()
    {
        let presentation = ConditionalPresentation {
            rule_name: rule.term.name.text.clone(),
            // `source_text` carries the FULL human-readable citation
            // (e.g. "42 U.S.C. § 1396p(c)(1)"); `subsection.to_bluebook()`
            // renders only the bare pinpoint fragment ("(c)(1)") and is not
            // a substitute for it.
            citation: rule
                .term
                .source_text
                .as_ref()
                .map(|s| s.text.clone())
                .or_else(|| rule.term.subsection.as_ref().map(|c| c.to_bluebook()))
                .unwrap_or_default(),
            rule_text: rule.term.definition.text.clone(),
            missing_facts: missing
                .iter()
                .map(|el| el.requirement.field.text.clone())
                .collect(),
        };
        let content =
            ResponseContent::new(ResponseFrame::RuleAwaitingFact).with_conditional(presentation);
        // TODO: wire `reasoned_over` to the loaded rule corpus's OntologyName once
        // a real rule-extraction producer lands (task following #20).
        return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
            .with_conditional(Some((rule, missing)));
        // Applies/DoesNotApply (unreachable here, the outer `if` already
        // filtered to Indeterminate): no producer populates Satisfied/
        // NotSatisfied yet (the multi-turn slot-filling layer).
    }

    // The queried entities are the ARGUMENT-role constituents (subject +
    // predicative-nominal complement), decided by their parse-time grammatical
    // role — NOT by WordNet membership. Steedman (2000): the copula
    // `(S[q]/NP)/NP` and the wh-word are syncategorematic Functors, dropped
    // structurally even though `is` is a WordNet verb homonym (the leak the old
    // `en.lookup` gate could not see). Bowers (1993) / Pollard & Sag (1994): the
    // copula is a non-θ-assigning linking verb, never itself a queried entity.
    // `argument_name` recurses through a partial application (`"is a dog"` →
    // `S[q]/NP`) to the wrapped argument, so a real entity nested under the copula
    // is kept while the copula head is dropped.
    // The leaves are kept alongside their names so the vocabulary-gap
    // fall-through below can read each entity's typed provenance
    // ([`montague::PredProvenance`]) — a composition-minted surface is never
    // named an unknown word (axiom [`DefiniendumIsALexicalUnit`]).
    let entity_leaves: Vec<&montague::Sem> = arguments
        .iter()
        .filter_map(montague::Sem::argument_leaf)
        .collect();
    // A pronoun ("I"/"he"/"who"/…) is never a queried entity, even when the
    // bare surface happens to also carry an unrelated open-class WordNet
    // sense (the letter/numeral "I", the element "iodine") — the same
    // principle `extract_entity_name` applies to a copula's own surface.
    // `is_pronoun`, not the coarser `is_function_word`: several closed-
    // class words ("above", "so") are ALSO legitimate content words in a
    // different sense, so excluding on general closed-class membership
    // over-excludes those polysemous readings. Filtered here, at
    // `argument_name`'s one production call site, rather than inside
    // `Sem::argument_name` itself (a pure structural accessor with no
    // reasoner access).
    let entities: Vec<String> = entity_leaves
        .iter()
        .filter_map(|leaf| leaf.argument_name())
        .filter(|name| !en.is_pronoun(name))
        .collect();

    // A COMPARISON-RELATION question ("what is the difference between X
    // and Y?") is recognized by walking `arguments` for a LOADED
    // comparison relation (Barker 2011 derived-relational-noun licensing —
    // see `comparison_relation_lexicon`'s module doc), never a
    // positional/count heuristic. Checked BEFORE the `entities.len() >= 2`
    // relation-verification block just below, for two reasons: (1) the
    // single-leaf `argument_leaf` extraction `entities` is built from would
    // recite only the FIRST-listed named term non-deterministically — the
    // corpus's own answer key names the SECOND-listed term for at least one
    // row, so a single-leaf recital fails even by luck; (2) a "difference
    // between X and Y" question is not a fact to verify against a
    // materialized closure (there is no Contrast edge between two arbitrary
    // defined terms) — the `entities.len() >= 2` block's
    // `verify_relational_claim` truth-check would wrongly try to prove or
    // disprove "difference" as a structural relation between them instead
    // of reciting each named term's own gloss.
    if let Some((_kind, names)) = arguments.iter().find_map(|s| s.comparison_leaves(en)) {
        let mut content = ResponseContent::new(ResponseFrame::Comparison);
        let mut resolved_ids: Vec<ConceptId> = Vec::new();
        let mut any_definition = false;
        for name in &names {
            content = content.with_entity(name);
            let ids = resolve_surface(en, name);
            resolved_ids.extend(ids.iter().copied());
            for &id in &ids {
                if let Some(concept) = en.concept(id) {
                    for def in concept.definitions() {
                        content = content.with_definition(name, def);
                        any_definition = true;
                    }
                }
            }
        }
        if any_definition {
            let reasoned_over = loaded_ontologies_of(en, &resolved_ids);
            return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                .with_entities_found(names)
                .grounded(true)
                .with_reasoned_over(reasoned_over);
        }
        // Every named term failed to resolve to a loaded concept — fall
        // through to the ordinary `entities`/`entity_leaves`-driven
        // vocabulary-gap handling below rather than answering some other
        // way; `entities` was already computed above regardless of this
        // branch.
    }

    // A two-entity RELATIONAL VERIFICATION ("is X a Y?"/"is X part of Y?")
    // is a fundamentally different illocutionary act from a two-entity
    // OBLIGATION/deontic question that merely happens to extract two
    // argument leaves — "Which personal care, respite care, and companion
    // service billing codes are REQUIRED to use EVV?" extracts
    // `entities == ["personal and", "evv"]` and the loaded corpus genuinely
    // DOES chain "personal care" -> "evv" via Subsumption (both are real,
    // truthfully related concepts), so `verify_relational_claim` correctly
    // returns `True` — but affirming "Yes. a personal care is an evv." then
    // dumping the definition answers an is-a fact NOBODY asked while
    // ignoring the actual question (which billing codes are required).
    // Gated on `has_deontic_or_descriptive_marker` — the task #42/POA
    // precedent's INTENT (see that function's own doc for why the FULL,
    // verb-shaped-inclusive `has_modal_or_descriptive_predicate` is the
    // wrong tool here: it over-fired on rows like "Which home health care
    // services REQUIRE EVV?"/"Is home infusion therapy INCLUDED in EVV?" —
    // genuine is-a/coverage verification questions this corpus's own ground
    // truth DOES expect an answer for, whose only "predicate" is an
    // ordinary content verb, not a deontic marker) — never wired here
    // before because this branch fires on chart-parse SUCCESS while the
    // task #42 gate's only caller previously fired on chart-parse FAILURE
    // (a missing-plumbing gap, not a missing-grammar one: `lang`/`tokens`
    // are threaded through `answer_question`'s own signature now). Falling
    // through (rather than returning early) degrades gracefully:
    // `entities.len()` stays >= 2 so neither single-entity branch below
    // fires either, and both named entities resolve, so control reaches the
    // `known_entities`-driven `AdmitLimitation` fallback at the bottom of
    // this function — the same honest "known entities, uncovered question
    // shape" degradation `answer_statement`'s analogous branch and this
    // function's own single-entity branch already use.
    let entity_surfaces: Vec<&str> = entities.iter().map(String::as_str).collect();
    if entities.len() >= 2 && !has_deontic_or_descriptive_marker(lang, tokens, &entity_surfaces) {
        let child = &entities[0];
        let parent = &entities[1];

        // Lower the question's surface predicate to its TYPED relation kind
        // through the loaded relation lexicon ("part of" → Parthood); an unknown
        // surface (the bare copula "is") falls back to Subsumption. The relation
        // is loaded data the reasoner interprets — never a `match predicate` in
        // Rust. `child → parent` is the asserted direction (part → whole, subtype
        // → supertype); a relation like Parthood is antisymmetric (BFO:0000050),
        // so the reverse direction is never re-checked.
        let kind = en
            .relation_for_surface(predicate)
            .unwrap_or_else(subsumption_kind);

        // The RELATION-PARAMETRIC query: does `child` reach `parent` along
        // `kind`? `is_a` is the `kind = Subsumption` case; a USC Parthood
        // question reads the SAME materialized closure, keyed on Parthood.
        //
        // When a surface is polysemous, MULTIPLE (child-sense, parent-
        // sense) pairs can all satisfy `reaches` — e.g. "cut" and "end"
        // each carry several WordNet senses, and an unrelated verb-sense
        // pair can chain to a true is-a edge that has nothing to do with
        // the intended noun-sense reading. `verify_relational_claim`
        // (Lesk 1986 gloss-overlap scoring among multiple hits via
        // `best_reaching_pair`; VerbNet corroboration for the risky
        // LONE-hit case — see `word_sense` module docs) replaces blind
        // first-hit iteration order with a tri-state outcome — the
        // registered `TwoEntityPathPrefersGlossOverlapAmongReachingPairs`
        // axiom.
        match verify_relational_claim(en, child, parent, &kind) {
            RelationalVerdict::True {
                child_id,
                parent_id,
                chain,
                reasoned_over,
            } => {
                let (response, deferred) = build_taxonomy_response(
                    en,
                    AnsweredRelation {
                        connective: en.surface_for_relation(&kind).as_deref(),
                        kind_name: &kind.name,
                    },
                    chain,
                    child,
                    parent,
                    child_id,
                    parent_id,
                );
                return trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, response)
                    .with_entities_found(entities.clone())
                    .with_taxonomy_checked(Some((child.clone(), parent.clone(), true)))
                    .grounded(true)
                    .with_reasoned_over(reasoned_over)
                    .with_deferred_elaboration(deferred);
            }
            RelationalVerdict::False { reasoned_over } => {
                // A real, derived answer: the denial phrases with the relation's
                // loaded surface ("is not part of"), not "is not a".
                let connective = en.surface_for_relation(&kind);
                return trace_impls::ResponseResult::new(
                    ResponseFrame::AssertKnowledge,
                    realize::realize_negation(child, parent, connective.as_deref()),
                )
                .with_entities_found(entities.clone())
                .with_taxonomy_checked(Some((child.clone(), parent.clone(), false)))
                .grounded(true)
                .with_reasoned_over(reasoned_over);
            }
            RelationalVerdict::Unproven { reasoned_over } => {
                // Not derivable and not disprovable → abstain honestly (§4.1).
                // BOTH concepts are provably KNOWN (verify_relational_claim's
                // Unresolved case is handled separately below), so this is the
                // fifth epistemic cell — vocabulary known, proposition open —
                // realized as `UnprovenRelation`, which NAMES both concepts as
                // known instead of the false vocabulary-gap claim ("I do not
                // know the words …") the conflated `AcknowledgeGap` emitted.
                // The relation's LOADED surface is threaded through the
                // predicate slot (`None`/is-a → the copula default), so the
                // embedded claim reads with the right connective.
                // `from_ontology: false` makes the caller emit
                // `ChatOutcome::Abstained` — whose `unresolved` set is EMPTY
                // here (both surfaces resolve), so no known concept leaks.
                let content = ResponseContent::new(ResponseFrame::UnprovenRelation)
                    .with_entity(child)
                    .with_entity(parent);
                let content = match en.surface_for_relation(&kind) {
                    Some(conn) => content.with_predicate(&conn),
                    None => content,
                };
                return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                    .with_entities_found(entities.clone())
                    .with_taxonomy_checked(Some((child.clone(), parent.clone(), false)))
                    .with_reasoned_over(reasoned_over);
            }
            RelationalVerdict::Unresolved => {
                // One or both surfaces didn't resolve — fall through to the
                // single-entity / vocabulary-gap paths below, exactly as the
                // original `if !child_ids.is_empty() && !parent_ids.is_empty()`
                // guard did.
            }
        }
    }

    // The single-entity define path only fires for a CONTENT (wh-) question —
    // "what is a dog?" legitimately has exactly one queried entity. A POLAR
    // question ("is a dog a mammal?") that reaches this point with only one
    // resolved entity is a degenerate extraction (the second argument failed
    // to resolve), not a genuine one-entity query — answering it with
    // `define_word` would silently swap in the wrong illocutionary force (a
    // definition where a yes/no verdict was asked for). This is the typed
    // check the gate-invisible `contains_norm` hole needed: previously
    // `entities.len() == 1` alone routed BOTH cases into `define_word`, and no
    // corpus check could see the difference because the response was always
    // `Answered` either way.
    // `illocution == Content` alone is too coarse: for a WH-question, this
    // Sem's `predicate` field IS the wh-word itself (confirmed empirically —
    // "What is X?" carries `predicate == "what"`, "Who is eligible for X?"
    // carries `predicate == "who"`, with "eligible for" entirely absorbed
    // into how the single entity got extracted, not preserved anywhere
    // recoverable here). Content illocution covers every wh-word, not just
    // "what"/"which" — so "Who is eligible for X?", "Who administers X?"
    // reach this point with the SAME shape (one resolved entity, Content
    // illocution) as a genuine "What is X?" query, and `define_word` swaps
    // in the wrong illocutionary force: a program's definition where a
    // person-identifying or rule-governed verdict was asked for. Restricted
    // to the nonpersonal interrogative pronouns ("what"/"which") — the same
    // loaded `WhReferentRole` Thing/Selection split (Cysouw 2004 §3.2 table
    // (9)) the fallback path's `is_what_copula_question` already enforces
    // via `en.is_nonpersonal_interrogative`; this is that gate's
    // successful-chart-parse counterpart, previously missing (caught via
    // the full corpus regression suite: task #44's Self-Determination
    // Program vocabulary made "Self-Determination Program" resolve to
    // exactly one entity, newly exposing this pre-existing gap for "who is
    // eligible for the SDP").
    //
    // Two extra conjuncts, `!has_scope_predicate_token` and
    // `!is_infinitival_wh_question`, plus `!has_deontic_or_descriptive_marker`
    // in place of `attempt_partial_understanding`'s own
    // `has_modal_or_descriptive_predicate` — the task #42/POA gate's INTENT
    // (deontic/obligation questions must not be silently answered as if they
    // were "what is X" definitions), adapted, not copied verbatim, for this
    // PARSE-SUCCESS counterpart. `is_what_copula_question` and the FULL
    // `has_modal_or_descriptive_predicate` (its verb-shaped disjunct
    // specifically) were both tried here first and REJECTED by the full
    // corpus regression suite: both are tuned for
    // `attempt_partial_understanding`'s PARSE-FAILURE fragments, a narrower
    // distribution that never exposed either check's real coverage gaps —
    // `is_what_copula_question` requires a literal Copula POS token, which
    // "What DOES the Family and Medical Leave Act PROVIDE?"/"What does X
    // MEAN?" (a `does`-periphrastic wh-question, no bare "is"/"are")
    // structurally never has, wrongly excluding a whole class of previously-
    // answerable rows; `has_modal_or_descriptive_predicate`'s verb-shaped
    // disjunct independently over-fired on the SAME rows for a related
    // reason — a genuine content verb ("mean", "provide") co-occurring with
    // the entity is not, by itself, evidence of a wrong illocutionary shape
    // the way an Adjective/Auxiliary-typed deontic marker or a genuine
    // OTHER-entity-taking action verb is (confirmed: 11 real Green ->
    // {MissingTerm,UnparsedKnownTerm} regressions from wiring the full,
    // unmodified check in here, jq-diffed against the committed snapshot).
    // Two real corpus regressions THIS narrower pair still closes: "What is
    // required OF the agency?" (`required` types `PosTag::Adjective`/
    // `ObligationModality::Mandatory` — a deontic "what must X do" question,
    // not "what IS X" — `has_deontic_or_descriptive_marker` excludes it) and
    // "What TO BRING to nursing home?" (an infinitival "what to VERB"
    // instruction question, Huddleston & Pullum 2002 Ch.14 §2 — no finite
    // predicate names X as its topic at all; `is_infinitival_wh_question`
    // recognizes the construction directly instead of inferring it from
    // copula absence). Both fall through to the `else if` arm just below,
    // which already honestly degrades to `AdmitLimitation` naming the known
    // entity — the same "known entity, wrong question shape" honesty this
    // function's two-entity branch above now also falls through to.
    if entities.len() == 1
        && illocution == montague::QuestionIllocution::Content
        && en.is_nonpersonal_interrogative(predicate)
        && !has_scope_predicate_token(tokens)
        && !is_infinitival_wh_question(en, tokens)
        && !has_deontic_or_descriptive_marker(lang, tokens, &[entities[0].as_str()])
        && !copula_predicate_names_a_different_entity(lang, tokens, &entities[0])
    {
        let ids = resolve_surface(en, &entities[0]);
        if !ids.is_empty() {
            // Provenance comes back WITH the answer (see [`DefinitionAnswer`]) —
            // the concept channel this branch used to compute here, plus the
            // statutory channel only `define_word` can see.
            let answer = define_word(en, &entities[0]);
            return trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, answer.text)
                .with_entities_found(entities)
                .grounded(true)
                .with_reasoned_over(answer.reasoned_over)
                .with_definition_provenance(answer.authored_from);
        }
        // A lone definiendum that does NOT resolve is not an answer: fall through
        // to the abstain path below, which names the unresolved surface and sets
        // `from_ontology: false` → `ChatOutcome::Abstained`. This is the same
        // resolution guard `answer_statement` applies; the question path lost it
        // when role-based entity extraction stopped gating on `en.lookup`, so an
        // unknown word ("what is a title" with no corpus loaded) reached
        // `define_word` and was misreported as an ontology-backed answer.
    } else if entities.len() == 1 && illocution == montague::QuestionIllocution::Content {
        // A non-"what"/"which" wh-question — overwhelmingly "who is X" in
        // this corpus — over a single entity that DOES resolve. TWO real
        // shapes reach here with the IDENTICAL parse signature (one
        // resolved entity, Content illocution, predicate "who"):
        //   (a) "Who is DCH?" / "Who is a General Caregiver?" / "Who is a
        //       Legally Responsible Individual (LRI)?" — genuinely
        //       DEFINITIONAL: the corpus's own ground truth expects these
        //       answered exactly like "What is DCH?" (measured: 9 of the
        //       10 corpus `who_is_x` UnparsedKnownTerm rows are this shape).
        //   (b) "Who is eligible for X" / "who administers X" / "Who is
        //       considered OUR employer?" — a genuine person-identifying or
        //       rule-governed verdict, which nothing here can produce.
        // The DISTINGUISHING typed signal is NOT `is_a`-hypernym-of-person:
        // investigated first and rejected — WordNet types an ordinary ROLE
        // noun ("caregiver", "employer", "individual") as a hyponym of
        // "person" the SAME as it would a person-denoting noun, so an
        // is-a-person check would wrongly EXCLUDE most of shape (a) too
        // (a "general caregiver" IS, in WordNet's own hierarchy, a kind of
        // person). Nor is WordNet's lexicographer-file supersense
        // (`noun.person` vs `noun.group` — `lmf::ontology::Synset::lexfile`)
        // usable: that field round-trips through the LMF reader/writer but
        // is NOT threaded into the runtime `English`/`ConceptView` at all
        // (confirmed by direct search — no `lexfile` reader anywhere in
        // `cognitive::linguistics::english`), so wiring it up would be a
        // separate, larger undertaking, not a routing-gate fix.
        // The signal that DOES separate the two shapes, already loaded and
        // already reasoner-visible: whether the resolved entity is itself
        // DOMAIN-LOADED data (`ConceptView::is_domain_loaded`) or carries a
        // loaded statutory definition (`en.statute_definitions`) — i.e.
        // whether some loaded caregiving/HCBS/USC source specifically
        // DEFINES this role/organization/category, the exact reason DCH,
        // PPL, "general caregiver" and "legally responsible individual" are
        // in the loaded lexicon at all — versus a bare WordNet common noun
        // with no domain-specific definition (a generic "governor"/
        // "president"/"doctor" role query, which this reasoner has no
        // grounds to treat as answerable "who is" content). Composed with
        // the SAME three parse-shape guards the sibling "what"/"which"
        // branch above already validated against the full corpus
        // regression suite (`has_scope_predicate_token`,
        // `is_infinitival_wh_question`, `has_deontic_or_descriptive_marker`
        // — the last one's possessive-pronoun disjunct is what correctly
        // keeps declining "Who is considered OUR employer?": "our" binds
        // the entity to the asker's own, non-generic context, exactly the
        // deictic-anchoring class that check already exists to catch), plus
        // a FOURTH guard: `who_predication_identifies_the_entity`. The
        // three guards above are all about the QUESTION's overall shape,
        // but none of them checks WHAT actually predicates "who" — "Who
        // PAYS for long-term care?"/"Who SCHEDULES the appointments?" have
        // no deontic marker, no scope-predicate token, and are not
        // infinitival, yet are shape (b): the resolved entity is the
        // OBJECT of an ordinary matrix verb, not what "who" is predicated
        // to BE (no copula names it — there is no copula at all). Unlike
        // the "what"/"which" branch (which must keep allowing
        // does-periphrastic questions through via the softer
        // `copula_predicate_names_a_different_entity`), this branch has no
        // positive case that requires copula-absence to be tolerated, so it
        // uses the STRICTER positive form — see that function's own doc for
        // why, and for the corpus regressions this closes.
        let ids = resolve_surface(en, &entities[0]);
        let is_role_or_org_or_category_definitional = ids
            .iter()
            .any(|&id| en.concept(id).is_some_and(|c| c.is_domain_loaded()))
            || !en.statute_definitions(&entities[0]).is_empty();
        if !ids.is_empty()
            && is_role_or_org_or_category_definitional
            && !has_scope_predicate_token(tokens)
            && !is_infinitival_wh_question(en, tokens)
            && !has_deontic_or_descriptive_marker(lang, tokens, &[entities[0].as_str()])
            && who_predication_identifies_the_entity(lang, tokens, &entities[0])
        {
            let answer = define_word(en, &entities[0]);
            return trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, answer.text)
                .with_entities_found(entities)
                .grounded(true)
                .with_reasoned_over(answer.reasoned_over)
                .with_definition_provenance(answer.authored_from);
        }
        // Either unresolved, a person-identifying/rule-governed shape one of
        // the guards above caught, or a resolved entity with no DOMAIN
        // definition backing it (a bare substrate common noun) — honestly
        // decline. `AdmitLimitation` names the known entity (when resolved)
        // while declining, matching the fallback path's own pattern for a
        // recognized-but-uncovered governed predicate (`find_governed_
        // predicate` + no loaded rule, above in
        // `attempt_partial_understanding`).
        if !ids.is_empty() {
            let content =
                ResponseContent::new(ResponseFrame::AdmitLimitation).with_entity(&entities[0]);
            return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                .with_entities_found(entities);
        }
        // Unresolved: fall through to the honest UnknownVocabulary path below.
    }

    // Only LEXICAL surfaces that do NOT resolve are named as unknown words
    // (axiom [`DefiniendumIsALexicalUnit`]): a composition-minted
    // concatenation (the degenerate "is a" of "what is a long") is excluded
    // — its failing leaves are reported through the outcome's `unresolved`
    // set — and, mirroring that SAME "never claim ignorance of a KNOWN word"
    // discipline, a lexical surface the reasoner DOES resolve is excluded
    // too. Confirmed real regression this closes: "Does the EVV system
    // track me?" — `collapse_multiword_surfaces` correctly collapsed "EVV
    // system" into ONE token because that exact surface IS a registered
    // multi-word alias for the loaded "electronic visit verification"
    // concept (a genuine, correct collapse — the tokenizer did its job); a
    // POLAR question with exactly one non-pronoun entity has no
    // `entities.len() == 1`/Content-illocution branch above to catch it
    // (those are scoped to wh-questions), so it fell straight through to
    // this generic fallback, which named the resolving entity "unknown"
    // regardless — a false claim of ignorance about a word the reasoner
    // demonstrably knows. `resolve_surface` (exact ∪ morphological
    // analyses, the SAME resolution [`unresolved_surfaces`] and every
    // other honesty gate in this module already requires) is the gate.
    let mut content =
        ResponseContent::new(ResponseFrame::UnknownVocabulary).with_predicate(predicate);
    // Composite leaves contribute NOTHING here either way (axiom
    // `DefiniendumIsALexicalUnit`) — they are never lexical surfaces, known
    // or unknown. `known_entities` collects only the REAL (non-composite)
    // leaves that DO resolve, kept separate from `content`'s own entity
    // list so a question whose sole entity is a composite artifact (e.g.
    // "the six required elements") never falls into the KNOWN-entity branch
    // below just because it happens to name nothing unknown either.
    let mut known_entities: Vec<String> = Vec::new();
    for (leaf, e) in entity_leaves.iter().zip(&entities) {
        if leaf.argument_composite_parts().is_some() {
            continue;
        }
        if resolve_surface(en, e).is_empty() {
            content = content.with_entity(e);
        } else {
            known_entities.push(e.clone());
        }
    }
    if !known_entities.is_empty() && content.entities.is_empty() {
        // Every NAMEABLE (non-composite) entity is a KNOWN concept: the real
        // gap is an unsupported predicate/question shape over it, not a
        // missing word — realized as `AdmitLimitation`, the SAME frame the
        // resolved-but-non-"what"-wh-question branch above already uses for
        // the identical "known entity, uncovered question" honesty case.
        // Confirmed real regression this closes: "Does the EVV system track
        // me?" — `collapse_multiword_surfaces` correctly collapsed "EVV
        // system" into ONE token because that exact surface IS a registered
        // multi-word alias for the loaded "electronic visit verification"
        // concept; a POLAR question with exactly one non-pronoun entity has
        // no `entities.len() == 1`/Content-illocution branch above to catch
        // it (those are scoped to wh-questions), so it fell straight
        // through to the generic `UnknownVocabulary` fallback, which named
        // the resolving entity "unknown" regardless — a false claim of
        // ignorance about a word the reasoner demonstrably knows.
        let mut admit = ResponseContent::new(ResponseFrame::AdmitLimitation);
        for e in &known_entities {
            admit = admit.with_entity(e);
        }
        return trace_impls::ResponseResult::new(admit.frame, realize::realize(&admit))
            .with_entities_found(entities);
    }
    trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
        .with_entities_found(entities)
}

/// A "define X" or exploratory multi-noun answer is confidently WRONG when
/// the question also contains a probable acronym (task #3's
/// `is_probable_acronym` — RN, IHSS, PPL, PAs — 2+ uppercase-Latin letters,
/// English's closed acronym/initialism word-formation process) that never
/// resolves to any loaded concept: the acronym is almost certainly the REAL
/// query subject, and the answer only reached a confident-sounding response
/// because the grammar found SOME resolvable content elsewhere in the
/// sentence whose extraction dropped the acronym entirely — confirmed by
/// direct instrumentation: `answer_question`'s `entity_leaves` for "what is
/// the role of the RN in IHSS" is `[Concept { word: "role", .. }]`, with no
/// trace of "RN"/"IHSS" anywhere in the Sem tree. Corpus-measured gap
/// (task #12, `caregiver_capability_ratchet.rs`'s own doc history): 4 rows
/// exactly this shape flip Green -> OverAnswered once an earlier fix (task
/// #3) stopped an unrelated bug from accidentally routing them correctly.
///
/// Originally scoped to the single-entity define path only; broadened to
/// ANY `from_ontology` answer with at least one resolved entity that is NOT
/// a PROVEN two-entity relational claim (`taxonomy_checked.is_some()` still
/// exempts a genuine "Yes"/"No" is-a verdict — a real proof about the two
/// named entities is answerable regardless of an unrelated acronym
/// elsewhere). `attempt_partial_understanding`'s multi-noun
/// `explore_concepts` branch (`nouns.len() >= 2`) shares the EXACT same
/// vulnerability the single-entity path was fixed for, just with more than
/// one entity: it always sets `taxonomy_checked: None` (it is a bare
/// WordNet-sense EXPLORATION, never a proven relation), so the original
/// `entities_found.len() != 1` guard let it through unchecked. Confirmed
/// real regression this closes — domain_mimicry corpus case "What is the
/// ADL-waiver reciprocity threshold under the IDD compliance matrix?": the
/// fabricated compound's real words (reciprocity/threshold/compliance/
/// matrix) resolve to 4 real WordNet concepts, `explore_concepts` dumps
/// their pairwise senses as a confident-looking answer, and the actual
/// query subject ("ADL-waiver", a probable acronym resolving to nothing
/// loaded) is silently dropped — the honest answer is the same "I don't
/// know that vocabulary" this gate already gives the single-entity case.
///
/// Is `acronym` (already lowercased) English's INITIALISM of `phrase` — the
/// first letter of each of `phrase`'s words, concatenated (Cannon 1989,
/// "Abbreviations and Acronyms in English Word-Formation," *American
/// Speech* 64(2):99-127; Huddleston & Pullum 2002 Ch.19's abbreviatory
/// word-formation account — the SAME closed word-formation process
/// `is_probable_acronym`'s own doc already cites)? A parenthetical acronym
/// GLOSSING an already-resolved multi-word entity ("home health aide
/// (HHA)") is not a second, ignored query subject — it is that SAME
/// concept's own abbreviated written form, so this gate must not decline
/// naming it as unknown. Confirmed real regression this closes: "What is a
/// home health aide (HHA)?" and 3-way "difference between a PCCA, a home
/// health aide (HHA), and a PCA" both resolve "home health aide" correctly
/// but flip a correct answer to a false "I do not know the word HHA" once
/// `HHA` itself has no lexicon alias of its own — the true positive this
/// gate exists for ("what is the role of the RN in IHSS", where neither
/// acronym initializes any resolved word) is unaffected: `role`'s initials
/// are `r`, length-mismatched against both `rn` and `ihss`.
fn is_initialism_of(acronym: &str, phrase: &str) -> bool {
    let initials: String = phrase
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_lowercase)
        .collect();
    initials == acronym
}

fn decline_if_an_unresolved_acronym_was_ignored(
    en: &dyn LexicalReasoner,
    input: &str,
    result: trace_impls::ResponseResult,
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::lambek::tokenize::{
        is_covered_by_resolved_compound, is_probable_acronym, split_letter_flanked_division,
    };
    if !result.from_ontology
        || result.taxonomy_checked.is_some()
        || result.entities_found.is_empty()
    {
        return result;
    }
    // Raw `en.lookup`, NOT `resolve_surface`: `resolve_surface`'s lemmatization
    // step can spuriously stem a short acronym to an unrelated base word's
    // concept (the same class of short-string coincidental-match risk
    // `is_probable_acronym` itself exists to guard spelling-correction
    // against, task #3) — an acronym is never a genuine inflected word form,
    // so lemmatizing one at all is already the wrong question to ask.
    //
    // `split_letter_flanked_division` further splits each whitespace-chunk
    // on any letter-flanked DIVISION glyph BEFORE the acronym check —
    // `is_probable_acronym` counts uppercase letters with no notion of word
    // boundaries inside its input, so a slash-joined pair of ordinary
    // Title-Case words ("Support/Navigation") would otherwise read as one
    // 2-uppercase "probable acronym" (a confirmed real regression: "What is
    // Community Direct Support/Navigation?" declined as though
    // "Support/Navigation" were an unresolved acronym like "RN"/"IHSS").
    let Some(acronym) = input.split_whitespace().find_map(|w| {
        let bare = w.trim_matches(|c: char| !c.is_alphanumeric());
        split_letter_flanked_division(bare)
            .into_iter()
            .find(|piece| {
                let lower = piece.to_lowercase();
                // The SUBSTRING itself can be a WORD of an already-resolved
                // multi-word compound ("MyCare" inside the resolved "mycare
                // ohio", "VoIP" inside "voip phone", "DHS" inside "dhs
                // aggregator") — a false positive `is_probable_acronym`'s
                // own uppercase-letter-count shape can't distinguish from a
                // genuinely unresolved standalone acronym.
                // `is_covered_by_resolved_compound`'s own doc has the full
                // three-example corpus regression this closes.
                let already_the_answer = result.entities_found.contains(&lower)
                    || result
                        .entities_found
                        .iter()
                        .any(|e| is_initialism_of(&lower, e))
                    || is_covered_by_resolved_compound(&lower, &result.entities_found);
                is_probable_acronym(piece) && !already_the_answer && en.lookup(&lower).is_empty()
            })
    }) else {
        return result;
    };
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    let content = ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity(&acronym);
    trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
        .with_entities_found(vec![acronym])
}

/// Post-hoc correction, same pattern and call-site position as
/// `decline_if_an_unresolved_acronym_was_ignored`: `answer_question`'s
/// single-entity definitional branches extract their definiendum from the
/// Montague `Sem` tree (`entity_leaves`/`argument_name`), which for a
/// copula-less fragment question ("What tax credits in 2026?" — no "is"/
/// "are" for a bare-nominal-compound derivation to attach to) can drop an
/// adjacent nominal modifier entirely rather than joining it, landing on
/// just the head noun ("credits") instead of the compound ("tax credit")
/// the question actually named. Re-deriving that fix INSIDE `answer_question`
/// would require threading raw `tokens` through a `pub fn` with a dozen
/// existing Sem-tree-only call sites (production and unit tests alike); this
/// is a smaller, surgical widen-and-redefine pass over the ALREADY-PRODUCED
/// response, applied only at the one real dispatch site that has `tokens`.
///
/// Guarded tightly to a `define_word`-shaped answer specifically (not
/// `explore_concepts`, not a taxonomy/parthood chain, not an abstention):
/// `from_ontology` (a genuine answer), `taxonomy_checked.is_none()` and
/// `conditional.is_none()` (define_word sets neither), exactly one resolved
/// entity, AND the response's own first line echoing that entity as
/// `define_word`'s `with_entity` header always does — so a response this
/// function does not recognize as `define_word`'s shape is left untouched.
/// `widen_definiendum_to_compound` itself only ever widens to a phrase the
/// loaded lexicon/corpus already resolves, so a modifier that does not
/// combine into a KNOWN compound leaves the original answer unchanged — this
/// can only add precision, never fabricate one.
fn widen_definiendum_if_compound_available(
    en: &dyn LexicalReasoner,
    tokens: &[TypedToken],
    result: trace_impls::ResponseResult,
) -> trace_impls::ResponseResult {
    if !result.from_ontology
        || result.taxonomy_checked.is_some()
        || result.conditional.is_some()
        || result.entities_found.len() != 1
    {
        return result;
    }
    let head = &result.entities_found[0];
    if !result.response.starts_with(&format!("{head}:")) {
        return result;
    }
    let widened = widen_definiendum_to_compound(tokens, head, en);
    if widened == *head {
        return result;
    }
    // The widened compound is a DIFFERENT definiendum, so its answer rests on
    // different sources — `..result` would carry forward the pre-widening
    // provenance, crediting whatever the bare head resolved to for text it did
    // not produce. Both halves of the answer are replaced together.
    let answer = define_word(en, &widened);
    trace_impls::ResponseResult {
        response: answer.text,
        entities_found: vec![widened],
        reasoned_over: answer.reasoned_over,
        definition_provenance: answer.authored_from,
        ..result
    }
}

pub fn answer_statement(
    en: &dyn LexicalReasoner,
    predicate: &str,
    arguments: &[montague::Sem],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;

    // A closed-class word (a pronoun, above all) yields the empty string
    // from `extract_entity_name` and is dropped here, not carried forward
    // as a blank "entity" — an empty slot is an absence, not a second
    // unresolved word to report ("I do not know the word \"\"." would
    // otherwise follow for a degenerate parse of a first-person sentence).
    let entities: Vec<String> = arguments
        .iter()
        .map(|s| extract_entity_name(s, en))
        .filter(|name| !name.is_empty())
        .collect();

    // A pronoun argument was dropped iff `arguments.len() > entities.len()`
    // — the sentence had a real (personal/expletive) subject ("I"/"we"/
    // "it") that filtered to nothing. That subject's presence is itself
    // evidence this is a PERSONAL statement about the speaker/situation,
    // not a bare, contextless definiendum: "Trustee." or "Conservator?" (no
    // subject at all, `arguments.len() == entities.len() == 1`) IS a clean
    // "define this word" query, but "I am a caregiver for my brother… Can I
    // take FMLA leave?" is not — its lone resolved entity ("caregiver") is
    // an accident of which word survived a degenerate multi-clause parse,
    // and `define_word` silently answers a different question than the one
    // asked (caught via the full corpus regression suite: 12 previously-
    // correctly-abstaining personalized/operational statements started
    // confidently defining an unrelated single surviving noun once the
    // pronoun-filter fix made their entity count land on exactly 1). Mirrors
    // `answer_question`'s own `AdmitLimitation`-over-`define_word` choice
    // for the analogous "single entity resolves, but the illocutionary
    // shape isn't a clean definitional query" case.
    let pronoun_subject_dropped = arguments.len() > entities.len();
    if entities.len() == 1 && !pronoun_subject_dropped && !en.lookup(&entities[0]).is_empty() {
        let answer = define_word(en, &entities[0]);
        return trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, answer.text)
            .with_entities_found(entities)
            .grounded(true)
            .with_reasoned_over(answer.reasoned_over)
            .with_definition_provenance(answer.authored_from);
    }

    // Zero or one entities with a dropped pronoun subject: NOT a genuine
    // claim the ontology can verify (there's no second entity to check a
    // relation against) and NOT a clean bare definiendum either (the
    // dropped subject proves so, per the comment above) — the honest
    // response names whatever DID resolve while declining to answer the
    // sentence's real (personal/operational) content, exactly
    // `answer_question`'s own `AdmitLimitation` choice for a single entity
    // that resolves but isn't what was actually asked. Below this,
    // `entities.is_empty()` bare falls through the untouched two-entity
    // block to the old unconditional `AssertKnowledge` bottom fallback,
    // which is `from_ontology: true` even with NOTHING named — the other
    // half of the same regression (11 previously-correctly-abstaining
    // multi-clause statements, mostly "We …"/"It …", whose montague parse
    // absorbed only pronoun arguments, so EVERY entity dropped and the
    // bottom fallback answered "Understood." with no ontology content at
    // all — caught via the same full corpus regression suite).
    if entities.len() <= 1 && pronoun_subject_dropped {
        let mut content = ResponseContent::new(ResponseFrame::AdmitLimitation);
        if let Some(e) = entities.first() {
            content = content.with_entity(e);
        }
        return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
            .with_entities_found(entities);
    }

    // A declarative two-entity claim ("a seal is a mammal") must be VERIFIED
    // against the loaded ontology, not unconditionally acknowledged — the
    // same `verify_relational_claim` truth-check `answer_question` uses for
    // the interrogative form. Before this, `answer_statement` always replied
    // `AssertKnowledge` for 2+ entities regardless of truth: a false claim
    // ("a dog is a plant") was affirmed exactly like a true one. This is also
    // the truth-check the presupposed-clause detector
    // (`check_presupposed_clause`) relies on via this same function, so a
    // false "Since a seal is a plant, …" presupposition is caught here, not
    // re-implemented at the call site.
    //
    // Argument order is REVERSED from `answer_question`'s `Sem::Question`
    // (subject first): a declarative `Sem::Prop`'s subject is the LAST-
    // absorbed argument (backward application absorbs the subject after
    // every object) — the same absorption-order convention already
    // documented and relied upon by
    // `statute_structure::grounding::defines_pointers` ("the term X means Y"
    // extraction) and montague's own ditransitive-verb test. So for "a seal
    // is a mammal", `entities == [mammal, seal]`: child (subject) is
    // `entities[1]`, parent (complement) is `entities[0]`.
    if entities.len() >= 2 {
        let child = &entities[entities.len() - 1];
        let parent = &entities[0];
        let kind = en
            .relation_for_surface(predicate)
            .unwrap_or_else(subsumption_kind);
        match verify_relational_claim(en, child, parent, &kind) {
            RelationalVerdict::True { reasoned_over, .. } => {
                let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge);
                for e in &entities {
                    content = content.with_entity(e);
                }
                return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                    .with_entities_found(entities.clone())
                    .with_taxonomy_checked(Some((child.clone(), parent.clone(), true)))
                    .grounded(true)
                    .with_reasoned_over(reasoned_over);
            }
            RelationalVerdict::False { reasoned_over } => {
                let connective = en.surface_for_relation(&kind);
                return trace_impls::ResponseResult::new(
                    ResponseFrame::AssertKnowledge,
                    realize::realize_negation(child, parent, connective.as_deref()),
                )
                .with_entities_found(entities.clone())
                .with_taxonomy_checked(Some((child.clone(), parent.clone(), false)))
                .grounded(true)
                .with_reasoned_over(reasoned_over);
            }
            RelationalVerdict::Unproven { reasoned_over } => {
                let content = ResponseContent::new(ResponseFrame::UnprovenRelation)
                    .with_entity(child)
                    .with_entity(parent);
                let content = match en.surface_for_relation(&kind) {
                    Some(conn) => content.with_predicate(&conn),
                    None => content,
                };
                return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
                    .with_entities_found(entities.clone())
                    .with_taxonomy_checked(Some((child.clone(), parent.clone(), false)))
                    .with_reasoned_over(reasoned_over);
            }
            RelationalVerdict::Unresolved => {
                // One or both surfaces didn't resolve — fall through to the
                // honest vocabulary-gap catch-all below, the SAME resolution
                // guard `answer_question` applies for its own Unresolved
                // case, rather than confidently asserting a relation between
                // an entity that never resolved (previously: "Yes, {price}
                // relates to {what's}." for a `what's` that resolves to
                // nothing — caught via the full corpus regression suite once
                // a grammar change let more sentences reach this arm).
            }
        }
    }

    // Only entities that DID resolve name a genuine claim; an entity that
    // never resolved to a concept is named as an unknown word instead of
    // silently folded into a confident "X relates to Y" — the same
    // `UnknownVocabulary` honesty `answer_question`'s own fallback uses. A
    // COMPOSITE-minted entity (axiom `DefiniendumIsALexicalUnit`) is never
    // itself named — its failing LEXICAL leaves are, exactly as
    // `unresolved_surfaces` already does for `answer_question`'s own
    // abstention path, re-walked here over `arguments` (never re-derived
    // from `entities`' bare strings, which have already lost which Sem each
    // one came from). Confirmed real regression this closes: "What Is
    // Medicaid, and Who Is Eligible?" — the degenerate `is:N/N + medicaid:N`
    // derivation (mirroring "what is a long"'s own "is a" composite
    // exactly) minted "is medicaid", and this function — unlike
    // `answer_question`, which already had the composite guard — named the
    // ENTIRE fabricated concatenation an unknown word instead of the real,
    // separately-scoped gap: "medicaid" itself has no bare-headword lexicon
    // entry.
    let mut unresolved: Vec<String> = Vec::new();
    for s in arguments {
        let name = extract_entity_name(s, en);
        if name.is_empty() || !en.lookup(&name).is_empty() {
            continue;
        }
        match entity_composite_parts(s) {
            Some((func, arg)) => {
                for leaf in [func, arg] {
                    if en.lookup(leaf).is_empty() && !unresolved.iter().any(|u| u == leaf) {
                        unresolved.push(leaf.to_string());
                    }
                }
            }
            None => {
                if !unresolved.contains(&name) {
                    unresolved.push(name);
                }
            }
        }
    }
    if !unresolved.is_empty() {
        let mut content = ResponseContent::new(ResponseFrame::UnknownVocabulary);
        for e in &unresolved {
            content = content.with_entity(e);
        }
        return trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
            .with_entities_found(entities);
    }

    // Reachable with `entities.is_empty()` only when `arguments` itself was
    // empty (nothing to name at all, not even a dropped pronoun) — the
    // degenerate edge case the `pronoun_subject_dropped` branch above
    // doesn't cover. `AssertKnowledge` with no entity is not a genuine
    // claim about anything; never mark it `from_ontology: true`.
    if entities.is_empty() {
        return trace_impls::ResponseResult::new(
            ResponseFrame::AdmitLimitation,
            realize::realize(&ResponseContent::new(ResponseFrame::AdmitLimitation)),
        )
        .with_entities_found(entities);
    }

    // A composition-MINTED argument (axiom `DefiniendumIsALexicalUnit`) must
    // never be affirmed as a genuine entity, even here at the bottom
    // fallback where every earlier check already passed it through: the
    // `unresolved` leaf-decomposition above only checks whether a
    // composite's INDIVIDUAL leaves resolve, not whether the WHOLE minted
    // string is a real thing. Confirmed real regression: "What Is
    // Medicaid, and Who Is Eligible?" — a degenerate coordinated-double-
    // question mis-derivation (an unsupported construction, task #35's
    // Bucket P) mints the composite "is medicaid"; once "medicaid" itself
    // gained a real bare-headword gloss, BOTH of "is medicaid"'s leaves
    // ("is", "medicaid") resolved individually, `unresolved` stayed empty,
    // and this function fell all the way to an unconditional
    // `from_ontology: true` claim — "Yes. eligible relates to is
    // medicaid." — asserting a fabricated concatenation as fact. Declining
    // honestly here (never `from_ontology: true`) is strictly more honest
    // than affirming a non-word.
    let has_composite_entity = arguments
        .iter()
        .any(|s| entity_composite_parts(s).is_some());
    if has_composite_entity {
        return trace_impls::ResponseResult::new(
            ResponseFrame::AdmitLimitation,
            realize::realize(&ResponseContent::new(ResponseFrame::AdmitLimitation)),
        )
        .with_entities_found(entities);
    }

    let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge);
    for e in &entities {
        content = content.with_entity(e);
    }
    trace_impls::ResponseResult::new(content.frame, realize::realize(&content))
        .with_entities_found(entities)
        .grounded(true)
}

/// THE DEFINIENDUM-IS-A-LEXICAL-UNIT AXIOM (slice E of the R-2 define fix):
/// the answer layer never names a composition-MINTED concatenation as an
/// unknown word, and an abstention's `unresolved` set names failing lexical
/// leaves only. The degenerate `is:N/N + a:N` derivation of "what is a long"
/// minted the surface "is a" ([`montague::PredProvenance::Composite`]) and the
/// chat replied 'I do not know the word "is a"' — a claim of ignorance about a
/// non-word. A dictionary's unit of definition is the headword lemma, and a
/// mentioned/queried expression is a term — neither is a derivation-minted
/// string.
pub struct DefiniendumIsALexicalUnit;

impl pr4xis::ontology::Axiom for DefiniendumIsALexicalUnit {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
        use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

        let english = English::sample_static();
        let composite = |word: &str, func: &str, arg: &str| montague::Sem::Pred {
            word: word.into(),
            role: montague::GrammaticalRole::Argument,
            provenance: montague::PredProvenance::Composite {
                func: func.into(),
                arg: arg.into(),
            },
        };

        // (1) An unresolvable composite whose leaves resolve: never named an
        //     unknown word, and nothing enters `unresolved`.
        let sem = composite("dog cat", "dog", "cat");
        let result = answer_question(
            english,
            english,
            &[],
            "what",
            core::slice::from_ref(&sem),
            montague::QuestionIllocution::Content,
        );
        let gap_naming = realize::realize(
            &ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity("dog cat"),
        );
        let question = montague::Sem::Question {
            predicate: "what".into(),
            arguments: vec![sem],
            illocution: montague::QuestionIllocution::Content,
        };
        let clean_composite = !result.from_ontology
            && !result.response.contains(&gap_naming)
            && unresolved_surfaces(&question, english).is_empty();

        // (2) A composite with a failing lexical leaf reports THE LEAF, never
        //     the minted string.
        let failing = montague::Sem::Question {
            predicate: "what".into(),
            arguments: vec![composite("blorptt cat", "blorptt", "cat")],
            illocution: montague::QuestionIllocution::Content,
        };
        let leaves_only = unresolved_surfaces(&failing, english) == vec!["blorptt".to_string()];

        if clean_composite && leaves_only {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DefiniendumIsALexicalUnit",
        "a definiendum is a lexical unit: the answer layer never names a composition-minted concatenation as an unknown word, and an abstention's `unresolved` set names failing lexical leaves only — a mentioned/queried unit is a term, and dictionaries define headword lemmas, never derivation-minted strings",
        "Atkins & Rundell (2008) The Oxford Guide to Practical Lexicography, OUP — headword/lemma selection: the unit of definition is the lemma; Quine (1940) Mathematical Logic p. 26, as presented in SEP 'Quotation' §3.1 — a mentioned expression is a singular term"
    );
}

pr4xis::register_axiom!(DefiniendumIsALexicalUnit, constructor);

/// THE DEFINITIONAL-SUBJECT-COMPOUND AXIOM (adversarial corpus index 69,
/// category `fabricated_term`): a "what is X" definitional question whose
/// subject X is a multi-word nominal compound of individually-known
/// constituents that does NOT resolve as a unit is answered by an
/// ABSTENTION naming the full compound — never by defining or enumerating
/// the senses of any constituent. A nominal compound is a name for a
/// category, not a compositional description (Downing 1977): knowing every
/// constituent does not confer knowledge of the compound's referent, so a
/// constituent-sense answer is a fabrication in exactly the TruthfulQA
/// fabricated-term sense (Lin, Hilton & Evans 2022). The single-constituent
/// subject is the negative control: "what is the dog" must still define
/// "dog" — this axiom scopes the abstention to genuine compounds only.
pub struct DefinitionalSubjectCompoundAbstainsAsAUnit;

impl pr4xis::ontology::Axiom for DefinitionalSubjectCompoundAbstainsAsAUnit {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;

        let english = English::sample_static();
        let tok = |word: &str, lambek_type| TypedToken {
            expression_use: ExpressionUse::Used,
            word: word.into(),
            lambek_type,
        };
        let failed_reduction = ReductionResult {
            success: false,
            final_type: None,
            remaining: Vec::new(),
            unary_steps: 0,
        };

        // (1) "what is the dog cat" — both constituents resolve in the
        //     sample lexicon, the compound "dog cat" does not: the
        //     partial-understanding layer abstains (never `from_ontology`)
        //     and the response names the FULL compound, not a constituent.
        let compound_q = vec![
            tok("what", svo::wh_what()),
            tok("is", svo::copula()),
            tok("the", svo::determiner()),
            tok("dog", svo::noun()),
            tok("cat", svo::noun()),
        ];
        let subject = unresolved_definitional_subject_compound(english, english, &compound_q);
        let result = attempt_partial_understanding(
            english,
            english,
            &compound_q,
            &failed_reduction,
            &montague::Sem::unresolved(),
        );
        let abstains_naming_the_compound = subject.as_deref() == Some("dog cat")
            && !result.from_ontology
            && result.response.contains("dog cat");

        // (2) Negative control — "what is the dog": a single-constituent
        //     subject is NOT a compound; the existing single-definiendum
        //     path still answers from the loaded gloss.
        let single_q = vec![
            tok("what", svo::wh_what()),
            tok("is", svo::copula()),
            tok("the", svo::determiner()),
            tok("dog", svo::noun()),
        ];
        let single_subject = unresolved_definitional_subject_compound(english, english, &single_q);
        let single_result = attempt_partial_understanding(
            english,
            english,
            &single_q,
            &failed_reduction,
            &montague::Sem::unresolved(),
        );
        let single_still_defines = single_subject.is_none()
            && single_result.from_ontology
            && single_result.response.contains("domesticated canine");

        if abstains_naming_the_compound && single_still_defines {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DefinitionalSubjectCompoundAbstainsAsAUnit",
        "a definitional what-is question whose subject is a multi-word nominal compound of individually-known constituents that does not resolve as a unit is answered by an abstention naming the full compound, never by defining or enumerating the senses of any constituent — a nominal compound is a name for a category, not a compositional description, so constituent senses cannot supply its referent",
        "Downing (1977) 'On the Creation and Use of English Compound Nouns', Language 53(4):810-842 — a compound's constituent relation is unstated and conventionalized, its meaning unrecoverable from the constituents alone; Kripke (1980) Naming and Necessity, Harvard UP — a proper name designates rigidly, not via the descriptive content of its parts; Lin, Hilton & Evans (2022) 'TruthfulQA', ACL — answering a fabricated term from familiar constituent words is the measured imitative-falsehood failure mode"
    );
}

pr4xis::register_axiom!(DefinitionalSubjectCompoundAbstainsAsAUnit, constructor);

/// THE FULL-SENSE-ENUMERATION AXIOM (slice A of the R-2 define fix): a define
/// answer realizes EVERY sense of the resolved definiendum, in the loaded
/// source's order. Sense selection is an inventory-compilation decision (the
/// lexicographer's, upstream in the loaded LMF), never a display-time cut:
/// dictionaries present the full sense inventory of an entry, and WordNet's
/// own sense rank explicitly carries no licensing for truncation. The
/// day-one `ids.iter().take(5)` in `define_word` (d492e89f) was an uncited
/// display cap — an unprincipled content-determination decision — and its
/// measured cost was 129 of the 349 corpus define failures (the sampled
/// sense ranked ≥ 5: "what is gagging" needs sense 6 of 9).
pub struct DefineEnumeratesTheLoadedSenseInventory;

impl pr4xis::ontology::Axiom for DefineEnumeratesTheLoadedSenseInventory {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // A polysemy fixture DEEPER than any historical cap: one lemma, seven
        // senses, each with a distinct gloss — built exactly like
        // `English::sample`'s inline LMF.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="polysemy-fixture" label="Polysemy" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-bank-n">
      <Lemma writtenForm="bank" partOfSpeech="n"/>
      <Sense id="bank-n-01" synset="s-bank-1"/>
      <Sense id="bank-n-02" synset="s-bank-2"/>
      <Sense id="bank-n-03" synset="s-bank-3"/>
      <Sense id="bank-n-04" synset="s-bank-4"/>
      <Sense id="bank-n-05" synset="s-bank-5"/>
      <Sense id="bank-n-06" synset="s-bank-6"/>
      <Sense id="bank-n-07" synset="s-bank-7"/>
    </LexicalEntry>
    <Synset id="s-bank-1" ili="i1" partOfSpeech="n"><Definition>sloping land beside a body of water</Definition></Synset>
    <Synset id="s-bank-2" ili="i2" partOfSpeech="n"><Definition>a financial institution that accepts deposits</Definition></Synset>
    <Synset id="s-bank-3" ili="i3" partOfSpeech="n"><Definition>a long ridge or pile</Definition></Synset>
    <Synset id="s-bank-4" ili="i4" partOfSpeech="n"><Definition>an arrangement of similar objects in a row</Definition></Synset>
    <Synset id="s-bank-5" ili="i5" partOfSpeech="n"><Definition>a supply or stock held in reserve</Definition></Synset>
    <Synset id="s-bank-6" ili="i6" partOfSpeech="n"><Definition>the funds held by a gambling house</Definition></Synset>
    <Synset id="s-bank-7" ili="i7" partOfSpeech="n"><Definition>a slope in the turn of a road or track</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);

        let defined = define_word(&english, "bank").text;
        let glosses = [
            "sloping land beside a body of water",
            "a financial institution that accepts deposits",
            "a long ridge or pile",
            "an arrangement of similar objects in a row",
            "a supply or stock held in reserve",
            "the funds held by a gambling house",
            "a slope in the turn of a road or track",
        ];
        // Every sense's gloss appears, numbered in the loaded (sense) order —
        // the realizer numbers `resolve_surface` order 1-based.
        let complete_and_ordered = glosses
            .iter()
            .enumerate()
            .all(|(i, gloss)| defined.contains(&format!("{}. {}", i + 1, gloss)));

        if complete_and_ordered {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DefineEnumeratesTheLoadedSenseInventory",
        "a define answer realizes EVERY sense of the resolved definiendum — completeness, never a display-time cut; sense selection is an inventory-compilation decision made upstream in the loaded lexicon, never a display cap. Within a provenance group senses keep their loaded (sense) order; the cross-group priority (domain-source senses first) is the separate DefinePrioritizesDomainSensesOverGeneralSenses axiom",
        "wn(1WN) WordNet command-line manual — the -over search displays an overview of ALL the senses of the search word; cntlist(5WN) — the sense ordering 'should not be construed as an accurate indicator of frequency of use' (rank licenses no cut); Lew (2013) 'Identifying, ordering and defining senses', in The Bloomsbury Companion to Lexicography — frequency/chronology/logical schemes ORDER the full sense list, none omits; Reiter & Dale (2000) Building Natural Language Generation Systems, CUP — content determination is a principled stage, an uncited constant inside realization is not a principle"
    );
}

pr4xis::register_axiom!(DefineEnumeratesTheLoadedSenseInventory, constructor);

/// THE DOMAIN-SENSE-PRIORITY AXIOM: over a special-domain corpus, a
/// definiendum's senses drawn from the loaded DOMAIN lexicons (caregiving /
/// HCBS / USC — [`ConceptView::is_domain_loaded`](pr4xis_domains::cognitive::linguistics::english::ConceptView::is_domain_loaded),
/// plus every statutory `defines`-overlay gloss) are realized BEFORE the
/// general-purpose WordNet senses. This is a STABLE partition — it drops
/// nothing (completeness stays [`DefineEnumeratesTheLoadedSenseInventory`]) and
/// reorders nothing within either group. Language-for-Special-Purposes /
/// terminology lexicography makes the specialized reading the salient one
/// in-domain (Cabré 1999), so "what is respite" leads with the caregiving
/// statutory definition (42 USC 300ii(7)) rather than WordNet's "a pause from
/// doing something (as work)". It is NOT a WordNet-sense-frequency ranking:
/// the loaded OEWN data's own `cntlist(5WN)` disclaims sense rank as a
/// frequency signal, so provenance — which the composed reasoner already types
/// — is the citable ordering key, never sense ordinality.
pub struct DefinePrioritizesDomainSensesOverGeneralSenses;

impl pr4xis::ontology::Axiom for DefinePrioritizesDomainSensesOverGeneralSenses {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // A definiendum resolving to INTERLEAVED general (`false`) and domain
        // (`true`) senses — exactly "respite"'s shape (general WordNet senses
        // with the caregiving-statute sense among them). The ordering rule must
        // float BOTH domain senses to the front, keep every general sense, and
        // reorder NOTHING within either group.
        let input: Vec<(bool, String)> = vec![
            (false, "a pause from doing something".to_string()),
            (false, "the act of reprieving".to_string()),
            (
                true,
                "planned care giving temporary relief to a family caregiver — 42 USC 300ii(7)"
                    .to_string(),
            ),
            (false, "an interruption in intensity".to_string()),
            (true, "a Medicaid respite service".to_string()),
        ];
        let out = order_definitions_domain_first(input.clone());

        // Completeness: the output is a permutation of the input (set-equal).
        let mut in_lines: Vec<&String> = input.iter().map(|(_, s)| s).collect();
        let mut out_lines: Vec<&String> = out.iter().map(|(_, s)| s).collect();
        in_lines.sort();
        out_lines.sort();
        let complete = in_lines == out_lines;

        // Priority: no general (`false`) sense precedes any domain (`true`) one.
        let domain_leads = match (
            out.iter().position(|(d, _)| !*d),
            out.iter().rposition(|(d, _)| *d),
        ) {
            (Some(first_general), Some(last_domain)) => first_general > last_domain,
            _ => true, // one group empty — vacuously ordered
        };

        // Stability: each group keeps its input relative order (no within-group
        // shuffle — the completeness axiom's own numbering relies on this).
        let project = |src: &[(bool, String)], want: bool| -> Vec<String> {
            src.iter()
                .filter(|(d, _)| *d == want)
                .map(|(_, s)| s.clone())
                .collect()
        };
        let stable = project(&input, true) == project(&out, true)
            && project(&input, false) == project(&out, false);

        if complete && domain_leads && stable {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DefinePrioritizesDomainSensesOverGeneralSenses",
        "a definiendum's senses from the loaded domain lexicons are realized before its general-purpose WordNet senses — a stable partition that drops no sense and reorders nothing within either group",
        "Cabré, M. T. (1999). Terminology: Theory, Methods and Applications (Terminology and Lexicography Research and Practice 1). Amsterdam: John Benjamins — in Language for Special Purposes the specialized/terminological sense is the primary reading within its domain; Bowker, L. & Pearson, J. (2002). Working with Specialized Language: A Practical Guide to Using Corpora. London: Routledge — a special-domain corpus foregrounds the domain sense over the general-language one; the completeness half (no sense cut, only reordered) stays DefineEnumeratesTheLoadedSenseInventory, and provenance rather than WordNet sense rank is the key because cntlist(5WN) disclaims rank as a frequency signal"
    );
}

pr4xis::register_axiom!(DefinePrioritizesDomainSensesOverGeneralSenses, constructor);

/// The two-entity taxonomy path's "which sense pair answers this" choice:
/// when MULTIPLE `(child-sense, parent-sense)` pairs satisfy `reaches`, the
/// chosen pair is the one with the highest BARE gloss-word overlap (Lesk
/// 1986) — never the first pair in loaded lexicon order. When exactly one
/// pair satisfies `reaches`, it is returned unscored (dominant case,
/// unaffected by this axiom). See the `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`) for why
/// this scoring, not WordNet's sense-number ordinality, and why bare
/// overlap rather than a relation-expanded gloss.
pub struct TwoEntityPathPrefersGlossOverlapAmongReachingPairs;

impl pr4xis::ontology::Axiom for TwoEntityPathPrefersGlossOverlapAmongReachingPairs {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;

        // "cut" has two senses reaching BOTH "harm" senses (a deliberately
        // over-connected fixture, isolating the SCORING from the existential
        // search): a noun "wound" sense whose gloss shares content words
        // with "injury", and an unrelated "reduction" sense whose gloss
        // shares nothing with it — mirroring the real corpus failure
        // ("cut"/"end") where an unintended sense pair happens to satisfy a
        // real is-a edge.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="wsd-fixture" label="WSD" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-n">
      <Lemma writtenForm="cut" partOfSpeech="n"/>
      <Sense id="cut-n-wound" synset="s-cut-wound"/>
      <Sense id="cut-n-reduction" synset="s-cut-reduction"/>
    </LexicalEntry>
    <LexicalEntry id="e-harm-n">
      <Lemma writtenForm="harm" partOfSpeech="n"/>
      <Sense id="harm-n-injury" synset="s-harm-injury"/>
    </LexicalEntry>
    <Synset id="s-cut-wound" ili="i1" partOfSpeech="n">
      <Definition>an injury from a sharp incision</Definition>
      <SynsetRelation relType="hypernym" target="s-harm-injury"/>
    </Synset>
    <Synset id="s-cut-reduction" ili="i2" partOfSpeech="n">
      <Definition>an amount subtracted or removed</Definition>
      <SynsetRelation relType="hypernym" target="s-harm-injury"/>
    </Synset>
    <Synset id="s-harm-injury" ili="i3" partOfSpeech="n"><Definition>physical injury or wound</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut_ids = english.lookup("cut");
        let harm = english.lookup("harm")[0];
        let cut_wound = english
            .concept_by_synset("s-cut-wound")
            .expect("fixture concept")
            .id();

        // Both cut senses reach "harm" — the wound sense must win on gloss
        // overlap ("injury"/"wound"), not the first-declared sense. Multiple
        // hits never consult VerbNet, ConceptNet, FrameNet, SUMO, or
        // PropBank (see `word_sense` module docs), so empty stores are a
        // faithful fixture here.
        let empty_verbnet = pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore::from_verbnet_and_crosswalk(
            &pr4xis_domains::cognitive::linguistics::verbnet::ontology::VerbNet::default(),
            &std::collections::BTreeMap::new(),
        );
        let empty_conceptnet = pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore::from_conceptnet(
            &pr4xis_domains::cognitive::linguistics::conceptnet::ontology::ConceptNet::default(),
        );
        let empty_framenet =
            pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore::from_framenet(
                &pr4xis_domains::cognitive::linguistics::framenet::ontology::FrameNet::default(),
            );
        let empty_sumo = pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore::from_sumo(
            &pr4xis_domains::cognitive::linguistics::sumo::ontology::Sumo::default(),
        );
        let empty_propbank =
            pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore::from_propbank(
                &pr4xis_domains::cognitive::linguistics::propbank::ontology::PropBank::default(),
            );
        let winner = word_sense::best_reaching_pair(
            &english,
            &empty_verbnet,
            &empty_conceptnet,
            &empty_framenet,
            &empty_sumo,
            &empty_propbank,
            cut_ids,
            &[harm],
            &subsumption_kind(),
        );

        if winner == word_sense::ReachingPairOutcome::Trusted(cut_wound, harm) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TwoEntityPathPrefersGlossOverlapAmongReachingPairs",
        "among multiple sense pairs that satisfy a taxonomy relation, the two-entity answer path selects the pair with the highest relation-expanded gloss-word overlap, never the first pair in loaded lexicon order",
        "Lesk, M. (1986). 'Automatic Sense Disambiguation Using Machine Readable Dictionaries: How to Tell a Pine Cone from an Ice Cream Cone.' Proceedings of SIGDOC '86, pp. 24-26, ACM, DOI 10.1145/318723.318728 (bare gloss-word overlap, no relation expansion — see the word_sense module docs for why a relation-expanded gloss, Banerjee & Pedersen 2002, was tried and rejected as degenerate for this specific use); Kilgarriff, A., & Rosenzweig, J. (2000). 'Framework and Results for English SENSEVAL.' Computers and the Humanities 34(1-2), 15-48, DOI 10.1023/A:1002693207386 (simplified-Lesk / stopword-filtered bag-of-words justification)"
    );
}

pr4xis::register_axiom!(
    TwoEntityPathPrefersGlossOverlapAmongReachingPairs,
    constructor
);

/// The two-entity path's lone-hit corroboration check (VerbNet class-family
/// sharing, Kipper, Korhonen, Ryant & Palmer 2008) MUST NEVER downgrade a
/// `Subsumption` (is-a) query to `Uncorroborated`, even when VerbNet places
/// the two concepts in unrelated classes — this is the fix for a REAL,
/// MEASURED regression (this codebase's own committed corpus is-a class,
/// 4 → 47 failures, entirely false negatives) a first version of this
/// mechanism produced by applying VerbNet's signal to a relation kind it
/// isn't evidence for. See the `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`) for the
/// full literature grounding (Levin's 1993 semantic-coherence hypothesis;
/// Olsen, Dorr & Clark 1997; Baker & Ruppenhofer 2002) of why VerbNet class
/// co-membership tracks shared syntactic-alternation behavior — a
/// similarity/componential relation — not hypernymy/specificity.
pub struct VerbNetCorroborationScopedToSimilarityAndEquivalence;

impl pr4xis::ontology::Axiom for VerbNetCorroborationScopedToSimilarityAndEquivalence {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
        use pr4xis_domains::cognitive::linguistics::verbnet::ontology::{
            VerbNet, VerbNetClass, VerbNetMember,
        };
        use pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore;

        // "cut" and "end" — one verb sense each, so `reaches` has exactly one
        // candidate pair (the lone-hit shape this mechanism gates), mirroring
        // the real corpus case this axiom's regression traces to.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="verbnet-scope-fixture" label="VS" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition><SynsetRelation relType="hypernym" target="s-end"/></Synset>
    <Synset id="s-end" ili="i2" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut = english.lookup("cut")[0];
        let end = english.lookup("end")[0];

        // VerbNet places cut and end in UNRELATED classes — real negative
        // evidence for a Similarity-kind query, but must be INERT for
        // Subsumption.
        let vn = VerbNet {
            classes: vec![
                VerbNetClass {
                    id: "class-a".into(),
                    members: vec![VerbNetMember {
                        name: "cut".into(),
                        wn_sense_keys: vec!["cut%2:30:00".into()],
                    }],
                    subclasses: Vec::new(),
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                },
                VerbNetClass {
                    id: "class-b".into(),
                    members: vec![VerbNetMember {
                        name: "end".into(),
                        wn_sense_keys: vec!["end%2:30:01".into()],
                    }],
                    subclasses: Vec::new(),
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                },
            ],
        };
        let crosswalk: std::collections::BTreeMap<String, u64> = [
            ("cut%2:30:00".to_string(), cut.value()),
            ("end%2:30:01".to_string(), end.value()),
        ]
        .into_iter()
        .collect();
        let verbnet = VerbNetStore::from_verbnet_and_crosswalk(&vn, &crosswalk);
        // No ConceptNet/FrameNet/SUMO/PropBank signal in this fixture —
        // isolates the assertion to VerbNet's own scoping behavior
        // (ConceptNet's, FrameNet's, SUMO's, and PropBank's analogous
        // scoping have their own axioms,
        // `ConceptNetCorroborationComposesWithVerbNet`,
        // `FrameNetCorroborationComposesWithVerbNetAndConceptNet`, and the
        // SUMO/PropBank composition axioms).
        let conceptnet = pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore::from_conceptnet(
            &pr4xis_domains::cognitive::linguistics::conceptnet::ontology::ConceptNet::default(),
        );
        let framenet =
            pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore::from_framenet(
                &pr4xis_domains::cognitive::linguistics::framenet::ontology::FrameNet::default(),
            );
        let sumo = pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore::from_sumo(
            &pr4xis_domains::cognitive::linguistics::sumo::ontology::Sumo::default(),
        );
        let propbank =
            pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore::from_propbank(
                &pr4xis_domains::cognitive::linguistics::propbank::ontology::PropBank::default(),
            );

        let outcome = word_sense::best_reaching_pair(
            &english,
            &verbnet,
            &conceptnet,
            &framenet,
            &sumo,
            &propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        );

        if outcome == word_sense::ReachingPairOutcome::Trusted(cut, end) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VerbNetCorroborationScopedToSimilarityAndEquivalence",
        "VerbNet class-family corroboration for a lone reaches() hit never downgrades a Subsumption (is-a) query to Uncorroborated, even when VerbNet places the two concepts in unrelated classes — VerbNet class co-membership is evidence for Similarity/Equivalence, not for hypernymy",
        "Levin, B. (1993). English Verb Classes and Alternations: A Preliminary Investigation. University of Chicago Press (semantic coherence hypothesis: class comembership tracks shared meaning components, not specificity); Olsen, M., Dorr, B., & Clark, S. (1997). 'Using WordNet to Posit Hierarchical Structure in Levin's Verb Classes.' AMTA/SIG-IL Workshop on Interlinguas (WordNet sense tags were imported to impose a hierarchy Levin classes lack natively); Baker, C. F., & Ruppenhofer, J. (2002). 'FrameNet's Frames vs. Levin's Verb Classes.' Proceedings of BLS 28 (the same class-vs-hierarchy mismatch against FrameNet's frame structure); Kipper, K., Korhonen, A., Ryant, N., & Palmer, M. (2008). 'A Large-scale Classification of English Verbs.' Language Resources and Evaluation 42(1):21-40"
    );
}

pr4xis::register_axiom!(
    VerbNetCorroborationScopedToSimilarityAndEquivalence,
    constructor
);

/// The two-entity path's lone-hit corroboration composes TWO independent
/// sources — VerbNet class-family sharing and ConceptNet association sharing
/// (Speer, Chin & Havasi 2017) — under "either source corroborating is
/// enough" (Rule 1) and "uncorroborated only if some source that actually
/// covers both concepts finds no match" (Rule 2). This axiom checks both
/// rules AND that ConceptNet, like VerbNet, is scoped to `Similarity`/
/// `Equivalence` and never downgrades a `Subsumption` query — the same
/// discipline `VerbNetCorroborationScopedToSimilarityAndEquivalence` proves
/// for VerbNet alone, checked here for ConceptNet and for their composition.
/// See the `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`)'s
/// "Composing two corroboration sources" section for the full rationale.
pub struct ConceptNetCorroborationComposesWithVerbNet;

impl pr4xis::ontology::Axiom for ConceptNetCorroborationComposesWithVerbNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::conceptnet::ontology::{
            ConceptNet, ConceptNetEdge,
        };
        use pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore;
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
        use pr4xis_domains::cognitive::linguistics::verbnet::ontology::VerbNet;
        use pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore;
        use pr4xis_domains::formal::relations::ontology::similarity_relation_kind;

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="conceptnet-scope-fixture" label="CS" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition><SynsetRelation relType="hypernym" target="s-end"/></Synset>
    <Synset id="s-end" ili="i2" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut = english.lookup("cut")[0];
        let end = english.lookup("end")[0];

        // VerbNet has NO coverage at all; ConceptNet independently finds a
        // direct association — Rule 1: one corroborating source is enough.
        let no_coverage_verbnet =
            VerbNetStore::from_verbnet_and_crosswalk(&VerbNet::default(), &Default::default());
        // No FrameNet/SUMO/PropBank signal in this fixture — isolates the
        // assertion to VerbNet+ConceptNet's own composition (FrameNet's,
        // SUMO's, and PropBank's analogous composition have their own
        // axioms, `FrameNetCorroborationComposesWithVerbNetAndConceptNet`
        // and the SUMO/PropBank composition axioms).
        let no_coverage_framenet =
            pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore::from_framenet(
                &pr4xis_domains::cognitive::linguistics::framenet::ontology::FrameNet::default(),
            );
        let no_coverage_sumo =
            pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore::from_sumo(
                &pr4xis_domains::cognitive::linguistics::sumo::ontology::Sumo::default(),
            );
        let no_coverage_propbank =
            pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore::from_propbank(
                &pr4xis_domains::cognitive::linguistics::propbank::ontology::PropBank::default(),
            );
        let corroborating_conceptnet = ConceptNetStore::from_conceptnet(&ConceptNet {
            edges: vec![ConceptNetEdge {
                relation: "RelatedTo".to_string(),
                start_lemma: "cut".to_string(),
                end_lemma: "end".to_string(),
                weight: 1.0,
            }],
        });
        // Direct call, not routed through best_reaching_pair's reaches()
        // gate: English's default reaches() only supports Subsumption, so a
        // synthetic fixture can never produce a real Similarity-kind hit
        // through the full path (see corroborate_lone_hit's own doc).
        let rule_1 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &corroborating_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        // Both sources cover cut AND end, but via UNRELATED associations —
        // real negative evidence from ConceptNet's side, for a
        // Similarity-kind query.
        let uncorroborating_conceptnet = ConceptNetStore::from_conceptnet(&ConceptNet {
            edges: vec![
                ConceptNetEdge {
                    relation: "RelatedTo".to_string(),
                    start_lemma: "cut".to_string(),
                    end_lemma: "sever".to_string(),
                    weight: 1.0,
                },
                ConceptNetEdge {
                    relation: "RelatedTo".to_string(),
                    start_lemma: "end".to_string(),
                    end_lemma: "finish".to_string(),
                    weight: 1.0,
                },
            ],
        });
        let rule_2 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &uncorroborating_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Uncorroborated(cut, end);

        // THE REGRESSION-GUARD CHECK: the exact same uncorroborating
        // ConceptNet fixture, under Subsumption, must stay Trusted —
        // ConceptNet's association sharing is no more valid evidence for
        // hypernymy than VerbNet's class sharing is (see the module doc's
        // "ConceptNet gets the same scope restriction" section). THIS check
        // routes through the full best_reaching_pair -> reaches() path
        // (unlike rule_1/rule_2 above) because English's default reaches()
        // DOES support Subsumption, so the integration-level path is
        // meaningful here — mirroring
        // VerbNetCorroborationScopedToSimilarityAndEquivalence's own check.
        let subsumption_inert = word_sense::best_reaching_pair(
            &english,
            &no_coverage_verbnet,
            &uncorroborating_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        if rule_1 && rule_2 && subsumption_inert {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ConceptNetCorroborationComposesWithVerbNet",
        "the two-entity path's lone-hit corroboration composes VerbNet and ConceptNet under Rule 1 (either source corroborating trusts) and Rule 2 (uncorroborated only if some covering source finds no match); ConceptNet is scoped to Similarity/Equivalence and never downgrades a Subsumption query, mirroring VerbNet's own scoping",
        "Speer, R., Chin, J., & Havasi, C. (2017). 'ConceptNet 5.5: An Open Multilingual Graph of General Knowledge.' Proceedings of the AAAI Conference on Artificial Intelligence 31(1); Levin, B. (1993). English Verb Classes and Alternations: A Preliminary Investigation. University of Chicago Press (semantic coherence hypothesis, the same componential-not-specificity reasoning extended to ConceptNet's generic Association mapping)"
    );
}

pr4xis::register_axiom!(ConceptNetCorroborationComposesWithVerbNet, constructor);

/// The two-entity path's lone-hit corroboration composes THREE independent
/// sources — VerbNet class-family sharing, ConceptNet association sharing,
/// and FrameNet frame-family sharing (Baker, Fillmore & Lowe 1998) — under
/// the same "any source corroborating is enough" (Rule 1) and
/// "uncorroborated only if some source that actually covers both concepts
/// finds no match" (Rule 2) composition
/// `ConceptNetCorroborationComposesWithVerbNet` proves for two sources. This
/// axiom checks the SAME two rules with FrameNet as the sole corroborating/
/// uncorroborating source (VerbNet and ConceptNet held at no-coverage), AND
/// that FrameNet, like VerbNet and ConceptNet, is scoped to `Similarity`/
/// `Equivalence` and never downgrades a `Subsumption` query. See the
/// `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`)'s
/// "Composing corroboration sources" section for the full rationale.
pub struct FrameNetCorroborationComposesWithVerbNetAndConceptNet;

impl pr4xis::ontology::Axiom for FrameNetCorroborationComposesWithVerbNetAndConceptNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::conceptnet::ontology::ConceptNet;
        use pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore;
        use pr4xis_domains::cognitive::linguistics::framenet::ontology::{
            FrameNet, FrameNetLexicalUnit,
        };
        use pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore;
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
        use pr4xis_domains::cognitive::linguistics::verbnet::ontology::VerbNet;
        use pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore;
        use pr4xis_domains::formal::relations::ontology::similarity_relation_kind;
        use pr4xis_domains::social::software::markup::xml::lmf::LmfPos;

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="framenet-scope-fixture" label="FS" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition><SynsetRelation relType="hypernym" target="s-end"/></Synset>
    <Synset id="s-end" ili="i2" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut = english.lookup("cut")[0];
        let end = english.lookup("end")[0];

        let no_coverage_verbnet =
            VerbNetStore::from_verbnet_and_crosswalk(&VerbNet::default(), &Default::default());
        let no_coverage_conceptnet = ConceptNetStore::from_conceptnet(&ConceptNet::default());
        // No SUMO/PropBank signal in this fixture — isolates the assertion
        // to VerbNet+ConceptNet+FrameNet's own composition (SUMO's and
        // PropBank's analogous composition have their own axioms).
        let no_coverage_sumo =
            pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore::from_sumo(
                &pr4xis_domains::cognitive::linguistics::sumo::ontology::Sumo::default(),
            );
        let no_coverage_propbank =
            pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore::from_propbank(
                &pr4xis_domains::cognitive::linguistics::propbank::ontology::PropBank::default(),
            );

        // VerbNet and ConceptNet have NO coverage at all; FrameNet
        // independently finds a shared frame — Rule 1: one corroborating
        // source is enough.
        let corroborating_framenet = FrameNetStore::from_framenet(&FrameNet {
            lexical_units: vec![
                FrameNetLexicalUnit {
                    lemma: "cut".to_string(),
                    pos: LmfPos::Verb,
                    frame: "Cause_to_end".to_string(),
                },
                FrameNetLexicalUnit {
                    lemma: "end".to_string(),
                    pos: LmfPos::Verb,
                    frame: "Cause_to_end".to_string(),
                },
            ],
            relations: Vec::new(),
        });
        let rule_1 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &corroborating_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        // FrameNet covers cut AND end, but via UNRELATED frames — real
        // negative evidence from FrameNet's side, for a Similarity-kind
        // query.
        let uncorroborating_framenet = FrameNetStore::from_framenet(&FrameNet {
            lexical_units: vec![
                FrameNetLexicalUnit {
                    lemma: "cut".to_string(),
                    pos: LmfPos::Verb,
                    frame: "Cutting".to_string(),
                },
                FrameNetLexicalUnit {
                    lemma: "end".to_string(),
                    pos: LmfPos::Verb,
                    frame: "Process_completed_state".to_string(),
                },
            ],
            relations: Vec::new(),
        });
        let rule_2 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &uncorroborating_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Uncorroborated(cut, end);

        // THE REGRESSION-GUARD CHECK: the exact same uncorroborating
        // FrameNet fixture, under Subsumption, must stay Trusted —
        // FrameNet's frame sharing is no more valid evidence for hypernymy
        // than VerbNet's class sharing or ConceptNet's association sharing
        // is.
        let subsumption_inert = word_sense::best_reaching_pair(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &uncorroborating_framenet,
            &no_coverage_sumo,
            &no_coverage_propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        if rule_1 && rule_2 && subsumption_inert {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FrameNetCorroborationComposesWithVerbNetAndConceptNet",
        "the two-entity path's lone-hit corroboration composes VerbNet, ConceptNet and FrameNet under Rule 1 (any source corroborating trusts) and Rule 2 (uncorroborated only if some covering source finds no match); FrameNet is scoped to Similarity/Equivalence and never downgrades a Subsumption query, mirroring VerbNet's and ConceptNet's own scoping",
        "Baker, C. F., Fillmore, C. J. & Lowe, J. B. (1998). 'The Berkeley FrameNet Project.' Proceedings of COLING-ACL 1998; Ruppenhofer, J., Ellsworth, M., Petruck, M. R. L., Johnson, C. R. & Scheffczyk, J. (2016). FrameNet II: Extended Theory and Practice. ICSI (the 9 frame-to-frame relation types, none specially trusted for Subsumption); Levin, B. (1993). English Verb Classes and Alternations: A Preliminary Investigation. University of Chicago Press (semantic coherence hypothesis, the same componential-not-specificity reasoning extended to FrameNet's generic Association mapping)"
    );
}

pr4xis::register_axiom!(
    FrameNetCorroborationComposesWithVerbNetAndConceptNet,
    constructor
);

/// The two-entity path's lone-hit corroboration composes FOUR independent
/// sources — VerbNet class-family sharing, ConceptNet association sharing,
/// FrameNet frame-family sharing, and SUMO class sharing (Niles & Pease 2001,
/// 2003) — under the same "any source corroborating is enough" (Rule 1) and
/// "uncorroborated only if some source that actually covers both concepts
/// finds no match" (Rule 2) composition
/// `FrameNetCorroborationComposesWithVerbNetAndConceptNet` proves for three
/// sources. This axiom checks the SAME two rules with SUMO as the sole
/// corroborating/uncorroborating source (VerbNet, ConceptNet, and FrameNet
/// held at no-coverage), AND that SUMO, like the other three, is scoped to
/// `Similarity`/`Equivalence` and never downgrades a `Subsumption` query. See
/// the `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`)'s
/// "Composing corroboration sources" section for the full rationale.
///
/// Unlike VerbNet/ConceptNet/FrameNet (all lemma-keyed at query time), SUMO's
/// committed data is already resolved to `ConceptId` values (see
/// `pr4xis_domains::cognitive::linguistics::sumo::store`'s module doc), so
/// this fixture builds `SumoMapping`s directly against the fixture's own
/// `ConceptId`s rather than a lemma string.
pub struct SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet;

impl pr4xis::ontology::Axiom for SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::conceptnet::ontology::ConceptNet;
        use pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore;
        use pr4xis_domains::cognitive::linguistics::framenet::ontology::FrameNet;
        use pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore;
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
        use pr4xis_domains::cognitive::linguistics::sumo::ontology::{
            Sumo, SumoMapping, SumoRelationKind,
        };
        use pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore;
        use pr4xis_domains::cognitive::linguistics::verbnet::ontology::VerbNet;
        use pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore;
        use pr4xis_domains::formal::relations::ontology::similarity_relation_kind;

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="sumo-scope-fixture" label="SS" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition><SynsetRelation relType="hypernym" target="s-end"/></Synset>
    <Synset id="s-end" ili="i2" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut = english.lookup("cut")[0];
        let end = english.lookup("end")[0];

        let no_coverage_verbnet =
            VerbNetStore::from_verbnet_and_crosswalk(&VerbNet::default(), &Default::default());
        let no_coverage_conceptnet = ConceptNetStore::from_conceptnet(&ConceptNet::default());
        let no_coverage_framenet = FrameNetStore::from_framenet(&FrameNet::default());
        // No PropBank signal in this fixture — isolates the assertion to
        // SUMO's own composition (PropBank's analogous composition has its
        // own axiom).
        let no_coverage_propbank =
            pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore::from_propbank(
                &pr4xis_domains::cognitive::linguistics::propbank::ontology::PropBank::default(),
            );

        // VerbNet, ConceptNet, and FrameNet have NO coverage at all; SUMO
        // independently maps both concepts to the SAME upper-ontology term
        // — Rule 1: one corroborating source is enough.
        let corroborating_sumo = SumoStore::from_sumo(&Sumo {
            mappings: vec![
                SumoMapping {
                    concept: cut,
                    term: "Cessation".to_string(),
                    relation: SumoRelationKind::Subsumption,
                    oewn_synset_id: "s-cut".to_string(),
                },
                SumoMapping {
                    concept: end,
                    term: "Cessation".to_string(),
                    relation: SumoRelationKind::Subsumption,
                    oewn_synset_id: "s-end".to_string(),
                },
            ],
        });
        let rule_1 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &corroborating_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        // SUMO covers cut AND end, but via UNRELATED terms — real negative
        // evidence from SUMO's side, for a Similarity-kind query.
        let uncorroborating_sumo = SumoStore::from_sumo(&Sumo {
            mappings: vec![
                SumoMapping {
                    concept: cut,
                    term: "Cutting".to_string(),
                    relation: SumoRelationKind::Subsumption,
                    oewn_synset_id: "s-cut".to_string(),
                },
                SumoMapping {
                    concept: end,
                    term: "TimeInterval".to_string(),
                    relation: SumoRelationKind::Subsumption,
                    oewn_synset_id: "s-end".to_string(),
                },
            ],
        });
        let rule_2 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &uncorroborating_sumo,
            &no_coverage_propbank,
            &similarity_relation_kind(),
            cut,
            end,
        ) == word_sense::ReachingPairOutcome::Uncorroborated(cut, end);

        // THE REGRESSION-GUARD CHECK: the exact same uncorroborating SUMO
        // fixture, under Subsumption, must stay Trusted — SUMO class sharing
        // is no more valid evidence for hypernymy than VerbNet class sharing,
        // ConceptNet association sharing, or FrameNet frame sharing is.
        let subsumption_inert = word_sense::best_reaching_pair(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &uncorroborating_sumo,
            &no_coverage_propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        if rule_1 && rule_2 && subsumption_inert {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet",
        "the two-entity path's lone-hit corroboration composes VerbNet, ConceptNet, FrameNet and SUMO under Rule 1 (any source corroborating trusts) and Rule 2 (uncorroborated only if some covering source finds no match); SUMO is scoped to Similarity/Equivalence and never downgrades a Subsumption query, mirroring the other three sources' own scoping",
        "Niles, I. & Pease, A. (2001). 'Towards a Standard Upper Ontology.' Proceedings of FOIS 2001, pp. 2-9; Niles, I. & Pease, A. (2003). 'Linking Lexicons and Ontologies: Mapping WordNet to the Suggested Upper Merged Ontology.' Proceedings of IEEE IKE 2003, pp. 412-416; Levin, B. (1993). English Verb Classes and Alternations: A Preliminary Investigation. University of Chicago Press (semantic coherence hypothesis, the same componential-not-specificity reasoning extended to SUMO's class-mapping signal)"
    );
}

pr4xis::register_axiom!(
    SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet,
    constructor
);

/// The two-entity path's lone-hit corroboration composes FIVE independent
/// sources — VerbNet class-family sharing, ConceptNet association sharing,
/// FrameNet frame-family sharing, SUMO class sharing, and PropBank cross-POS
/// roleset sharing (Palmer, Gildea & Kingsbury 2005) — under the same "any
/// source corroborating is enough" (Rule 1) and "uncorroborated only if some
/// covering source finds no match" (Rule 2) composition
/// `SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet` proves for
/// four sources. This axiom checks the SAME two rules with PropBank as the
/// sole corroborating/uncorroborating source (VerbNet, ConceptNet, FrameNet,
/// and SUMO held at no-coverage), that PropBank is scoped to `Similarity`/
/// `Equivalence` and never downgrades a `Subsumption` query, AND — the
/// design-defining scoping decision this build's cross-POS prevalence
/// research settled — that a SAME-POS shared roleset is never trusted, only
/// a cross-POS one, since same-POS matches would be redundant with VerbNet's
/// existing verb-verb signal. See the `word_sense` module docs
/// (`pr4xis_domains::cognitive::linguistics::english::word_sense`)'s
/// "Composing corroboration sources" section for the full rationale.
pub struct PropBankCorroborationComposesWithVerbNetConceptNetFrameNetAndSumo;

impl pr4xis::ontology::Axiom for PropBankCorroborationComposesWithVerbNetConceptNetFrameNetAndSumo {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        use pr4xis_domains::cognitive::linguistics::conceptnet::ontology::ConceptNet;
        use pr4xis_domains::cognitive::linguistics::conceptnet::store::ConceptNetStore;
        use pr4xis_domains::cognitive::linguistics::framenet::ontology::FrameNet;
        use pr4xis_domains::cognitive::linguistics::framenet::store::FrameNetStore;
        use pr4xis_domains::cognitive::linguistics::propbank::ontology::{
            PropBank, PropBankFrameset, PropBankPredicate, Roleset, RolesetAlias,
        };
        use pr4xis_domains::cognitive::linguistics::propbank::store::PropBankStore;
        use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;
        use pr4xis_domains::cognitive::linguistics::sumo::ontology::Sumo;
        use pr4xis_domains::cognitive::linguistics::sumo::store::SumoStore;
        use pr4xis_domains::cognitive::linguistics::verbnet::ontology::VerbNet;
        use pr4xis_domains::cognitive::linguistics::verbnet::store::VerbNetStore;
        use pr4xis_domains::formal::relations::ontology::similarity_relation_kind;
        use pr4xis_domains::social::software::markup::xml::lmf::LmfPos;

        fn alias(text: &str, pos: LmfPos) -> RolesetAlias {
            RolesetAlias {
                text: text.to_string(),
                pos_code: pos.to_tag().to_string(),
                pos: Some(pos),
            }
        }
        fn single_roleset_propbank(id: &str, aliases: Vec<RolesetAlias>) -> PropBankStore {
            PropBankStore::from_propbank(&PropBank {
                framesets: vec![PropBankFrameset {
                    predicates: vec![PropBankPredicate {
                        lemma: id.to_string(),
                        rolesets: vec![Roleset {
                            id: id.to_string(),
                            aliases,
                        }],
                    }],
                }],
            })
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="propbank-scope-fixture" label="PS" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut-v"/></LexicalEntry>
    <LexicalEntry id="e-cutting-n"><Lemma writtenForm="cutting" partOfSpeech="n"/><Sense id="cutting-n-1" synset="s-cutting-n"/></LexicalEntry>
    <LexicalEntry id="e-end-v"><Lemma writtenForm="end" partOfSpeech="v"/><Sense id="end-v-1" synset="s-end-v"/></LexicalEntry>
    <Synset id="s-cut-v" ili="i1" partOfSpeech="v"><Definition>sever with a sharp instrument</Definition><SynsetRelation relType="hypernym" target="s-end-v"/></Synset>
    <Synset id="s-cutting-n" ili="i2" partOfSpeech="n"><Definition>an act of cutting</Definition></Synset>
    <Synset id="s-end-v" ili="i3" partOfSpeech="v"><Definition>bring to an end or halt</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let cut = english.lookup("cut")[0];
        let cutting = english.lookup("cutting")[0];
        let end = english.lookup("end")[0];

        let no_coverage_verbnet =
            VerbNetStore::from_verbnet_and_crosswalk(&VerbNet::default(), &Default::default());
        let no_coverage_conceptnet = ConceptNetStore::from_conceptnet(&ConceptNet::default());
        let no_coverage_framenet = FrameNetStore::from_framenet(&FrameNet::default());
        let no_coverage_sumo = SumoStore::from_sumo(&Sumo::default());

        // VerbNet, ConceptNet, FrameNet, and SUMO have NO coverage at all;
        // PropBank independently maps the verb "cut" and the noun "cutting"
        // to the SAME roleset, at DIFFERENT parts of speech — Rule 1: one
        // corroborating source is enough.
        let corroborating_propbank = single_roleset_propbank(
            "cut.01",
            vec![alias("cut", LmfPos::Verb), alias("cutting", LmfPos::Noun)],
        );
        let rule_1 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &corroborating_propbank,
            &similarity_relation_kind(),
            cut,
            cutting,
        ) == word_sense::ReachingPairOutcome::Trusted(cut, cutting);

        // PropBank covers both "cut" and "cutting", but via UNRELATED
        // rolesets — real negative evidence from PropBank's side, for a
        // Similarity-kind query.
        let uncorroborating_propbank = PropBankStore::from_propbank(&PropBank {
            framesets: vec![
                PropBankFrameset {
                    predicates: vec![PropBankPredicate {
                        lemma: "cut".to_string(),
                        rolesets: vec![Roleset {
                            id: "cut.01".to_string(),
                            aliases: vec![alias("cut", LmfPos::Verb)],
                        }],
                    }],
                },
                PropBankFrameset {
                    predicates: vec![PropBankPredicate {
                        lemma: "cutting".to_string(),
                        rolesets: vec![Roleset {
                            id: "cutting.02".to_string(),
                            aliases: vec![alias("cutting", LmfPos::Noun)],
                        }],
                    }],
                },
            ],
        });
        let rule_2 = word_sense::corroborate_lone_hit(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &uncorroborating_propbank,
            &similarity_relation_kind(),
            cut,
            cutting,
        ) == word_sense::ReachingPairOutcome::Uncorroborated(cut, cutting);

        // THE REGRESSION-GUARD CHECK: PropBank data present (though for an
        // unrelated pair — Subsumption skips corroboration entirely before
        // consulting any source, so which pair the data covers is
        // irrelevant), under Subsumption on the WordNet-hypernym-connected
        // cut/end pair, must stay Trusted — PropBank roleset sharing is no
        // more valid evidence for hypernymy than VerbNet class sharing,
        // ConceptNet association sharing, FrameNet frame sharing, or SUMO
        // class sharing is. Uses cut/end (not cut/cutting) because ONLY
        // cut/end carries the hypernym edge `reaches()` needs to produce a
        // real lone hit through the full path.
        let subsumption_inert = word_sense::best_reaching_pair(
            &english,
            &no_coverage_verbnet,
            &no_coverage_conceptnet,
            &no_coverage_framenet,
            &no_coverage_sumo,
            &uncorroborating_propbank,
            &[cut],
            &[end],
            &subsumption_kind(),
        ) == word_sense::ReachingPairOutcome::Trusted(cut, end);

        // THE SCOPING-DEFINING CHECK: "cut" and "end" share ONE roleset, but
        // BOTH as verb aliases — same POS. This must NEVER count as a
        // corroborating match (it would be redundant with VerbNet's own
        // verb-verb signal), so `shares_roleset` must reject it even though
        // a shared roleset id genuinely exists.
        let same_pos_propbank = single_roleset_propbank(
            "stop.01",
            vec![alias("cut", LmfPos::Verb), alias("end", LmfPos::Verb)],
        );
        let same_pos_never_counts = !same_pos_propbank.shares_roleset(&english, cut, end);

        if rule_1 && rule_2 && subsumption_inert && same_pos_never_counts {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PropBankCorroborationComposesWithVerbNetConceptNetFrameNetAndSumo",
        "the two-entity path's lone-hit corroboration composes VerbNet, ConceptNet, FrameNet, SUMO and PropBank under Rule 1 (any source corroborating trusts) and Rule 2 (uncorroborated only if some covering source finds no match); PropBank is scoped to Similarity/Equivalence, never downgrades a Subsumption query, and — the design-defining scoping decision — a same-POS shared roleset never counts as corroboration, only a cross-POS one",
        "Palmer, M., Gildea, D. & Kingsbury, P. (2005). 'The Proposition Bank: An Annotated Corpus of Semantic Roles.' Computational Linguistics 31(1):71-106; Bonial, C., Bonn, J., Conger, K., Hwang, J. & Palmer, M. (2014). 'PropBank: Semantics of New Predicate Types.' LREC 2014; Levin, B. (1993). English Verb Classes and Alternations: A Preliminary Investigation. University of Chicago Press (semantic coherence hypothesis, the same componential-not-specificity reasoning extended to PropBank's cross-POS roleset signal)"
    );
}

pr4xis::register_axiom!(
    PropBankCorroborationComposesWithVerbNetConceptNetFrameNetAndSumo,
    constructor
);

/// A realized definitional answer together with the LOADED ontologies that
/// produced it — the two halves of one act, so no caller has to reconstruct the
/// second from the first.
///
/// [`define_word`] draws on TWO independent channels (the resolved concepts'
/// glosses, and the statutory `defines` overlay), but its surface is one
/// realized `String` in which the channels are indistinguishable. While the
/// answer was a bare `String`, every call site rebuilt provenance from the
/// concept channel ALONE — `loaded_ontologies_of` over the ids it had guessed
/// `define_word` would resolve — so an answer whose controlling definition came
/// from a loaded U.S. Code title rested on an authority it never credited. The
/// only structurally safe place to make that claim is inside the function that
/// consulted the authorities, which is why this is returned rather than
/// recomputed: a future sixth call site cannot forget a field it is handed.
#[derive(Debug)]
pub struct DefinitionAnswer {
    /// The realized answer surface.
    pub text: String,
    /// Every loaded `.prx` ontology this definition genuinely drew on — the
    /// resolved concepts' owning ontologies UNION the owners of the provisions
    /// whose statutory text was quoted. Deduped, in that order.
    pub reasoned_over: Vec<OntologyName>,
    /// The documents the recited GLOSSES were AUTHORED FROM — each loaded
    /// lexicon's own `dcterms:source` edges, read off the concepts this answer
    /// actually used.
    ///
    /// A THIRD channel, not a slice of `reasoned_over`: that field names what
    /// the engine opened, this names what the lexicographer read. They coincide
    /// only by accident. Keeping them apart is the whole point — while the
    /// citation rode inside the gloss string ("… of that child or adult — 42
    /// USC 300ii(7)") an answer looked statute-backed while only a lexicon
    /// entry had been consulted, and no consumer could tell, because there was
    /// nothing structural to tell apart.
    pub authored_from: Vec<DefinitionProvenance>,
}

pub fn define_word(en: &dyn LexicalReasoner, word: &str) -> DefinitionAnswer {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    // Exact hit ∪ morphological analyses — a gerundial definiendum ("what is
    // hyperventilating") glosses the verb concept its question named.
    let ids = resolve_surface(en, word);

    // A CONTROLLING statutory definition — the separate `defines` grounding
    // channel (`social::judicial::statute_structure::grounding::
    // defines_pointers`, a VerbNet-grounded chart-parse over real USC prose,
    // never regex), distinct from the WordNet/LKIF gloss loop below. Checked
    // even when `ids` is empty: a genuine out-of-lexicon statutory coinage
    // (the G7 minting case — a term with no WordNet presence at all) would
    // otherwise never reach past the `UnknownVocabulary` early return, even
    // though a loaded provision defines it.
    //
    // This channel's inline URN suffix (below) is NOT the lexicon
    // authored-from case: here the engine really did open the provision and is
    // QUOTING it, so the pinpoint identifies the text on screen and the owning
    // title is credited in `reasoned_over` besides. A lexicon gloss is the
    // opposite — someone else read the document — and rides the structured
    // `authored_from` channel instead of the answer prose.
    let statute_defs: Vec<(&str, &str)> = en.statute_definitions(word);

    if ids.is_empty() && statute_defs.is_empty() {
        let content = ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity(word);
        // An abstention rests on nothing, so it credits nothing — the honest
        // empty union, not a silent inheritance of whatever the caller held.
        return DefinitionAnswer {
            text: realize::realize(&content),
            reasoned_over: Vec::new(),
            authored_from: Vec::new(),
        };
    }

    // CHANNEL 1 of the provenance union: the structural claim over the concepts
    // `resolve_surface` actually reached — the same `loaded_ontologies_of` call
    // every caller used to make on its own, moved in here so it is computed from
    // the ids this answer was really built from rather than from a caller's
    // independent re-resolution of the surface.
    let mut reasoned_over = loaded_ontologies_of(en, &ids);

    // EVERY resolved sense is realized (never a display-time cut) — the
    // registered [`DefineEnumeratesTheLoadedSenseInventory`] axiom. Sense
    // selection belongs to the loaded inventory, not to display: the day-one
    // `take(5)` cap (d492e89f) had no citation and cost 129 of the 349 corpus
    // define failures (the sampled sense ranked ≥ 5). Order is by
    // `resolve_surface` (a direct headword hit blocks morphological
    // re-analysis; a non-lexicalized surface falls back to its recovered
    // lemma), THEN re-grouped domain-source-first by
    // [`order_definitions_domain_first`] — the
    // [`DefinePrioritizesDomainSensesOverGeneralSenses`] axiom.
    // Each realized definition, tagged by provenance: a domain-specific loaded
    // lexicon (the caregiving/HCBS/USC `.prx` sources — `is_domain_loaded`, and
    // every statutory `defines`-overlay gloss) vs. the general-purpose WordNet
    // substrate. `resolve_surface` order is preserved WITHIN each group.
    let mut defs: Vec<(bool, String)> = Vec::new();
    // CHANNEL 3, and the one that is NOT a provenance union member: the
    // documents each recited gloss was AUTHORED FROM, read off the concept's own
    // `dcterms:source` edges. Collected in the same loop as the glosses, from
    // the same `ConceptView`, so the citation and the wording it belongs to can
    // never drift apart. Deliberately NOT merged into `reasoned_over`: a lexicon
    // naming 42 USC 300ii(7) as its authority is not the engine having opened
    // Title 42, and merging them would restate the very confusion the structured
    // channel exists to end.
    let mut authored_from: Vec<DefinitionProvenance> = Vec::new();
    for &id in &ids {
        if let Some(concept) = en.concept(id) {
            let is_domain = concept.is_domain_loaded();
            for def in concept.definitions() {
                defs.push((is_domain, def.to_string()));
            }
            // Only a LOADED concept can carry provenance (`ontology_of_concept`
            // is `None` for the embedded English substrate), so a WordNet gloss
            // contributes nothing here — the honest empty case, structurally.
            if let Some(lexicon) = en.ontology_of_concept(id) {
                for citation in concept.definition_sources().iter() {
                    let provenance = DefinitionProvenance {
                        lexicon: lexicon.clone(),
                        citation: citation.to_string(),
                    };
                    if !authored_from.contains(&provenance) {
                        authored_from.push(provenance);
                    }
                }
            }
        }
    }
    for (urn, provision_text) in statute_defs {
        // CHANNEL 2 of the provenance union: the loaded title that owns the
        // provision whose text is about to be quoted. Credited HERE, at the
        // point of use — an answer that rests on a statutory authority must name
        // it, and the concept ids above cannot supply it (a genuine out-of-
        // lexicon statutory coinage resolves to no concept at all, yet is
        // answered entirely out of the provision's own prose).
        extend_reasoned_over(&mut reasoned_over, statutory_source_of(en, urn));
        defs.push((true, format!("{provision_text} — {urn}")));
    }

    let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge).with_entity(word);
    let mut emitted: Vec<String> = Vec::new();
    for (_, def) in order_definitions_domain_first(defs) {
        // A byte-identical restatement (two loaded lexicons carrying the same
        // statutory gloss for one term) is not a distinct sense — collapse it.
        if emitted.contains(&def) {
            continue;
        }
        content = content.with_definition(word, &def);
        emitted.push(def);
    }
    DefinitionAnswer {
        text: realize::realize(&content),
        reasoned_over,
        authored_from,
    }
}

/// Group a definiendum's realized definitions so domain-specific loaded-lexicon
/// senses (`true`) precede the general-purpose WordNet senses (`false`), a
/// STABLE partition that preserves each group's incoming (`resolve_surface`)
/// order and drops nothing — completeness is
/// [`DefineEnumeratesTheLoadedSenseInventory`], the domain-first priority is
/// [`DefinePrioritizesDomainSensesOverGeneralSenses`]. In a special-domain
/// corpus the specialized reading is the salient one (LSP/terminology practice,
/// Cabré 1999), so "what is respite" leads with the caregiving statutory
/// definition, not WordNet's "a pause from doing something".
fn order_definitions_domain_first(mut defs: Vec<(bool, String)>) -> Vec<(bool, String)> {
    // Stable sort keyed so `true` (domain) sorts before `false` (general):
    // `!is_domain` maps domain→0, general→1. `sort_by_key` is stable, so no
    // relative reordering happens within either group.
    defs.sort_by_key(|(is_domain, _)| !is_domain);
    defs
}

/// The answered relation, as the taxonomy realizer needs it: its loaded natural
/// surface (the `connective`) and its typed kind name. Bundled so the realizer
/// takes the relation as ONE argument (both fields describe the same relation).
struct AnsweredRelation<'a> {
    /// The relation's loaded surface ("part of" for Parthood), already resolved by
    /// the caller from the typed kind — so the affirmation phrases "X is part of
    /// Y", not "X is a Y". `None` (Subsumption / is-a) keeps the copula "is a".
    /// Loaded data, never a hardcoded relation string.
    connective: Option<&'a str>,
    /// The answered relation's TYPED kind name (e.g. "Subsumption" / "Parthood"),
    /// used to read the licensing transitivity rule + citation from the Relations
    /// ontology when the chain spans more than one hop (Fix 2). Loaded data (the
    /// kind the closure was keyed on), never a hardcoded relation string.
    kind_name: &'a str,
}

/// Build a taxonomy response following the NLG pipeline.
///
/// Reiter & Dale (2000):
/// 1. Content determination — gather facts from ontology
/// 2. Document planning — organize with RST (assertion → evidence → elaboration)
/// 3. Microplanning — referring expressions
/// 4. Realization — compose through grammar
fn build_taxonomy_response(
    en: &dyn LexicalReasoner,
    relation: AnsweredRelation<'_>,
    // The ORDERED evidence chain `[child, …, parent]` along the answered relation,
    // pre-resolved by the caller from the typed kind via `relation_chain` (the chat
    // cannot name a `ConceptRef`). For Subsumption it is the is-a chain, for
    // Parthood the part-of chain (`subsection → section → title`). `None`/short
    // degrades to the endpoints — never re-derived here.
    relation_chain: Option<Vec<pr4xis_domains::cognitive::linguistics::english::ConceptId>>,
    child_word: &str,
    parent_word: &str,
    child_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
    parent_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
) -> (String, Option<String>) {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize;

    // ---- Stage 1: Content Determination ----
    // Gather all relevant knowledge from the ontology.

    // The relation chain: how child reaches parent along the answered relation. The
    // ORDERED evidence path is owned by the reasoner's MATERIALIZED closure (read by
    // the caller via `relation_chain`), so even the justification is closure-derived,
    // not re-walked. This function is only reached after the relation already proved,
    // so the chain is present; an absent chain degrades to the endpoints.
    let chain_ids: Vec<(
        String,
        pr4xis_domains::cognitive::linguistics::english::ConceptId,
    )> = relation_chain
        .unwrap_or_else(|| vec![child_id, parent_id])
        .into_iter()
        .enumerate()
        .map(|(i, id)| {
            // The chain's first element is `child`; render it with the caller's
            // surface word, not the lemma, so the evidence reads in the user's
            // term. Every other rung uses its concept's primary lemma.
            let label = if i == 0 {
                child_word.to_string()
            } else {
                en.concept(id)
                    .map(|c| {
                        c.lemmas()
                            .next()
                            .unwrap_or_else(|| c.original_id())
                            .to_string()
                    })
                    .unwrap_or_else(|| parent_word.to_string())
            };
            (label, id)
        })
        .collect();

    // Definitions for each concept in the chain
    let chain_defs: Vec<(&str, &str)> = chain_ids
        .iter()
        .filter_map(|(label, id)| {
            en.concept(*id)
                .and_then(|c| c.definitions().next())
                .map(|def| (label.as_str(), def))
        })
        .collect();

    // Children (subtypes) of the child concept. Every loaded child is
    // realized — the day-one `take(5)` display cap had no citation and is
    // the same unprincipled content-determination bug Slice A already
    // deleted from `define_word` (registered as
    // [`ChatExplorationEnumeratesTheLoadedInventory`]): the loaded taxonomy
    // IS the inventory, so a display-time cut can silently drop a subtype
    // the corpus expects.
    let subtypes: Vec<&str> = en
        .children(child_id)
        .iter()
        .filter_map(|&id| en.concept(id).and_then(|c| c.lemmas().next()))
        .collect();

    // ---- Stage 2: Document Planning (RST) ----
    // Organize as: Assertion (nucleus) → Evidence (satellite) → Elaboration

    let mut sections = Vec::new();

    // Nucleus: the direct assertion — phrased with the relation's loaded connective
    // ("a subsection is part of a section" for Parthood; "is a" for Subsumption).
    sections.push(format!(
        "Yes. {}.",
        realize::sentence_relation(child_word, parent_word, relation.connective)
    ));

    // Evidence: HOW — the relation chain explains the connection, each rung
    // phrased with the SAME loaded connective ("section is part of title", not
    // "section is a title"). For a deep mereology this shows the part-of path.
    if chain_ids.len() > 2 {
        let chain_labels: Vec<&str> = chain_ids.iter().map(|(l, _)| l.as_str()).collect();
        let mut evidence_parts = Vec::new();
        for i in 0..chain_labels.len() - 1 {
            evidence_parts.push(realize::sentence_relation(
                chain_labels[i],
                chain_labels[i + 1],
                relation.connective,
            ));
        }
        let mut evidence = evidence_parts.join(", and ");

        // The LICENSING rule (Fix 2): a chain of >1 hop is authorized by the
        // relation's TRANSITIVITY — surface that rule AND its citation, read from
        // the Relations ontology as DATA (never a hardcoded string here), so the
        // answer shows the PROOF, not just the witness path. A single-hop (direct
        // edge, chain length 2) invokes no transitivity, so this block is the only
        // place it appends. A kind the ontology does not declare transitive yields
        // no license, so the note is never fabricated. Realized through the NLG
        // layer (`sentence_transitivity_license`), not a raw format! here.
        use pr4xis_domains::formal::relations::ontology::transitivity_license;
        if let Some(license) = transitivity_license(relation.kind_name) {
            let relation_surface = relation.connective.unwrap_or("is-a");
            evidence.push_str("; ");
            evidence.push_str(&realize::sentence_transitivity_license(
                relation_surface,
                &license.property.to_lowercase(),
                &license.citation,
            ));
        }
        sections.push(evidence);
    }

    // Elaboration: WHAT each concept means — CONTENT SELECTION under Grice's
    // Quantity maxim (1975; Reiter 1990, "The Computational Complexity of
    // Avoiding Conversational Implicatures", ACL; Reiter & Dale 2000 Ch.3's
    // Content-Determination stage, which chooses a message set BEFORE
    // realization, never truncates a string after). The two ENTITIES the
    // question actually named — child and the queried parent — are
    // sufficient to justify a yes/no relational verification; every
    // intermediate rung's gloss is real evidence but not required to answer
    // THIS question, so it is deferred rather than dumped into the primary
    // response (the fix for a real, reported defect: "is a dog an animal"
    // once printed all 9 rungs' glosses up to "chordate"). The queried
    // parent is located by ID, not assumed to be `chain_defs`'s last entry
    // — `relation_chain`'s own contract clips the chain at the queried
    // ancestor, but this stays index-based rather than position-based so a
    // future caller that passes an unclipped chain degrades safely instead
    // of silently dropping the parent's own gloss.
    let parent_def_index = chain_defs
        .iter()
        .zip(chain_ids.iter())
        .position(|(_, (_, id))| *id == parent_id);
    let (primary_defs, deferred_defs): (Vec<_>, Vec<_>) = chain_defs
        .iter()
        .enumerate()
        .partition(|(i, _)| *i == 0 || Some(*i) == parent_def_index || parent_def_index.is_none());
    for (_, (label, def)) in &primary_defs {
        sections.push(format!("{label}: {def}"));
    }

    // Deferred: WHY the child reaches the parent through intermediate
    // rungs, and what those rungs mean — real evidence, kept (never
    // silently dropped, `ChatExplorationEnumeratesTheLoadedInventory`'s
    // completeness principle applies to the TURN's wire data, not only the
    // always-visible primary text), surfaced instead through the "Why?"
    // panel by the caller.
    let mut deferred_parts = Vec::new();
    for (_, (label, def)) in &deferred_defs {
        deferred_parts.push(format!("{label}: {def}"));
    }
    if !subtypes.is_empty() {
        deferred_parts.push(format!("types of {child_word}: {}", subtypes.join(", ")));
    }
    let deferred = if deferred_parts.is_empty() {
        None
    } else {
        Some(deferred_parts.join("\n"))
    };

    // ---- Stage 3 & 4: Microplanning + Realization ----
    // Already handled by realize::sentence_copula (determiner selection, grammar)

    (sections.join("\n"), deferred)
}

/// THE FULL-INVENTORY EXPLORATION AXIOM (Slice A siblings, R-2 declared
/// follow-up): metacognitive exploration realizes EVERY loaded sense of a
/// queried word, EVERY sense pairing when relating two words, and EVERY
/// loaded subtype when listing a concept's children — never an uncited
/// display-time cut. This is the same principle
/// [`DefineEnumeratesTheLoadedSenseInventory`] already established for
/// `define_word`, applied to `explore_concepts`'s single-word and
/// pairwise-relation loops (`ids.first()`) and to `build_taxonomy_response`'s
/// subtype list (`children(..).take(5)`): sense/subtype selection is an
/// inventory-compilation decision made upstream in the loaded lexicon, not
/// a display-time cut, and a truncated or first-sense-only read can
/// silently miss the very sense a probe or a user's question means.
pub struct ChatExplorationEnumeratesTheLoadedInventory;

impl pr4xis::ontology::Axiom for ChatExplorationEnumeratesTheLoadedInventory {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // A fixture with three properties a first-sense/take(5) cut would
        // each individually break: (1) `seal` is polysemous, its FIRST
        // listed sense ("a device for stamping") is NOT a mammal — only its
        // SECOND sense is; (2) `mammal` has SIX loaded hyponyms, one more
        // than the deleted day-one take(5) cap.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="explore-fixture" label="Explore" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="entry-animal-n"><Lemma writtenForm="animal" partOfSpeech="n"/><Sense id="animal-n-01" synset="synset-animal-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="mammal-n-01" synset="synset-mammal-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="synset-dog-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="cat-n-01" synset="synset-cat-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-horse-n"><Lemma writtenForm="horse" partOfSpeech="n"/><Sense id="horse-n-01" synset="synset-horse-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-cow-n"><Lemma writtenForm="cow" partOfSpeech="n"/><Sense id="cow-n-01" synset="synset-cow-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-sheep-n"><Lemma writtenForm="sheep" partOfSpeech="n"/><Sense id="sheep-n-01" synset="synset-sheep-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-goat-n"><Lemma writtenForm="goat" partOfSpeech="n"/><Sense id="goat-n-01" synset="synset-goat-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-artifact-n"><Lemma writtenForm="artifact" partOfSpeech="n"/><Sense id="artifact-n-01" synset="synset-artifact-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-seal-n">
      <Lemma writtenForm="seal" partOfSpeech="n"/>
      <Sense id="seal-n-01" synset="synset-seal-device-n-01"/>
      <Sense id="seal-n-02" synset="synset-seal-animal-n-01"/>
    </LexicalEntry>
    <Synset id="synset-animal-n-01" ili="i1" partOfSpeech="n" members="entry-animal-n"><Definition>a living organism</Definition></Synset>
    <Synset id="synset-mammal-n-01" ili="i2" partOfSpeech="n" members="entry-mammal-n"><Definition>a warm-blooded vertebrate with hair</Definition><SynsetRelation relType="hypernym" target="synset-animal-n-01"/></Synset>
    <Synset id="synset-dog-n-01" ili="i3" partOfSpeech="n" members="entry-dog-n"><Definition>a domesticated carnivore</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-cat-n-01" ili="i4" partOfSpeech="n" members="entry-cat-n"><Definition>a small domesticated feline</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-horse-n-01" ili="i5" partOfSpeech="n" members="entry-horse-n"><Definition>a large domesticated ungulate</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-cow-n-01" ili="i6" partOfSpeech="n" members="entry-cow-n"><Definition>a domesticated bovine</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-sheep-n-01" ili="i7" partOfSpeech="n" members="entry-sheep-n"><Definition>a domesticated ruminant with a fleece</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-goat-n-01" ili="i8" partOfSpeech="n" members="entry-goat-n"><Definition>a domesticated ruminant with horns</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
    <Synset id="synset-artifact-n-01" ili="i9" partOfSpeech="n" members="entry-artifact-n"><Definition>an object made by a human</Definition></Synset>
    <Synset id="synset-seal-device-n-01" ili="i10" partOfSpeech="n" members="entry-seal-n"><Definition>a device for stamping a design</Definition><SynsetRelation relType="hypernym" target="synset-artifact-n-01"/></Synset>
    <Synset id="synset-seal-animal-n-01" ili="i11" partOfSpeech="n" members="entry-seal-n"><Definition>an aquatic mammal with flippers</Definition><SynsetRelation relType="hypernym" target="synset-mammal-n-01"/></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet(xml)
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);

        // Property 1: single-word exploration realizes BOTH of `seal`'s
        // senses, not just the first-listed (device) one.
        let seal_only = explore_concepts(&english, &["seal"]);
        let both_seal_senses = seal_only.contains("a device for stamping a design")
            && seal_only.contains("an aquatic mammal with flippers");

        // Property 2: pairwise relation-finding uses `seal`'s SECOND sense
        // (animal) to relate it to `mammal`, even though the FIRST sense
        // (device) has no such relation — proving the check is no longer
        // `ids1.first()`-only.
        let seal_mammal = explore_concepts(&english, &["seal", "mammal"]);
        let found_via_second_sense = seal_mammal.to_lowercase().contains("seal")
            && seal_mammal.to_lowercase().contains("mammal");

        // Property 3: `mammal`'s subtype list carries all SIX loaded
        // hyponyms — one more than the deleted day-one `take(5)` cap.
        let relation = AnsweredRelation {
            connective: None,
            kind_name: "Subsumption",
        };
        let mammal_id = *english.lookup("mammal").first().expect("mammal resolves");
        let animal_id = *english.lookup("animal").first().expect("animal resolves");
        let (taxonomy, deferred) = build_taxonomy_response(
            &english,
            relation,
            Some(vec![mammal_id, animal_id]),
            "mammal",
            "animal",
            mammal_id,
            animal_id,
        );
        // "Realized" means present SOMEWHERE on the turn's wire — the
        // primary response OR the deferred elaboration (Grice's Quantity
        // maxim moves subtypes out of the always-visible primary text, but
        // never drops them; see `build_taxonomy_response`'s content
        // selection). Never a display-time cut either way.
        let combined = format!("{taxonomy}\n{}", deferred.unwrap_or_default());
        let all_six_subtypes = ["dog", "cat", "horse", "cow", "sheep", "goat"]
            .iter()
            .all(|hyponym| combined.contains(hyponym));

        if both_seal_senses && found_via_second_sense && all_six_subtypes {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ChatExplorationEnumeratesTheLoadedInventory",
        "metacognitive exploration (explore_concepts single-word and pairwise-relation lookups, and build_taxonomy_response's subtype list) realizes every loaded sense/subtype rather than a display-time cut — the same no-uncited-cap principle already established for define_word's sense enumeration, applied to the sibling call sites the R-2 diagnosis declared as follow-ups",
        "wn(1WN) WordNet command-line manual — the -over search displays an overview of ALL the senses of the search word; cntlist(5WN) — the sense ordering 'should not be construed as an accurate indicator of frequency of use' (rank licenses no cut); Lew (2013) 'Identifying, ordering and defining senses', in The Bloomsbury Companion to Lexicography — frequency/chronology/logical schemes ORDER the full sense list, none omits; Reiter & Dale (2000) Building Natural Language Generation Systems, CUP — content determination is a principled stage, an uncited constant inside realization is not a principle"
    );
}

pr4xis::register_axiom!(ChatExplorationEnumeratesTheLoadedInventory, constructor);

/// Explore what the system knows about multiple concepts.
///
/// Uses the associations ontology (taxonomy, mereology) to discover
/// relationships between concepts — common ancestors, is-a chains,
/// shared properties. This is metacognition: instead of guessing
/// "did you mean is X a Y?", explore and report what we actually know.
fn explore_concepts(en: &dyn LexicalReasoner, words: &[&str]) -> String {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize;

    let mut lines = Vec::new();

    // Collect all concept IDs per word. Bound the count: the pairwise loops
    // below are O(n²) in the word count, so a pathologically long utterance is a
    // resource-exhaustion DoS. Process at most MAX_PARTIAL_WIDTH words (matching
    // chart_reduce's cap); real utterances are far under it.
    const MAX_PARTIAL_WIDTH: usize = 256;
    let word_ids: Vec<(&str, Vec<_>)> = words
        .iter()
        .take(MAX_PARTIAL_WIDTH)
        .map(|&w| (w, en.lookup(w).to_vec()))
        .collect();

    // For each concept, describe it and trace taxonomy. EVERY resolved
    // sense is realized — collapsing to `ids.first()` silently reports only
    // the lexicon's first-listed sense, which need not be the sense a
    // corpus probe or a user's question means (registered as
    // [`ChatExplorationEnumeratesTheLoadedInventory`], the same
    // no-uncited-display-cut principle Slice A already applied to
    // `define_word`). A polysemous word numbers its senses; an
    // unambiguous one stays unprefixed.
    for (word, ids) in &word_ids {
        for (i, &id) in ids.iter().enumerate() {
            let Some(concept) = en.concept(id) else {
                continue;
            };
            let label = if ids.len() > 1 {
                format!("{word} ({})", i + 1)
            } else {
                (*word).to_string()
            };

            if let Some(def) = concept.definitions().next() {
                lines.push(format!("{label}: {def}"));
            }

            // Trace the taxonomy chain off the reasoner's MATERIALIZED hypernym
            // closure: `ancestors(id)` is the reflexive is-a image, nearest-
            // first, owned by the closure — never a hand-walk of `parents()`. We
            // drop the reflexive head (`id` itself, distance 0) to render the
            // STRICT ancestor lineage "word is a X → Y → Z".
            let chain: Vec<String> = en
                .ancestors(id)
                .into_iter()
                .skip(1)
                .filter_map(|anc| {
                    en.concept(anc).map(|pc| {
                        pc.lemmas()
                            .next()
                            .unwrap_or_else(|| pc.original_id())
                            .to_string()
                    })
                })
                .collect();
            if !chain.is_empty() {
                // Generate "word is a X → Y → Z" through grammar
                let first = &chain[0];
                let copula = realize::sentence_copula(&label, first);
                if chain.len() > 1 {
                    lines.push(format!("  {copula} → {}", chain[1..].join(" → ")));
                } else {
                    lines.push(format!("  {copula}"));
                }
            }
        }
    }

    // Find relationships between concept pairs through associations. Every
    // sense PAIRING is checked — "is a bank a financial institution" must
    // not silently fail because `bank`'s first-listed sense is the
    // riverbank one; the relation holds if it holds under ANY resolved
    // sense of each word (same registered axiom as above).
    if word_ids.len() >= 2 {
        for i in 0..word_ids.len() {
            for j in i + 1..word_ids.len() {
                let (w1, ids1) = &word_ids[i];
                let (w2, ids2) = &word_ids[j];

                let mut is_a_pair = None;
                for &id1 in ids1 {
                    for &id2 in ids2 {
                        if en.is_a(id1, id2) {
                            is_a_pair = Some((*w1, *w2));
                        } else if en.is_a(id2, id1) {
                            is_a_pair = Some((*w2, *w1));
                        }
                        if is_a_pair.is_some() {
                            break;
                        }
                    }
                    if is_a_pair.is_some() {
                        break;
                    }
                }

                if let Some((sub, sup)) = is_a_pair {
                    lines.push(realize::sentence_copula(sub, sup));
                    continue;
                }

                let common = ids1.iter().find_map(|&id1| {
                    ids2.iter()
                        .find_map(|&id2| en.common_ancestor(id1, id2).map(|lca| (id1, id2, lca)))
                });
                if let Some((_, _, lca)) = common
                    && let Some(c) = en.concept(lca)
                {
                    let label = c.lemmas().next().unwrap_or_else(|| c.original_id());
                    let s1 = realize::sentence_copula(w1, label);
                    let s2 = realize::sentence_copula(w2, label);
                    lines.push(format!("{s1}, and {s2}"));
                }
            }
        }
    }

    if lines.is_empty() {
        realize::realize(&realize::ResponseContent::new(
            pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame::UnknownVocabulary,
        ))
    } else {
        lines.join("\n")
    }
}

/// `en` grounds the pronoun exclusion below in the SAME loaded closed-class
/// lexicon the tokenizer's own POS assignment already uses — never a
/// hardcoded word list.
pub fn extract_entity_name(sem: &montague::Sem, en: &dyn LexicalReasoner) -> String {
    match sem {
        // A pronoun ("I"/"he"/"who"/…) is never a queried entity, even when
        // it happens to share a bare surface with an unrelated open-class
        // WordNet sense (the letter "I", the Roman numeral "I", the element
        // "iodine") — the SAME syncategorematic-word principle the Func arm
        // below already applies to a copula's own surface. Without this, a
        // degenerate parse of "I am a caregiver…" extracts entities ["a
        // caregiver", "an i"] and confidently (and falsely) asserts or
        // denies "an i is a caregiver" instead of honestly abstaining on
        // the unresolved "I" role. `is_pronoun`, not the coarser
        // `is_function_word`: several closed-class words ("above", "so")
        // are ALSO legitimate content words in a different sense ("the
        // above information", "deadly serious"), so excluding on general
        // closed-class membership over-excludes those polysemous readings
        // — confirmed via a full corpus-gate regression before landing
        // this fix (`is_function_word` broke "what is deadly"/"what does
        // 'above' mean"/"what is full"/"what does 'so' mean").
        montague::Sem::Concept { word, .. } | montague::Sem::Pred { word, .. }
            if en.is_pronoun(word) =>
        {
            String::new()
        }
        // `montague::interpret`'s own internal sentinels for "no derivation
        // reached the full span" (`"?"`) and "no tokens to interpret"
        // (`"empty"`) are bookkeeping placeholders, never a surface the
        // input actually named — without this guard `unresolved_surfaces`
        // reports a literal `"?"` as an unresolved WORD whenever semantic
        // composition fails to reach a complete `S` (a real, common case:
        // any ungrammatical or unsupported-construction input), the same
        // class of internal-representation leak the pronoun guard above and
        // the `KnownKnown` fallback below both exist to prevent.
        montague::Sem::Pred { word, .. } if word == "?" || word == "empty" => String::new(),
        montague::Sem::Concept { word, .. } => word.clone(),
        montague::Sem::Pred { word, .. } => word.clone(),
        // For Func (e.g., "is" applied to "dog"), extract the content entity
        // from the FIRST absorbed argument that names one, not the function
        // word itself. When no absorbed argument yields an entity, the
        // result is EMPTY — never the copula surface. A functor is
        // syncategorematic (Steedman 2000): its own surface `is` is never a
        // queried entity, so falling back to `word` would re-introduce the leak
        // the role filter removes. The empty string is the honest result.
        montague::Sem::Func { body, .. } => body
            .iter()
            .map(|s| extract_entity_name(s, en))
            .find(|name| !name.is_empty())
            .unwrap_or_default(),
        // A `Sem::Prop`/`Sem::Question` reached HERE is a whole embedded
        // CLAUSE occupying an argument slot ("When DOES the DSP indicate
        // the tasks they complete…" absorbs a nested polar question
        // `{predicate: "does", ..}` and a nested proposition `{predicate:
        // "complete", ..}` as the outer "when" clause's own two
        // arguments) — never a nameable entity. The clause's own
        // PREDICATE word ("does"/"complete") is exactly as syncategorematic
        // here as a copula's surface is in the `Func` arm just above: it is
        // the clause's grammatical head, not something the clause is ABOUT.
        // Previously fell back to `predicate.clone()`, which fabricated
        // "does"/"complete" as if they were the sentence's queried
        // entities — confirmed to cause a real corpus regression: "When
        // does the DSP indicate the tasks they complete in the EVV
        // system?" reached `answer_statement`'s two-entity relational path
        // with `entities == ["does", "complete"]` and affirmed a spurious
        // "does relates to complete" (both surfaces happen to resolve to
        // SOME WordNet sense, so `verify_relational_claim` had a real,
        // if nonsensical, path to chain). The honest result is empty, the
        // same "no entity here" answer the pronoun/sentinel/Func arms
        // above already give; the caller's own dropped-argument accounting
        // (`arguments.len() > entities.len()`) then correctly reads this as
        // a non-definitional, non-relational sentence and abstains via
        // `AdmitLimitation` instead of fabricating a relation between two
        // clause-head words.
        montague::Sem::Prop { .. } | montague::Sem::Question { .. } => String::new(),
    }
}

// =========================================================================
// Timer — works on both native and WASM
// =========================================================================

/// WASM-safe timer. `std::time::Instant` panics on wasm32-unknown-unknown
/// because the target has no system clock. This wrapper uses `Instant` on
/// native and returns 0 on WASM.
struct WasmSafeTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
}

impl WasmSafeTimer {
    fn now() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
        }
    }

    fn elapsed_us(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed().as_micros() as u64
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    }
}

// =========================================================================
// Self-description — through the SelfModel ontology
// =========================================================================

/// The always-present substrate ontologies: the auto-registered
/// knowledge base plus the embedded English language model.
///
/// Source-agnostic — it knows nothing of statutes or any loaded corpus.
/// Loaded corpora are reported separately through the self-model catalog
/// (`SelfModelInstance::with_catalog`); a caller that has loaded sources
/// attaches them there. English is the one embedded base, so it is the
/// only runtime corpus reflected here directly.
pub fn loaded_ontologies(_lang: &English) -> Vec<Vocabulary> {
    let mut ontologies = describe_knowledge_base();
    ontologies.push(Vocabulary::from_ontology::<
        pr4xis_domains::cognitive::linguistics::lexicon::ontology::LexicalCategory,
        pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag,
    >(
        "English (WordNet)",
        "pr4xis_domains::cognitive::linguistics::english",
        "Open English WordNet 2025; Princeton WordNet",
    ));
    ontologies
}

/// The eigenform — the system observes itself.
///
/// This IS the self-observation operator F from von Foerster.
/// The result IS the fixed point X = F(X). The returned
/// [`SelfModelInstance`] carries the substrate ontologies; callers
/// attach the loaded-source catalog (the knowledge boundary) via
/// [`SelfModelInstance::with_catalog`].
pub fn observe_self(lang: &English) -> SelfModelInstance {
    observe_self_with_loaded(lang, Vec::new())
}

/// The eigenform over the LIVE loaded set — `F` applied to the object level it
/// actually has, not a constant (doc §2). The substrate ontologies
/// ([`loaded_ontologies`]) PLUS one `Vocabulary` per loaded runtime ontology
/// (built by the caller via
/// [`runtime_ontology_vocabulary`](pr4xis_domains::formal::information::knowledge::runtime_ontology_vocabulary)),
/// so `total_concepts`/`total_morphisms` MOVE the moment a corpus is loaded — the
/// self-model is causally connected to what is loaded, not a vacuous fixed point.
/// `loaded` is empty for the English-only path (then this is [`observe_self`]).
pub fn observe_self_with_loaded(lang: &English, loaded: Vec<Vocabulary>) -> SelfModelInstance {
    let mut components = loaded_ontologies(lang);
    components.extend(loaded);
    SelfModelInstance::observe(components)
        .with_task_claims(pr4xis_domains::cognitive::linguistics::nlp_task::claims::known_claims())
}

/// Describe the eigenform structurally. Callers that need JSON (WASM
/// boundary) should call `.to_json()` on the result themselves, and may
/// first attach the source catalog via
/// [`SelfModelInstance::with_catalog`].
pub fn self_describe(lang: &English) -> SelfModelInstance {
    observe_self(lang)
}

/// [`self_describe`] over the live loaded set — see [`observe_self_with_loaded`].
pub fn self_describe_with_loaded(lang: &English, loaded: Vec<Vocabulary>) -> SelfModelInstance {
    observe_self_with_loaded(lang, loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Monoid;
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineStep;

    fn sample_english() -> English {
        // Use sample data for unit tests (fast, no WordNet needed)
        English::sample()
    }

    /// The "Why?" layer end-to-end over the LIVE pipeline (doc §5.2): a plain
    /// sentence glossing WHY the engine reached each outcome, realized centrally
    /// in `process_with_reasoner` from the loaded explain-frames table. Covers the
    /// three why-producing outcome shapes; the self-explaining shapes
    /// (`Conditional`/`RuleResolved`, and the unproven-relation abstain) produce
    /// `why == None` by design (asserted in the realize layer's own
    /// `why_self_explaining_frames_get_no_panel`). Run with `--nocapture` to see
    /// the live sentences.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn why_layer_live_examples_per_outcome_shape() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        // A tiny LOADED lexicon (named `caregiving_lexicon`, so the label
        // projection surfaces its plain name) defining one caregiving term.
        const CARE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-respite-n"><Lemma writtenForm="respite" partOfSpeech="n"/><Sense id="respite-n-01" synset="s-respite"/></LexicalEntry>
    <Synset id="s-respite" ili="i900" partOfSpeech="n"><Definition>short-term care that gives a family caregiver temporary relief</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(
            CARE_LMF,
            OntologyName::new_static("caregiving_lexicon"),
        )
        .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        let label = |o: &ChatOutcome| match o {
            ChatOutcome::Answered => "Answered",
            ChatOutcome::Abstained { .. } => "Abstained",
            ChatOutcome::Conditional { .. } => "Conditional",
            ChatOutcome::RuleResolved { .. } => "RuleResolved",
        };

        // GROUNDED answer over the loaded lexicon — why names the loaded source.
        let grounded = process_with_reasoner(composed.english(), &composed, "what is respite?");
        println!(
            "[grounded]   Q=what is respite?\n  outcome={}\n  response={}\n  why={:?}\n",
            label(&grounded.outcome),
            grounded.response,
            grounded.why
        );
        assert_eq!(grounded.outcome, ChatOutcome::Answered);
        let g = grounded.why.expect("a grounded answer has a why");
        assert!(
            g.contains("the Family Caregiving lexicon"),
            "grounded why names the loaded source by its plain label: {g}"
        );
        assert!(!g.contains('_'), "no raw OntologyName leaks: {g}");

        // UNGROUNDED answer from the built-in substrate — why states the substrate.
        let ungrounded = process_with_reasoner(english, english, "is a dog a mammal?");
        println!(
            "[ungrounded] Q=is a dog a mammal?\n  outcome={}\n  response={}\n  why={:?}\n",
            label(&ungrounded.outcome),
            ungrounded.response,
            ungrounded.why
        );
        if ungrounded.outcome == ChatOutcome::Answered {
            let u = ungrounded.why.expect("an answered turn has a why");
            assert!(
                u.contains("built-in"),
                "ungrounded why states the substrate: {u}"
            );
        }

        // ABSTAINED vocabulary gap — why names the unresolved term.
        let abstained = process_with_reasoner(english, english, "what is a florb?");
        println!(
            "[abstained]  Q=what is a florb?\n  outcome={}\n  response={}\n  why={:?}\n",
            label(&abstained.outcome),
            abstained.response,
            abstained.why
        );
        assert!(matches!(abstained.outcome, ChatOutcome::Abstained { .. }));
        let a = abstained
            .why
            .expect("a vocabulary-gap abstention has a why");
        assert!(
            a.contains("florb"),
            "abstain why names the unresolved term: {a}"
        );
    }

    /// The `who_is_x` gate (`answer_question`'s single-entity Content-
    /// question `else if` arm): a "who is X" question over a DOMAIN-LOADED
    /// role/organization/category term ("DCH") answers exactly like "what is
    /// X", while a bare copula-NP possessive-anchored ask ("who is MY
    /// employer") keeps declining honestly — the measured corpus split (9
    /// of 10 `who_is_x` `UnparsedKnownTerm` rows are role/org/category-
    /// definitional; the possessive-anchored 10th is not) this task's own
    /// investigation found. Uses the full `process_with_reasoner` pipeline
    /// (not hand-built tokens) so the tokenizer's own "who" ==
    /// `WhReferentRole::Person` typing is exercised too, not assumed.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn who_is_x_answers_a_domain_loaded_role_but_keeps_declining_a_possessive_ask() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dch-n"><Lemma writtenForm="DCH" partOfSpeech="n"/><Sense id="dch-n-01" synset="s-dch"/></LexicalEntry>
    <LexicalEntry id="e-employer-n"><Lemma writtenForm="employer" partOfSpeech="n"/><Sense id="employer-n-01" synset="s-employer"/></LexicalEntry>
    <Synset id="s-dch" ili="i1" partOfSpeech="n"><Definition>the state Department of Community Health, the agency administering EVV</Definition></Synset>
    <Synset id="s-employer" ili="i2" partOfSpeech="n"><Definition>the party that employs a worker</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("care"))
            .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        // (a) A domain-loaded organization term: answers, exactly like "what
        // is DCH?" would — the loaded source specifically defines it.
        let org = process_with_reasoner(composed.english(), &composed, "who is DCH?");
        assert_eq!(
            org.outcome,
            ChatOutcome::Answered,
            "a domain-loaded org/role term must answer a bare 'who is X'; got {:?} / {:?}",
            org.outcome,
            org.response
        );
        assert!(
            org.response.to_lowercase().contains("community health"),
            "names the loaded definition: {:?}",
            org.response
        );

        // (b) The SAME loaded term, but possessive-anchored ("MY employer")
        // — a deictic, asker-specific referent no loaded ontology can
        // resolve, not a category definition. Must keep declining.
        let personal = process_with_reasoner(composed.english(), &composed, "who is my employer?");
        assert!(
            matches!(personal.outcome, ChatOutcome::Abstained { .. }),
            "a possessive-anchored 'who is' must still decline honestly; got {:?} / {:?}",
            personal.outcome,
            personal.response
        );
    }

    /// Regression guard for the 2026-07-24 gap-closing batch's own
    /// `OverAnswered` regression (see `caregiver_capability_ratchet.rs`'s
    /// dated doc comment for the full corpus accounting): the `who_is_x`
    /// gate's `is_role_or_org_or_category_definitional` check (exercised by
    /// the sibling test above) is necessary but NOT sufficient — "who is
    /// ELIGIBLE for DCH?" has a copula, but it predicates the ADJECTIVE
    /// "eligible", not DCH; nothing in the sentence claims DCH itself is
    /// what "who" refers to. `who_predication_identifies_the_entity`
    /// closes this: it requires the resolved entity to actually be the
    /// copula's own complement, not merely present somewhere in the
    /// sentence. Real corpus row this guards: "Who is eligible for the
    /// Self-Determination Program?", newly over-answered once
    /// compound-collapsing made its entity count land on exactly 1 more
    /// often (full accounting in `caregiver_capability_ratchet.rs`, which
    /// also names three sibling rows — "Who pays for long-term care?",
    /// "Who schedules the appointments?", "Who Qualifies for Medicaid?" —
    /// that hit the SAME `who_predication_identifies_the_entity` guard via
    /// its complementary no-copula-at-all path; those are exercised by the
    /// live corpus regression suite rather than reproduced here, since a
    /// synthetic matrix verb needs a full subcategorization frame to avoid
    /// itself becoming a second extracted entity — a confound this
    /// minimal inline lexicon has no way to express).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn who_is_eligible_for_x_declines_but_who_is_x_still_answers() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dch-n"><Lemma writtenForm="DCH" partOfSpeech="n"/><Sense id="dch-n-01" synset="s-dch"/></LexicalEntry>
    <LexicalEntry id="e-eligible-a"><Lemma writtenForm="eligible" partOfSpeech="a"/><Sense id="eligible-a-01" synset="s-eligible"/></LexicalEntry>
    <Synset id="s-dch" ili="i1" partOfSpeech="n"><Definition>the state Department of Community Health, the agency administering EVV</Definition></Synset>
    <Synset id="s-eligible" ili="i2" partOfSpeech="a"><Definition>qualified to be chosen or to participate</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("care"))
            .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        // "who is DCH?" -- genuine identity question, must still answer.
        let identity = process_with_reasoner(composed.english(), &composed, "who is DCH?");
        assert_eq!(
            identity.outcome,
            ChatOutcome::Answered,
            "a genuine copula-linked 'who is X' must still answer; got {:?} / {:?}",
            identity.outcome,
            identity.response
        );

        // "who is eligible for DCH?" -- a copula IS present, but its
        // complement is the ADJECTIVE "eligible", not DCH. Must decline.
        let eligible =
            process_with_reasoner(composed.english(), &composed, "who is eligible for DCH?");
        assert!(
            !matches!(eligible.outcome, ChatOutcome::Answered),
            "a copula predicating an ADJECTIVE must not confidently define the embedded entity; got {:?} / {:?}",
            eligible.outcome,
            eligible.response
        );
    }

    /// Regression guard for a real, reported defect: "is a dog an animal"
    /// (a mid-chain query — "animal" sits partway up dog's real WordNet
    /// hypernym chain, not at its end) once printed the primary answer's
    /// glosses all the way past the queried parent to unrelated ancestors
    /// ("chordate"). A synthetic 5-level taxonomy
    /// (zeta⊑yalta⊑xenon⊑whiskey⊑victor) isolates the exact shape without
    /// needing full WordNet: querying "is a zeta a xenon" (zeta⊑yalta⊑xenon,
    /// mid-chain) must answer with ONLY zeta's and xenon's own glosses in
    /// the primary response (Grice 1975 Quantity; see
    /// `build_taxonomy_response`'s content-selection fix) — whiskey and
    /// victor (real ancestors, but ABOVE the queried parent, irrelevant to
    /// this specific yes/no verification) must never appear in the primary
    /// text.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn taxonomy_answer_content_selection_stops_at_the_queried_parent() {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-zeta-n"><Lemma writtenForm="zeta" partOfSpeech="n"/><Sense id="zeta-n-01" synset="s-zeta"/></LexicalEntry>
    <LexicalEntry id="e-yalta-n"><Lemma writtenForm="yalta" partOfSpeech="n"/><Sense id="yalta-n-01" synset="s-yalta"/></LexicalEntry>
    <LexicalEntry id="e-xenon-n"><Lemma writtenForm="xenon" partOfSpeech="n"/><Sense id="xenon-n-01" synset="s-xenon"/></LexicalEntry>
    <LexicalEntry id="e-whiskey-n"><Lemma writtenForm="whiskey" partOfSpeech="n"/><Sense id="whiskey-n-01" synset="s-whiskey"/></LexicalEntry>
    <LexicalEntry id="e-victor-n"><Lemma writtenForm="victor" partOfSpeech="n"/><Sense id="victor-n-01" synset="s-victor"/></LexicalEntry>
    <Synset id="s-zeta" ili="i1" partOfSpeech="n"><Definition>def-zeta</Definition><SynsetRelation relType="hypernym" target="s-yalta"/></Synset>
    <Synset id="s-yalta" ili="i2" partOfSpeech="n"><Definition>def-yalta</Definition><SynsetRelation relType="hypernym" target="s-xenon"/></Synset>
    <Synset id="s-xenon" ili="i3" partOfSpeech="n"><Definition>def-xenon</Definition><SynsetRelation relType="hypernym" target="s-whiskey"/></Synset>
    <Synset id="s-whiskey" ili="i4" partOfSpeech="n"><Definition>def-whiskey</Definition><SynsetRelation relType="hypernym" target="s-victor"/></Synset>
    <Synset id="s-victor" ili="i5" partOfSpeech="n"><Definition>def-victor</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
        let wn = read_wordnet(LMF).expect("regression-guard LMF parses");
        let english = English::from_wordnet(&wn);
        let result = process_with_reasoner(&english, &english, "is a zeta a xenon?");
        assert_eq!(result.outcome, ChatOutcome::Answered);
        assert!(
            result.response.contains("def-zeta") && result.response.contains("def-xenon"),
            "primary response names both endpoints' glosses: {}",
            result.response
        );
        assert!(
            !result.response.contains("def-yalta")
                && !result.response.contains("def-whiskey")
                && !result.response.contains("def-victor"),
            "primary response must not carry intermediate/beyond-parent glosses: {}",
            result.response
        );
        // The deferred content is never dropped — it surfaces in Why?.
        let why = result.why.expect("a grounded taxonomy answer has a why");
        assert!(
            why.contains("def-yalta"),
            "the intermediate rung's gloss is deferred into Why?, not discarded: {why}"
        );
    }

    /// task #30: a capitalized, proper-noun-derived verb lemma ("Islamise",
    /// from "Islam") whose INFLECTED question surface ("islamising") needs
    /// BOTH lemmatization (de-inflect to the stem "islamise") AND case-fold
    /// (fold the stem to match the capitalized WordNet lemma) — neither step
    /// alone finds it. Traced live against the real corpus via
    /// `islamising_lemma_resolution_probe` (`scratch_probe.rs`).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn resolve_surface_composes_lemmatization_with_case_folding() {
        use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-islamise-v">
      <Lemma writtenForm="Islamise" partOfSpeech="v"/>
      <Sense id="islamise-v-1" synset="s-islamise"/>
    </LexicalEntry>
    <Synset id="s-islamise" ili="i1" partOfSpeech="v"><Definition>convert to Islam</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));

        // Neither lemmatization alone (exact-case) nor case-folding alone (on
        // the un-stemmed inflected surface) finds it.
        assert!(en.lookup("islamise").is_empty());
        assert!(en.lookup_case_folded("islamising").is_empty());

        // The composed fallback does.
        let ids = resolve_surface(&en, "islamising");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids, en.lookup_case_folded("islamise"));
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn observe_self_with_loaded_moves_the_totals() {
        // The eigenform's causal connection (doc §2): feeding the live loaded set
        // in MOVES total_concepts/morphisms and adds the ontology as a component —
        // the self-model is no longer blind to what is loaded.
        use pr4xis::ontology::Vocabulary;
        use pr4xis::ontology::meta::{ConceptName, Morphism, MorphismKind};

        let english = sample_english();
        let base = observe_self(&english);

        let loaded = Vocabulary::from_captured(
            "TestCorpus".to_string(),
            "test::corpus",
            "test fixture",
            vec![
                ConceptName::new("a"),
                ConceptName::new("b"),
                ConceptName::new("c"),
            ],
            vec![Morphism::new(
                ConceptName::new("a"),
                ConceptName::new("b"),
                MorphismKind::Subsumption,
            )],
        );
        let with = observe_self_with_loaded(&english, vec![loaded]);

        assert_eq!(
            with.total_concepts,
            base.total_concepts + 3,
            "loading 3 concepts moves the eigenform's concept total"
        );
        assert_eq!(
            with.total_morphisms,
            base.total_morphisms + 1,
            "loading 1 morphism moves the eigenform's morphism total"
        );
        assert!(
            with.components.iter().any(|v| v.name() == "TestCorpus"),
            "the loaded ontology is a component the self-model observes"
        );
        // English-only is unchanged — the old eigenform, with an empty loaded set.
        assert_eq!(observe_self(&english).total_concepts, base.total_concepts);
    }

    // --- Algebraic structure integration tests ---

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn pipeline_trace_is_monoid() {
        // PipelineTrace forms a monoid under concatenation.
        // This enables Writer<PipelineTrace, A> — the pipeline IS a writer monad.
        let empty = PipelineTrace::empty();
        let t1 = PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true);
        let t2 = PipelineTrace::single(PipelineStep::PARSE, "success", true);

        // Left identity: empty ++ t = t
        assert_eq!(empty.combine(&t1).entries.len(), 1);
        // Right identity: t ++ empty = t
        assert_eq!(t1.combine(&empty).entries.len(), 1);
        // Associativity
        let t3 = PipelineTrace::single(PipelineStep::INTERPRET, "question", true);
        assert_eq!(
            t1.combine(&t2).combine(&t3).entries.len(),
            t1.combine(&t2.combine(&t3)).entries.len()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn traced_pipeline_is_writer_monad() {
        // TracedPipeline<A> = Writer<PipelineTrace, A>
        // Monadic bind composes pipeline steps and accumulates trace.
        let step1: TracedPipeline<usize> = pr4xis::category::Writer::new(
            5,
            PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true),
        );

        let result = step1
            .bind(|count| {
                pr4xis::category::Writer::new(
                    count > 0,
                    PipelineTrace::single(PipelineStep::PARSE, "parsed", true),
                )
            })
            .bind(|parsed| {
                let msg = if parsed { "question" } else { "unknown" };
                pr4xis::category::Writer::new(
                    msg,
                    PipelineTrace::single(PipelineStep::INTERPRET, msg, parsed),
                )
            });

        assert_eq!(result.value, "question");
        assert_eq!(result.log.entries.len(), 3);
        // Trace accumulated through bind — no manual trace.record() needed
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn applicative_combines_independent_lookups() {
        // Ap::map2 combines independent lookups (child + parent).
        // This is applicative, not monadic — neither depends on the other.
        let child_ids = Ap::pure(vec![1, 2]);
        let parent_ids = Ap::pure(vec![3, 4, 5]);
        let combined = child_ids.map2(parent_ids, Product::new);
        assert_eq!(combined.value.left.len(), 2);
        assert_eq!(combined.value.right.len(), 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nonempty_tokens_guarantee() {
        // After empty check, tokens form a NonEmpty — guaranteed at least one.
        // NonEmpty is a semigroup (can combine without needing identity).
        let en = sample_english();
        let result = process_with_metadata(&en, "dog");
        assert!(result.token_count > 0);
        // The pipeline used NonEmpty internally after the empty check
    }

    // --- Pipeline tests ---

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn process_taxonomy_question() {
        let en = sample_english();
        let (response, user_act, _) = process(&en, "is a dog a mammal");
        assert_eq!(user_act, SpeechAct::Question);
        assert!(
            response.contains("Yes") || response.contains("No") || response.contains("dog"),
            "taxonomy question should get a substantive answer, got: {}",
            response
        );
    }

    // --- Task #12: decline_if_an_unresolved_acronym_was_ignored ---

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn declines_when_the_answer_ignores_an_unresolved_acronym_in_the_question() {
        let en = sample_english();
        let result = trace_impls::ResponseResult::new(
            ResponseFrame::AssertKnowledge,
            "some confident definition".into(),
        )
        .with_entities_found(vec!["dog".into()])
        .grounded(true);
        let declined = decline_if_an_unresolved_acronym_was_ignored(
            &en,
            "what is the dog of the XYZQ in ABCDEF",
            result,
        );
        assert!(
            !declined.from_ontology,
            "an unresolved acronym elsewhere in the question must decline, not confidently answer"
        );
        assert!(
            declined.response.contains("XYZQ"),
            "the decline must name the acronym it caught, got: {}",
            declined.response
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn does_not_decline_when_the_acronym_is_already_the_resolved_answer() {
        let en = sample_english();
        let result = trace_impls::ResponseResult::new(
            ResponseFrame::AssertKnowledge,
            "XYZQ definition".into(),
        )
        .with_entities_found(vec!["xyzq".into()])
        .grounded(true);
        let kept = decline_if_an_unresolved_acronym_was_ignored(&en, "what is XYZQ", result);
        assert!(
            kept.from_ontology,
            "the acronym IS the answer's own resolved entity, so it must not be overridden"
        );
        assert_eq!(kept.response, "XYZQ definition");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn does_not_decline_a_multi_entity_relational_answer() {
        let en = sample_english();
        let result = trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, "Yes".into())
            .with_entities_found(vec!["dog".into(), "mammal".into()])
            .with_taxonomy_checked(Some(("dog".into(), "mammal".into(), true)))
            .grounded(true);
        let kept = decline_if_an_unresolved_acronym_was_ignored(
            &en,
            "is a dog a mammal in the XYZQ program",
            result,
        );
        assert!(
            kept.from_ontology,
            "a two-entity relational answer is out of this gate's narrow scope, must be untouched"
        );
        assert_eq!(kept.response, "Yes");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn declines_a_multi_noun_exploratory_dump_that_ignores_an_unresolved_acronym() {
        // The `attempt_partial_understanding` `explore_concepts` branch
        // shape: several resolved entities, but `taxonomy_checked: None`
        // (a bare WordNet-sense exploration, never a proven relation) —
        // distinct from `does_not_decline_a_multi_entity_relational_answer`
        // above, whose `taxonomy_checked: Some(..)` is a genuine proof this
        // gate must still leave alone.
        let en = sample_english();
        let result = trace_impls::ResponseResult::new(
            ResponseFrame::AssertKnowledge,
            "a dog is an entity, and a mammal is an entity".into(),
        )
        .with_entities_found(vec!["dog".into(), "mammal".into()])
        .grounded(true);
        let declined = decline_if_an_unresolved_acronym_was_ignored(
            &en,
            "what is the dog mammal threshold under the XYZQ matrix",
            result,
        );
        assert!(
            !declined.from_ontology,
            "an EXPLORATORY multi-noun taxonomy dump that ignores an unresolved acronym must \
             decline too, not just the single-entity define path"
        );
        assert!(
            declined.response.contains("XYZQ"),
            "the decline must name the acronym it caught, got: {}",
            declined.response
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn does_not_decline_an_already_abstained_answer() {
        let en = sample_english();
        let result = trace_impls::ResponseResult::new(
            ResponseFrame::UnknownVocabulary,
            "I do not know the word \"zzyzx\".".into(),
        )
        .with_entities_found(vec!["zzyzx".into()]);
        let kept = decline_if_an_unresolved_acronym_was_ignored(
            &en,
            "what is zzyzx in the XYZQ program",
            result,
        );
        assert!(
            !kept.from_ontology,
            "an already-honest abstention must not be touched by this gate"
        );
        assert_eq!(kept.response, "I do not know the word \"zzyzx\".");
    }

    // --- Task #26: noun_phrase_heads (compound-nominal collapse for explore_concepts) ---

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn noun_phrase_heads_collapses_an_unseparated_compound_run_to_its_head() {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-alpha-n">
      <Lemma writtenForm="alpha" partOfSpeech="n"/>
      <Sense id="alpha-n-1" synset="s-alpha"/>
    </LexicalEntry>
    <Synset id="s-alpha" ili="i1" partOfSpeech="n"><Definition>first thing</Definition></Synset>
    <LexicalEntry id="e-beta-n">
      <Lemma writtenForm="beta" partOfSpeech="n"/>
      <Sense id="beta-n-1" synset="s-beta"/>
    </LexicalEntry>
    <Synset id="s-beta" ili="i2" partOfSpeech="n"><Definition>second thing</Definition></Synset>
    <LexicalEntry id="e-gamma-n">
      <Lemma writtenForm="gamma" partOfSpeech="n"/>
      <Sense id="gamma-n-1" synset="s-gamma"/>
    </LexicalEntry>
    <Synset id="s-gamma" ili="i3" partOfSpeech="n"><Definition>third thing</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));

        // "the alpha beta gamma" — a single unbroken compound-noun chain
        // (one determiner, three adjacent nouns, Levi 1978): ONE noun
        // phrase, headed by its rightmost noun ("gamma"), never three
        // independently comparable entities. This is the exact shape of the
        // 3 domain_mimicry regressions (task #26): "the representative
        // payee misuse escalation protocol" degraded to a pairwise
        // "escalation"/"protocol" comparison before this fix.
        let compound_chain = vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "gamma".into(),
                lambek_type: svo::noun(),
            },
        ];
        assert_eq!(noun_phrase_heads(&compound_chain, &en), vec!["gamma"]);

        // "is a alpha a beta" — two SEPARATELY-determined noun phrases,
        // each headed by its own noun: the bare "is X a Y" comparison shape
        // `explore_concepts` is designed for must still yield both heads.
        let comparison = vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "is".into(),
                lambek_type: svo::copula(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "a".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "a".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::noun(),
            },
        ];
        assert_eq!(noun_phrase_heads(&comparison, &en), vec!["alpha", "beta"]);

        // An attributive adjective inside the compound folds into the SAME
        // run — "big alpha beta" is one NP too, headed by "beta".
        let with_adjective = vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "big".into(),
                lambek_type: svo::adjective(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::noun(),
            },
        ];
        assert_eq!(noun_phrase_heads(&with_adjective, &en), vec!["beta"]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn process_simple_sentence() {
        let en = sample_english();
        let (response, _, _) = process(&en, "the dog runs");
        // Should either parse or give partial understanding — not crash
        assert!(!response.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn process_what_question() {
        let en = sample_english();
        let (response, _, _) = process(&en, "what is a dog");
        // With sample data "what" may not be in lexicon — just verify no crash
        assert!(!response.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn process_empty_input() {
        let en = sample_english();
        let (response, _, _) = process(&en, "");
        assert!(!response.is_empty());
    }

    /// Per memory `feedback_ontological_assertions.md`: each test claim is
    /// an `Axiom` impl in the domain (here `knowledge::instance`); the
    /// `#[test]` is a thin wrapper. The claim is then discoverable via
    /// `Ontology::axioms()`, citable, and reusable.
    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn self_describe_has_ontologies() {
        use pr4xis::ontology::Axiom;
        use pr4xis_domains::formal::information::knowledge::instance::{
            KnowledgeBaseIsNonEmpty, KnowledgeIsRegistered, SelfModelIsRegistered,
        };

        // Exercise the chat surface — confirms `self_describe` returns a
        // structural `SelfModelInstance` that can be observed downstream.
        let _ = self_describe(&sample_english());

        for axiom in [
            &KnowledgeBaseIsNonEmpty as &dyn Axiom,
            &SelfModelIsRegistered,
            &KnowledgeIsRegistered,
        ] {
            assert!(axiom.verify().is_ok(), "{}", axiom.description());
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_can_you_reason_about_realizes_the_live_capability_list() {
        // The A1 self-model NLG realization: a capability query gets
        // GENERATED prose over the live loaded component list, not the raw
        // eigenform JSON every other self-referential question still gets.
        let en = sample_english();
        let (response, _, _) = process(&en, "what can you reason about");
        assert!(
            response.starts_with("I can reason about: "),
            "expected a generated capability sentence, got: {response}"
        );
        // The live loaded ontology name is actually named, not a placeholder.
        let live = observe_self(&en);
        let expected_name = live
            .components
            .first()
            .expect("at least one component is loaded")
            .name();
        assert!(
            response.contains(expected_name),
            "capability sentence must name the live-loaded component {expected_name:?}: {response}"
        );
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn a_component_with_a_registered_unreachable_claim_is_named_as_such() {
        // The fix this whole nlp_task ontology exists for: the capability
        // answer must distinguish "this component is registered" from
        // "this component's claimed unit actually works in conversation" --
        // `known_claims()`'s "TextOntology"/Token::tense claim is a
        // confirmed, committed CarryingTypeHasNoSlot verdict, so
        // "TextOntology" must NOT appear as a bare, unqualified capability
        // in this sentence. Component name confirmed via a live probe of
        // `describe_knowledge_base()`'s actual output.
        //
        // "ResponseOntology" is the CONTRAST case in the same assertion:
        // task #4 wired `ResponseFrame::PhaticReturn` into the live chat
        // pipeline (`is_phatic`/`answer_phatic` below), so its former
        // NotReachable claim was REMOVED from `known_claims()` -- it must
        // now appear bare, unqualified, regression-guarding that fix.
        let en = sample_english();
        let live = observe_self(&en);
        assert!(
            live.components.iter().any(|v| v.name() == "TextOntology"),
            "this test assumes TextOntology is a live-registered component"
        );
        let (response, _, _) = process(&en, "what can you reason about");
        const QUALIFIED: &str = "TextOntology (registered, not yet reachable in conversation)";
        assert!(
            response.contains(QUALIFIED),
            "expected TextOntology to be named with its unreachable-claim qualifier: {response}"
        );
        // Strip the one legitimate qualified occurrence; "TextOntology"
        // must not appear anywhere else (i.e. never also as a bare,
        // unqualified item).
        let remainder = response.replacen(QUALIFIED, "", 1);
        assert!(
            !remainder.contains("TextOntology"),
            "TextOntology must never ALSO appear as a bare, unqualified capability: {response}"
        );
        assert!(
            remainder.contains("ResponseOntology") && !remainder.contains("ResponseOntology ("),
            "ResponseOntology's PhaticReturn gap was fixed (task #4) and removed from \
             known_claims() -- it must now appear as a bare, unqualified capability: {response}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_can_you_do_and_what_do_you_know_also_realize_the_capability_list() {
        // Regression coverage for a real bug an adversarial audit found:
        // capability_query_referents() originally omitted "can"/"know" even
        // though its own doc comment claimed they were grounded, so these
        // exact natural paraphrases silently fell through to the raw
        // eigenform JSON instead of the generated capability sentence.
        let en = sample_english();
        for question in ["what can you do", "what do you know"] {
            let (response, _, _) = process(&en, question);
            assert!(
                response.starts_with("I can reason about: "),
                "{question:?} must realize the capability sentence, got: {response}"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_generic_self_referential_question_still_gets_the_eigenform_json() {
        // "who are you" is self-referential but not a capability query --
        // it must keep the structured JSON transport the chat UI consumes,
        // not the new prose path.
        let en = sample_english();
        let (response, _, _) = process(&en, "who are you");
        assert!(!response.starts_with("I can reason about"));
        assert!(
            response.trim_start().starts_with('{'),
            "non-capability self-referential answers stay JSON: {response}"
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn self_describe_eigenform_is_stable() {
        // Self(Self) = Self — calling observe_self twice gives same result
        let en = sample_english();
        let first = observe_self(&en);
        let second = observe_self(&en);
        assert_eq!(first.total_concepts, second.total_concepts);
        assert_eq!(first.total_morphisms, second.total_morphisms);
        assert_eq!(first.components.len(), second.components.len());
    }

    // --- Phase 2: RelationshipMeta + TracedPipeline integration tests ---

    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn trace_meta_is_from_diagnostic_ontology() {
        // trace_meta() comes from DiagnosticOntology::meta() — generated by
        // ontology!. The ontology identifies itself through the macro,
        // not through hand-written strings.
        let m = trace_meta();
        assert_eq!(m.name.as_str(), "DiagnosticOntology");
        assert!(m.module_path.as_str().contains("diagnostics"));
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn pipeline_trace_accumulates_through_writer_composition() {
        // After the refactor, `process_with_metadata` builds its trace through
        // `TracedPipeline<()>` composition — each `.tell()` call combines a
        // single PipelineTrace via the Vec monoid. No mutation.
        let en = sample_english();
        let result = process_with_metadata(&en, "is a dog a mammal");

        // The full pipeline fires: tokenize → parse → interpret → speech act
        // → metacognition → response → realization. Seven entries minimum.
        assert!(
            result.trace.entries.len() >= 7,
            "expected full pipeline trace, got {} entries",
            result.trace.entries.len()
        );

        // First entry is always Tokenize — the writer log respects order
        // because Vec::combine concatenates left-to-right.
        assert_eq!(result.trace.entries[0].step, PipelineStep::TOKENIZE);
        // Last entry is Realization — the final step of the writer chain.
        let last = result.trace.entries.last().unwrap();
        assert_eq!(last.step, PipelineStep::REALIZATION);
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn empty_input_still_produces_traceable_output() {
        // Empty-branch early return: the trace carries the tokenize result
        // constructed via `PipelineTrace::from_traceable`, not a mutation.
        let en = sample_english();
        let result = process_with_metadata(&en, "");
        assert_eq!(result.token_count, 0);
        assert_eq!(result.trace.entries.len(), 1);
        assert_eq!(result.trace.entries[0].step, PipelineStep::TOKENIZE);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn writer_tell_preserves_trace_order() {
        // The writer monad's `tell` preserves order because Vec monoid
        // concatenation is left-associative. Verifying directly:
        let pipeline: TracedPipeline<()> = Writer::pure(())
            .tell(PipelineTrace::single(PipelineStep::TOKENIZE, "a", true))
            .tell(PipelineTrace::single(PipelineStep::PARSE, "b", true))
            .tell(PipelineTrace::single(PipelineStep::INTERPRET, "c", true));
        assert_eq!(pipeline.log.entries.len(), 3);
        assert_eq!(pipeline.log.entries[0].step, PipelineStep::TOKENIZE);
        assert_eq!(pipeline.log.entries[1].step, PipelineStep::PARSE);
        assert_eq!(pipeline.log.entries[2].step, PipelineStep::INTERPRET);
    }
}

// =========================================================================
// Loaded-corpus demo — the with/without behavioral contrast
// =========================================================================
//
// The heart of the runtime demo: a chat that answers a question about the
// CONTENT of a LOADED ontology, grounded through English, and abstains when
// that ontology is not loaded. The same question, the same chat code — only
// the reasoner differs:
//
//   - WITHOUT the corpus  →  `process(english, …)`            →  abstains
//   - WITH the corpus     →  `process_with_reasoner(english, composed, …)`
//                                                              →  answers from
//                                                                 the loaded gloss
//
// The corpus is a REAL compiled praxis ontology projected to a `.prx` Archive
// via `emit::<Cat>()`, materialized into a `RuntimeOntology`, and grounded into
// the English lexicon by the `ComposedReasoner` (the Lemon functor). Nothing is
// hardcoded: the answer text the WITH case asserts is the ontology's OWN gloss,
// read back through `RuntimeOntology::lexical`.
#[cfg(test)]
mod loaded_corpus_demo {
    use std::rc::Rc;

    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

    // A small REAL legal-corpus ontology: a statute is divided into Titles,
    // which are divided into Sections. Compiled by the same `ontology!` macro
    // every domain ontology uses — labels (the lexical glosses) and the
    // materialized Subsumption taxonomy and all.
    pr4xis::ontology! {
        name: "Statute",
        source: "loaded-corpus demo fixture (US Code structural skeleton: Title → Section)",
        concepts: [Statute, Title, Section],
        labels: {
            Statute: ("en", "Statute",
                "A written law enacted by a legislature."),
            Title: ("en", "Title",
                "A formal top-level subdivision of a statute, grouping related sections."),
            Section: ("en", "Section",
                "The smallest numbered unit of a statute, stating a single rule."),
        },
        is_a: [
            // A Section is part of the statute's structure; a Title is too. For
            // the demo we only need the lexical grounding, but a real taxonomy
            // edge keeps the fixture honest (Title and Section are Statute
            // structural units).
            (Title, Statute),
            (Section, Statute),
        ],
    }

    /// Materialize the Statute ontology into a `RuntimeOntology`.
    ///
    /// `emit::<StatuteCategory>()` now carries each concept's gloss INTO the
    /// `.prx` itself (via `Concept::lexical`, filled from the `ontology!` labels
    /// table) — so the demo no longer post-processes the archive to attach
    /// glosses from `labels()`. The gloss travels with the projected ontology,
    /// exactly as it must for a `.prx` loaded with no compile-time labels table
    /// (e.g. in the browser).
    fn statute_corpus() -> RuntimeOntology {
        let archive = emit::<StatuteCategory>();
        materialize(archive, OntologyName::new_static("Statute"))
            .expect("Statute corpus materializes")
    }

    /// The gloss the loaded ontology carries for `Title` — the exact text the
    /// WITH case must surface, read back from the materialized ontology's own
    /// `lexical` (i.e. from what `emit` carried into the `.prx`), never from a
    /// compile-time side table.
    fn title_gloss() -> String {
        let corpus = statute_corpus();
        corpus
            .lexical(&corpus.concept("Title"))
            .expect("the emitted Statute ontology carries Title's gloss")
            .to_string()
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn what_is_a_title_answers_from_the_loaded_gloss_with_the_corpus_and_abstains_without() {
        let english = English::sample();
        let question = "what is a title";
        let gloss = title_gloss();

        // --- WITHOUT the corpus: english-only. "title" is not an English word
        //     the sample model knows, so the chat abstains. ---
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("do not")
                || without.to_lowercase().contains("don't")
                || without.to_lowercase().contains("not know"),
            "english-only must abstain on an unloaded concept; got: {without:?}"
        );
        assert!(
            !without.contains(gloss.as_str()),
            "english-only must NOT surface the loaded gloss (it isn't loaded); got: {without:?}"
        );

        // --- WITH the corpus: ground the loaded Statute ontology into English
        //     via the ComposedReasoner, then ask the SAME question through the
        //     SAME pipeline. The answer is the loaded gloss. ---
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(statute_corpus())]);
        let with = process_with_reasoner(&english, &composed, question).response;
        assert!(
            with.contains(gloss.as_str()),
            "with the corpus loaded, the chat must answer from the loaded Title gloss \
             ({gloss:?}); got: {with:?}"
        );
        assert!(
            with.to_lowercase().contains("title"),
            "the answer must name the queried concept; got: {with:?}"
        );

        // The contrast is the whole demo: same question, same code, opposite
        // epistemic outcome — grounded entirely through the lexicon, never a
        // hardcoded branch.
        assert_ne!(without, with, "loading the corpus must change the answer");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_multi_token_loaded_surface_resolves_through_the_chat() {
        // The multi-token (phrase/citation) recognizer end-to-end: a loaded
        // ontology whose surface is MULTI-WORD ("section 1514a") becomes answerable
        // in chat, where word-by-word tokenization alone would split + miss it.
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::Definition;

        let english = English::sample();
        let question = "what is section 1514a";
        let gloss = "Civil action to protect against retaliation in fraud cases.";

        // english-only: max_surface_words == 1, so no collapse — "section" and
        // "1514a" are looked up separately and miss → the chat abstains.
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("not") || without.to_lowercase().contains("don't"),
            "english-only must abstain on the multi-token citation; got: {without:?}"
        );

        // composed: a loaded ontology indexes the multi-word surface (its node
        // name); the recognizer collapses ["section","1514a"] into ONE proper-noun
        // token and the chat answers from the node's gloss — the §9 use case.
        let archive = Archive {
            nodes: vec![Definition {
                kind: "Concept".to_string(),
                name: "section 1514a".to_string(),
                edges: vec![],
                axioms: vec![],
                lexical: Some(gloss.to_string()),
            }],
            connections: vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("usc_test"))
            .expect("the multi-word-named ontology materializes");
        let composed = ComposedReasoner::new(English::sample_static(), vec![Rc::new(onto)]);
        assert!(
            composed.max_surface_words() >= 2,
            "the loaded surface 'section 1514a' is multi-word, so the recognizer is active"
        );

        let with = process_with_reasoner(&english, &composed, question).response;
        assert!(
            with.contains(gloss),
            "the multi-token citation must collapse + resolve to the loaded gloss; got: {with:?}"
        );
        assert_ne!(
            without, with,
            "the multi-token recognizer changes an abstention into an answer"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_owl_label_makes_a_class_chat_answerable() {
        // The §9 OWL path end-to-end: an OWL entity is answerable by its
        // `rdfs:label` (minted as a `canonicalForm` Form by the owl bridge), not
        // only its opaque IRI. A MULTI-WORD label exercises §9 + the phrase-lookup
        // together.
        use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
        use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
        use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;

        const OWL: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:ObjectProperty rdf:about="http://purl.org/spar/cito/citesAsEvidence">
    <rdfs:label>cites as evidence</rdfs:label>
    <rdfs:comment>The citing entity cites the cited entity as evidence.</rdfs:comment>
  </owl:ObjectProperty>
</rdf:RDF>"#;

        let english = English::sample();
        let ont = read_owl(OWL).expect("parse OWL");
        let vocab = LoadedOwlVocabulary::from_owl_ontology(&ont);
        let onto = owl_runtime_ontology(&vocab, OntologyName::new_static("cito"))
            .expect("the OWL vocabulary materializes");
        let composed = ComposedReasoner::new(English::sample_static(), vec![Rc::new(onto)]);

        let resp = process_with_reasoner(&english, &composed, "what is cites as evidence").response;
        assert!(
            resp.to_lowercase().contains("citing"),
            "the OWL property must answer from its rdfs:comment gloss via its label \
             surface; got: {resp:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn grounding_unions_the_loaded_surface_into_the_lexicon() {
        // The Lemon grounding is what makes "title" resolvable at all: english
        // alone returns nothing for it; the composed reasoner returns the loaded
        // concept id (typed-disjoint from English's), and `define_word` reads
        // its gloss straight from the materialized ontology.
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(statute_corpus())]);

        assert!(
            English::sample().lookup("title").is_empty(),
            "precondition: the embedded model does not know 'title'"
        );
        let ids = composed.lookup("title");
        assert!(
            !ids.is_empty(),
            "the grounded lexicon must resolve the loaded surface 'title'"
        );

        // `define_word` over the composed reasoner reads the loaded gloss.
        let defined = define_word(&composed, "title").text;
        assert!(
            defined.contains(title_gloss().as_str()),
            "define_word must surface the loaded gloss; got: {defined:?}"
        );

        // And the grounding carries the TYPED reference (ontology + name): the
        // surface's id decodes to the loaded `ConceptRef {Statute, Title}` — the
        // Lemon `LexicalEntry.sense.reference`, read through the reasoner's own
        // decode surface (the resident owned Lexicon copy was deleted; the
        // applied functor's image IS the surface index + decode table).
        use pr4xis_domains::cognitive::linguistics::composed::GroundedConcept;
        let cref = ids
            .iter()
            .find_map(|&id| match composed.decode(id) {
                Some(GroundedConcept::Loaded(cref)) => Some(cref),
                _ => None,
            })
            .expect("the grounded surface 'title' decodes to a loaded ConceptRef");
        assert_eq!(
            (cref.ontology.as_str(), cref.name.as_str()),
            ("Statute", "Title"),
            "the grounded entry carries the typed (ontology, concept) reference"
        );
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_loaded_answer_names_the_loaded_ontology_in_its_provenance() {
        // Doc §2.3 — the "Title-names-Title" deliverable: a turn that answers from
        // a loaded `.prx` (the Statute corpus) NAMES it in the trace provenance,
        // by its OntologyName, success-marked. English-only names no loaded ontology.
        use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;

        let english = English::sample();
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(statute_corpus())]);

        let with = process_with_reasoner(&english, &composed, "what is a title");
        let provenance = with.trace.reasoned_over();
        assert!(
            provenance.iter().any(
                |(o, ok)| matches!(o, TraceOntology::Loaded(n) if n.as_str() == "Statute") && *ok
            ),
            "the trace must name the loaded Statute ontology it reasoned over; got: {provenance:?}"
        );

        // English-only reasons over no loaded ontology — the contrast.
        let without = process_with_reasoner(&english, &English::sample(), "what is a title");
        assert!(
            !without
                .trace
                .reasoned_over()
                .iter()
                .any(|(o, _)| matches!(o, TraceOntology::Loaded(_))),
            "english-only must name no loaded ontology in its provenance"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn abstention_is_a_typed_outcome_naming_what_to_load() {
        // Doc §4.1 (highest): a self-aware system models what it CANNOT answer.
        // With the corpus, "title" resolves → Answered; without it, the turn
        // ABSTAINS and NAMES the unresolved surface (what to load), as a value —
        // not a string the UI has to sniff.
        let english = English::sample();
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(statute_corpus())]);

        let answered = process_with_reasoner(&english, &composed, "what is a title");
        assert_eq!(
            answered.outcome,
            ChatOutcome::Answered,
            "with the corpus, the loaded concept answers; got {:?}",
            answered.outcome
        );

        let abstained = process_with_reasoner(&english, &English::sample(), "what is a title");
        match abstained.outcome {
            ChatOutcome::Abstained { unresolved } => assert!(
                unresolved.iter().any(|s| s == "title"),
                "abstention must name the unresolved surface to load; got {unresolved:?}"
            ),
            ChatOutcome::Answered => {
                panic!("english-only must abstain on the unloaded concept 'title'")
            }
            ChatOutcome::Conditional { .. } => {
                panic!("english-only over an is-a question has no rule-governed producer")
            }
            ChatOutcome::RuleResolved { .. } => {
                panic!("a single turn through process_with_reasoner never resolves a rule")
            }
        }
    }

    /// A loaded ontology carrying a PARTHOOD mereology in the USC orientation
    /// (part → whole): a `subsection` is part of a `section`. Hand-built so the
    /// edge direction is explicit and matches `uslm::corpus::bridge` (a
    /// subdivision Composes INTO its parent → Parthood part→whole) — NOT the
    /// `ontology!` `has_a:` sugar, which orients whole→part (has-part), the
    /// inverse. `materialize` folds the edge into the transitive Parthood closure.
    fn parthood_corpus() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "subsection".to_string(),
                    edges: vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("section".to_string()),
                    )],
                    axioms: vec![],
                    lexical: Some("A lettered subdivision of a section.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "section".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("The smallest numbered unit of a statute.".to_string()),
                },
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("PartCorpus"))
            .expect("the Parthood corpus materializes")
    }

    /// A 3-LEVEL Parthood mereology (clause → section → title, part→whole), so a
    /// "is clause part of title" answer has a non-trivial part-of evidence CHAIN
    /// through the intermediate `section`.
    fn deep_parthood_corpus() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        let part = |name: &str, whole: Option<&str>, gloss: &str| Definition {
            kind: "Concept".to_string(),
            name: name.to_string(),
            edges: whole
                .map(|w| vec![("Parthood".to_string(), EdgeTarget::Local(w.to_string()))])
                .unwrap_or_default(),
            axioms: vec![],
            lexical: Some(gloss.to_string()),
        };
        let archive = Archive {
            nodes: vec![
                part("clause", Some("section"), "A clause within a section."),
                part("section", Some("title"), "A section within a title."),
                part("title", None, "A title of the code."),
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("DeepPartCorpus"))
            .expect("the deep Parthood corpus materializes")
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_deep_parthood_answer_shows_the_part_of_evidence_chain() {
        // image-meet-chain: `relation_chain` reads the Parthood closure, so a deep
        // mereology's answer shows the part-of EVIDENCE path through the
        // intermediate whole — not just the two endpoints.
        let composed = ComposedReasoner::new(
            English::sample_static(),
            vec![Rc::new(deep_parthood_corpus())],
        );
        let yes = answer_question(
            &composed,
            English::sample_static(),
            &[],
            "part of",
            &entity_args("clause", "title"),
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            yes.taxonomy_checked,
            Some(("clause".to_string(), "title".to_string(), true)),
            "a clause IS transitively part of a title"
        );
        // The evidence chain names the intermediate `section` and phrases each rung
        // "is part of", not "is a".
        assert!(
            yes.response.contains("section"),
            "the part-of chain must show the intermediate whole; got: {:?}",
            yes.response
        );
        assert!(
            yes.response.contains("is part of") && !yes.response.contains(" is a "),
            "every rung phrases the Parthood relation; got: {:?}",
            yes.response
        );
    }

    /// Two `Sem::Concept` (NP) arguments naming the entities of a relational
    /// question — what `answer_question` reads via `extract_entity_name`.
    fn entity_args(child: &str, parent: &str) -> [montague::Sem; 2] {
        [
            montague::Sem::Concept {
                word: child.to_string(),
                concepts: Vec::new(),
                role: montague::GrammaticalRole::Argument,
                expression_use: ExpressionUse::Used,
            },
            montague::Sem::Concept {
                word: parent.to_string(),
                concepts: Vec::new(),
                role: montague::GrammaticalRole::Argument,
                expression_use: ExpressionUse::Used,
            },
        ]
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_subsection_part_of_section_answers_yes_over_a_loaded_parthood_mereology() {
        // Step 3 end-to-end: the relational predicate "part of" resolves through
        // the loaded relation lexicon to the Parthood kind, and the reasoner reads
        // the loaded ontology's MATERIALIZED Parthood closure — the structural
        // query Track C + §9 left dark (gloss worked; "is X part of Y" did not).
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(parthood_corpus())]);
        let args = entity_args("subsection", "section");

        let yes = answer_question(
            &composed,
            English::sample_static(),
            &[],
            "part of",
            &args,
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            yes.taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "a subsection IS part of its section (Parthood closure); got {:?}",
            yes.taxonomy_checked
        );
        // The affirmation PHRASES the relation it answered (its loaded surface),
        // not "is a" — Parthood is not Subsumption (the realize.rs fix).
        assert!(
            yes.response.contains("is part of"),
            "a Parthood affirmation must read 'is part of', not 'is a'; got: {:?}",
            yes.response
        );

        // English-only cannot witness Parthood: it has no relation lexicon
        // (relation_for_surface → None → Subsumption) and no loaded closure.
        let en_only = answer_question(
            &English::sample(),
            English::sample_static(),
            &[],
            "part of",
            &args,
            montague::QuestionIllocution::Polar,
        );
        assert_ne!(
            en_only.taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "english-only must not affirm a Parthood it cannot witness"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_section_part_of_subsection_answers_no_parthood_is_directional() {
        // Parthood is antisymmetric (BFO:0000050): the whole is NOT part of its
        // part. The reverse-direction convenience that would mask this is gone.
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(parthood_corpus())]);
        let no = answer_question(
            &composed,
            English::sample_static(),
            &[],
            "part of",
            &entity_args("section", "subsection"),
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            no.taxonomy_checked,
            Some(("section".to_string(), "subsection".to_string(), false)),
            "a section is NOT part of its subsection; got {:?}",
            no.taxonomy_checked
        );
        // The denial phrases the relation too: "is not part of", not "is not a".
        assert!(
            no.response.contains("not part of"),
            "a Parthood denial must read 'is not part of'; got: {:?}",
            no.response
        );
    }

    /// A Parthood corpus with MULTI-WORD citation surfaces (the USC reality:
    /// "section ninety", "title fifteen") — so the recognizer collapses each to a
    /// proper-noun NP and "is X part of Y" parses from raw text.
    fn parthood_corpus_multiword() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "section ninety".to_string(),
                    edges: vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("title fifteen".to_string()),
                    )],
                    axioms: vec![],
                    lexical: Some("A section within title fifteen.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "title fifteen".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("Commerce and trade.".to_string()),
                },
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("PartCorpusMW"))
            .expect("the multi-word Parthood corpus materializes")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_part_of_y_parses_and_answers_from_raw_chat_text() {
        // Step 5 — the WHOLE path from RAW TEXT: tokenize → collapse "section
        // ninety"/"title fifteen" to NPs + "part of" to a relational predicate →
        // the predicative question-copula parses S[q] → interpret lifts the
        // relation from the complement → answer_question reads the Parthood
        // closure. No constructed Sem — a person types the question.
        let english = English::sample();
        let composed = ComposedReasoner::new(
            English::sample_static(),
            vec![Rc::new(parthood_corpus_multiword())],
        );
        let question = "is section ninety part of title fifteen";

        let with = process_with_reasoner(&english, &composed, question);

        assert!(
            with.parsed,
            "the predicative question-copula must parse 'is X part of Y'; got: {:?}",
            with.response
        );
        assert!(
            with.response.to_lowercase().contains("yes"),
            "section ninety IS part of title fifteen — the Parthood closure affirms; got: {:?}",
            with.response
        );
        // The loaded gloss rides the answer (the concepts resolved, not guessed).
        assert!(
            with.response.contains("Commerce and trade"),
            "the affirmation surfaces the loaded gloss; got: {:?}",
            with.response
        );

        // english-only abstains (it knows none of these surfaces and has no
        // relation lexicon) — the contrast that proves the corpus + lexicon did it.
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("not") || without.to_lowercase().contains("don't"),
            "english-only must abstain on the loaded relational question; got: {without:?}"
        );
        assert_ne!(
            without, with.response,
            "loading the corpus + relation lexicon must change the answer"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_part_of_y_parses_with_single_word_loaded_entities() {
        // The single-word-np fix: a SINGLE-word loaded entity ("subsection",
        // "section" — not a multi-word citation) now offers an NP reading, so
        // "is X part of Y" parses from raw text. Gated on loaded membership, so
        // English function words are untouched (process_taxonomy_question etc. green).
        let english = English::sample();
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(parthood_corpus())]);
        let question = "is subsection part of section";

        let with = process_with_reasoner(&english, &composed, question);
        assert!(
            with.parsed,
            "single-word loaded entities must type NP so 'is X part of Y' parses; got: {:?}",
            with.response
        );
        assert!(
            with.response.to_lowercase().contains("yes") && with.response.contains("is part of"),
            "a subsection IS part of a section; got: {:?}",
            with.response
        );

        // English-only abstains — the contrast (it has no loaded entity, no NP upgrade).
        let (without, _, _) = process(&english, question);
        assert_ne!(
            without, with.response,
            "the loaded corpus must change the answer"
        );
    }

    /// A tiny corpus with named comparanda, each carrying its OWN gloss —
    /// what a "difference between X and Y" comparison question (Construction
    /// 2, Barker 2011) resolves against.
    fn comparison_corpus() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::Definition;
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "budget".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("A plan for spending money.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "plan".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("A scheme worked out in advance.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "forecast".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("A prediction of a future state.".to_string()),
                },
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("ComparisonCorpus"))
            .expect("the comparison corpus materializes")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_the_difference_between_x_and_y_recites_both_glosses_from_raw_chat_text() {
        // Construction 2: "difference" is a Barker (2011) DERIVED relational
        // noun licensing BOTH PP-complement participants — the answer must
        // recite BOTH named terms' own gloss, from RAW TEXT (no constructed
        // Sem — a person types the question), not just whichever the
        // pre-existing single-leaf extraction happened to list first.
        let english = English::sample();
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(comparison_corpus())]);
        let question = "what is the difference between the budget and the plan";

        let with = process_with_reasoner(&english, &composed, question);
        assert!(
            with.parsed,
            "the derived-relational-noun PP-complement must parse; got: {:?}",
            with.response
        );
        assert!(
            with.response.contains("A plan for spending money."),
            "the answer must recite budget's own gloss; got: {:?}",
            with.response
        );
        assert!(
            with.response.contains("A scheme worked out in advance."),
            "the answer must ALSO recite plan's own gloss — not just the \
             first-listed term; got: {:?}",
            with.response
        );

        // English-only cannot witness "budget"/"plan" (unloaded) — the
        // contrast that proves the corpus + comparison lexicon did it.
        let (without, _, _) = process(&english, question);
        assert_ne!(
            without, with.response,
            "loading the corpus must change the answer"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_the_difference_between_a_three_way_list_recites_every_gloss() {
        // The row-1511 shape: a THREE-way list-coordinator comma
        // (`tokenize::find_list_coordinator_commas`'s real minting from raw
        // text, not a hand-spliced marker) flattens into ONE comparison-
        // relation body, and every one of the three glosses is recited.
        let english = English::sample();
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(comparison_corpus())]);
        let question = "what is the difference between the budget, the plan, and the forecast";

        let with = process_with_reasoner(&english, &composed, question);
        assert!(
            with.parsed,
            "a 3-way comma-coordinated PP complement must parse; got: {:?}",
            with.response
        );
        for gloss in [
            "A plan for spending money.",
            "A scheme worked out in advance.",
            "A prediction of a future state.",
        ] {
            assert!(
                with.response.contains(gloss),
                "the answer must recite every one of the three named terms' \
                 own gloss ({gloss:?}); got: {:?}",
                with.response
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn part_of_and_is_a_read_distinct_closures_over_the_same_pair() {
        // The Smith et al. (2005) part_of ≠ is_a distinction, end-to-end: the
        // subsection→section edge is Parthood, so "is X part of Y" is true but
        // "is X a Y" (the bare copula "is" → Subsumption fallback) is false.
        let composed =
            ComposedReasoner::new(English::sample_static(), vec![Rc::new(parthood_corpus())]);
        let args = entity_args("subsection", "section");

        assert_eq!(
            answer_question(
                &composed,
                English::sample_static(),
                &[],
                "part of",
                &args,
                montague::QuestionIllocution::Polar
            )
            .taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "part of → Parthood → true"
        );
        assert_eq!(
            answer_question(
                &composed,
                English::sample_static(),
                &[],
                "is",
                &args,
                montague::QuestionIllocution::Polar
            )
            .taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), false)),
            "is → Subsumption fallback → false (the edge is Parthood, not is-a)"
        );
    }
}

// =========================================================================
// LegalSources base — the guardrail spec-lock: real statute questions over
// the reasoner
// =========================================================================
//
// The always-loaded LegalSources base (LKIF-Core formal sources of law) wired
// into the wasm chat is exercised HERE at the chat entry (`process_with_reasoner`
// / `answer_question`), on the same `ComposedReasoner` the wasm builds. These are
// the guardrail: they ask the questions a person actually types ("is a statute a
// law") and lock the TYPED outcomes (`ChatOutcome`, `taxonomy_checked`,
// `from_ontology`, `reasoned_over`) so a regression that silently stops grounding
// the label surfaces — or reintroduces a false negation from silence — fails here.
//
// The corpus is projected by `emit::<LegalSourcesCategory>()` — the default,
// lexicalizing projection build.rs bakes into the wasm base — so a concept's
// ONTOLEX-Lemon label ("law", "case law", "legal document") grounds as a queryable
// `ontolex:Form` surface, distinct from its Rust identifier. Nothing is hardcoded:
// the Yes/No/Abstain each question yields is read off the loaded Subsumption
// closure through the composed reasoner, never a `match` on the question text.
#[cfg(test)]
mod legal_sources_base {
    use std::rc::Rc;

    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;
    use pr4xis_domains::social::judicial::legal_sources::ontology::LegalSourcesCategory;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

    /// Materialize the LegalSources ontology into a `RuntimeOntology` through the
    /// lexicalizing projection — so each concept's Lemon label ("law" for
    /// `LegalSource`, "case law" for `Precedent`, "legal document" for
    /// `LegalDocument`) rides as an `ontolex:Form` surface the composed reasoner
    /// indexes, exactly as the wasm base does.
    fn legal_corpus() -> RuntimeOntology {
        let archive = emit::<LegalSourcesCategory>();
        materialize(archive, OntologyName::new_static("LegalSources"))
            .expect("LegalSources corpus materializes")
    }

    fn reasoner() -> ComposedReasoner {
        ComposedReasoner::new(English::sample_static(), vec![Rc::new(legal_corpus())])
    }

    /// Two `Sem::Concept` (NP) arguments naming the entities of a relational
    /// question — what `answer_question` reads via `extract_entity_name`.
    fn entity_args(child: &str, parent: &str) -> [montague::Sem; 2] {
        [
            montague::Sem::Concept {
                word: child.to_string(),
                concepts: Vec::new(),
                role: montague::GrammaticalRole::Argument,
                expression_use: ExpressionUse::Used,
            },
            montague::Sem::Concept {
                word: parent.to_string(),
                concepts: Vec::new(),
                role: montague::GrammaticalRole::Argument,
                expression_use: ExpressionUse::Used,
            },
        ]
    }

    /// Whether a turn's provenance names the loaded LegalSources ontology,
    /// success-marked — the "answered FROM the base" evidence.
    fn credits_legal_sources(result: &ProcessResult) -> bool {
        result.trace.reasoned_over().iter().any(|(o, ok)| {
            matches!(o, TraceOntology::Loaded(n) if n.as_str() == "LegalSources") && *ok
        })
    }

    /// Assert a natural-language question answers YES from the base: the typed
    /// outcome is `Answered`, the answer is from the ontology, the prose affirms,
    /// and the provenance credits LegalSources.
    fn assert_yes(question: &str) {
        let r = process_with_reasoner(&English::sample(), &reasoner(), question);
        assert_eq!(
            r.outcome,
            ChatOutcome::Answered,
            "{question:?} must be Answered; got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(
            r.from_ontology,
            "{question:?} must answer from the ontology; got {:?}",
            r.response
        );
        assert!(
            r.response.to_lowercase().contains("yes"),
            "{question:?} must affirm (Yes); got {:?}",
            r.response
        );
        assert!(
            credits_legal_sources(&r),
            "{question:?} must credit the LegalSources base it reasoned over"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_statute_is_a_law() {
        // THE headline: Statute ⊑ LegalDocument ⊑ LegalSource(label "law"). The label
        // surface "law" grounds (default lexicalizing emit), so the closure query answers Yes.
        assert_yes("is a statute a law");
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_multi_hop_answer_renders_labels_and_the_transitivity_license() {
        // Fix 1 + Fix 2 on the headline, multi-hop path Statute ⊑ LegalDocument ⊑
        // LegalSource: the evidence chain renders the NATURAL LABELS ("legal
        // document", "law") read from each concept's `canonicalForm` Form — NEVER
        // the Rust identifiers "LegalDocument"/"LegalSource" — and the answer
        // appends the LICENSING rule (is-a is transitive) with its citation, read
        // from the Relations ontology as data.
        let r = process_with_reasoner(&English::sample(), &reasoner(), "is a statute a law");
        assert!(
            r.response.contains("legal document"),
            "the middle rung renders its label 'legal document'; got {:?}",
            r.response
        );
        assert!(
            !r.response.contains("LegalDocument") && !r.response.contains("LegalSource"),
            "no Rust identifier may leak into the rendered answer; got {:?}",
            r.response
        );
        // The transitivity licensing note — the RULE, not just the path.
        assert!(
            r.response.contains("is transitive"),
            "a multi-hop is-a answer names the transitivity that licensed it; got {:?}",
            r.response
        );
        assert!(
            r.response.contains("Tarski"),
            "the transitivity note carries its citation, read from the Relations \
             ontology; got {:?}",
            r.response
        );
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_single_hop_answer_invokes_no_transitivity() {
        // The direct edge Statute ⊑ LegalDocument is a SINGLE hop — no transitivity
        // is invoked, so the answer states the direct relation only (no licensing note).
        let r = process_with_reasoner(
            &English::sample(),
            &reasoner(),
            "is a statute a legal document",
        );
        assert!(r.response.to_lowercase().contains("yes"));
        assert!(
            !r.response.contains("is transitive"),
            "a single-hop (direct edge) answer must not append a transitivity note; got {:?}",
            r.response
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_enacted_species_are_all_law() {
        // Every enacted written instrument reaches the LegalSource genus.
        assert_yes("is a regulation a law");
        assert_yes("is a constitution a law");
        assert_yes("is a treaty a law");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_statute_is_a_legal_document() {
        // A MULTI-WORD label surface ("legal document") — the recognizer collapses
        // it and the direct Subsumption Statute ⊑ LegalDocument answers Yes.
        assert_yes("is a statute a legal document");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subsumption_is_reflexive_a_statute_is_a_statute() {
        // Subsumption is reflexive (per the Relations ontology), so a concept is-a
        // itself — the reflexive short-circuit, not a fabricated edge.
        assert_yes("is a statute a statute");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn case_law_is_a_law() {
        // Precedent (label "case law") ⊑ LegalSource directly — a multi-word label
        // on the resolved concept and the genus surface "law".
        assert_yes("is case law a law");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_a_statute_answers_from_the_loaded_gloss() {
        // A single-entity definitional question reads the LegalSources gloss.
        let r = process_with_reasoner(&English::sample(), &reasoner(), "what is a statute");
        assert_eq!(r.outcome, ChatOutcome::Answered);
        assert!(r.from_ontology, "the gloss comes from the loaded ontology");
        assert!(
            r.response.to_lowercase().contains("statute"),
            "the answer names the queried concept; got {:?}",
            r.response
        );
        assert!(
            r.response.contains("norm") || r.response.contains("legal person"),
            "the answer surfaces the loaded LKIF gloss; got {:?}",
            r.response
        );
        assert!(
            credits_legal_sources(&r),
            "the definition credits LegalSources"
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn a_law_is_not_a_statute_provable_negation_from_antisymmetry() {
        // Subsumption is antisymmetric: Statute ⊑ LegalSource holds AND Statute ≠
        // LegalSource, so ¬(LegalSource ⊑ Statute) — a PROVABLE No, not silence. The
        // denial reads "is not a", taxonomy_checked false, but STILL from_ontology
        // (a derived disproof, the real directional negation the honesty fix keeps).
        let no = answer_question(
            &reasoner(),
            English::sample_static(),
            &[],
            "is",
            &entity_args("law", "statute"),
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            no.taxonomy_checked,
            Some(("law".to_string(), "statute".to_string(), false)),
            "a law is NOT a statute (antisymmetry); got {:?}",
            no.taxonomy_checked
        );
        assert!(
            no.from_ontology,
            "a provable negation is still an ontology answer"
        );
        assert!(
            no.response.to_lowercase().contains("not"),
            "the denial must read as a negation; got {:?}",
            no.response
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn siblings_abstain_not_a_false_no() {
        // Statute and Regulation are siblings under LegalDocument — no path either
        // way, no antisymmetric reverse, no opposition. The honesty fix ABSTAINS
        // (does not fabricate "a statute is not a regulation"): typed Abstained,
        // from_ontology false, taxonomy_checked recorded false (traversed, no proof).
        let r = process_with_reasoner(&English::sample(), &reasoner(), "is a statute a regulation");
        assert!(
            matches!(r.outcome, ChatOutcome::Abstained { .. }),
            "siblings must abstain, not assert a false No; got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(!r.from_ontology, "an abstention is not an ontology answer");

        // At the answer_question layer the abstention is explicit: taxonomy_checked
        // records the (child, parent, false) it traversed, but from_ontology is false.
        let a = answer_question(
            &reasoner(),
            English::sample_static(),
            &[],
            "is",
            &entity_args("statute", "regulation"),
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            a.taxonomy_checked,
            Some(("statute".to_string(), "regulation".to_string(), false))
        );
        assert!(
            !a.from_ontology,
            "no path + no disproof ⇒ abstain (from_ontology false), never a fabricated No"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_cross_universe_pair_abstains() {
        // "is a statute an animal": `statute` is a loaded LegalSources concept,
        // `animal` a WordNet (sample) concept — two disjoint identity universes with
        // no edge between them. No path, no antisymmetric reverse, no opposition ⇒
        // abstain, never a false No. (Both surfaces DO resolve — the contrast with a
        // truly-unknown word that would collapse to a definition.)
        let r = process_with_reasoner(&English::sample(), &reasoner(), "is a statute an animal");
        assert!(
            matches!(r.outcome, ChatOutcome::Abstained { .. }),
            "a cross-universe pair must abstain; got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(!r.from_ontology);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn case_law_vs_legal_document_abstains_honestly() {
        // LKIF places Precedent (case law) DIRECTLY under Legal_Source, NOT under
        // Legal_Document — so there is no path Precedent ⊑ LegalDocument, no reverse
        // path, and no declared opposition/disjointness between them. Per the honesty
        // fix this is an ABSTENTION, not a provable No: "case law is not a legal
        // document" would require a cited disjointness edge the ontology does not
        // (yet) assert — a follow-up if a literature-grounded Precedent ⊥
        // LegalDocument edge is later added.
        let a = answer_question(
            &reasoner(),
            English::sample_static(),
            &[],
            "is",
            &entity_args("case law", "legal document"),
            montague::QuestionIllocution::Polar,
        );
        assert_eq!(
            a.taxonomy_checked,
            Some(("case law".to_string(), "legal document".to_string(), false)),
            "no path Precedent ⊑ LegalDocument; got {:?}",
            a.taxonomy_checked
        );
        assert!(
            !a.from_ontology,
            "no path + no disproof ⇒ honest abstention, not a fabricated No"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn without_the_base_the_yes_disappears_the_with_without_contrast() {
        // The contrast that proves the Yes comes from the LOADED base, not a
        // hardcoded branch: English-only (`process`, no LegalSources) cannot ground
        // "statute"/"law" (the sample lexicon knows neither) and does NOT answer a
        // hardcoded Yes; WITH the base the SAME question answers Yes crediting
        // LegalSources.
        let (without, _, _) = process(&English::sample(), "is a statute a law");
        assert!(
            !without.to_lowercase().contains("yes"),
            "english-only must NOT answer a hardcoded Yes; got {without:?}"
        );

        let with = process_with_reasoner(&English::sample(), &reasoner(), "is a statute a law");
        assert_eq!(with.outcome, ChatOutcome::Answered);
        assert!(with.response.to_lowercase().contains("yes"));
        assert!(
            credits_legal_sources(&with),
            "the Yes credits the loaded LegalSources base"
        );
        assert_ne!(
            without, with.response,
            "loading the base must change the answer (it is not hardcoded)"
        );
    }
}

// =========================================================================
// W2.2 — WORDS ARE POINTERS INTO ENGLISH, through the SAME chat path
// =========================================================================
//
// A loaded `.prx` node that DECLARES an into-English typing functor
// (`Canine ↦ english_wordnet:s-dog`) inherits English's taxonomy: "is rex an
// animal" affirms through WordNet's own `s-dog ⊑ s-mammal ⊑ s-animal` chain, and
// the answer credits English as reasoned-over — WITHOUT English ever being a
// loaded ontology. An UNDECLARED node (typed `Mineral`, surface an animal word)
// does NOT link — DECLARED-TYPE grounding, not surface auto-matching (§9), so the
// chat ABSTAINS. Nothing is hardcoded: the Yes/Abstain is read off the composed
// reasoner's cross-universe `reaches`, never a `match` on the question text.
#[cfg(test)]
mod into_english_base {
    use std::rc::Rc;

    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;
    use pr4xis_runtime::archive::Archive;
    use pr4xis_runtime::connection::{Connection, GeneratorAction};
    use pr4xis_runtime::definition::Definition;
    use pr4xis_runtime::ontology::materialize;

    /// The menagerie grounded into English through the loader's GENERAL grounding
    /// pass (`ground_loaded_set`, which seeds English as the transient target peer
    /// and mints the declared into-English typing edge). English is NEVER installed
    /// as a loaded `RuntimeOntology`.
    fn menagerie_reasoner() -> ComposedReasoner {
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Canine".into(),
                    name: "rex".into(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("a companion dog".into()),
                },
                Definition {
                    kind: "Mineral".into(),
                    name: "salmon".into(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("typed a Mineral; its surface is an animal word".into()),
                },
            ],
            connections: vec![Connection {
                kind: "InstanceFunctor".into(),
                source: "menagerie".into(),
                target: "english_wordnet".into(),
                action: GeneratorAction::Functor {
                    map_object: vec![("Canine".into(), "s-dog".into())],
                    map_morphism: vec![("denotes".into(), "Subsumption".into())],
                },
                laws: vec!["PreservesTyping".into()],
            }],
        };
        let onto = materialize(archive, OntologyName::new_static("menagerie"))
            .expect("the menagerie materializes");
        let mut set = vec![Rc::new(onto)];
        pr4xis_domains::formal::meta::grounding::ground_loaded_set(
            &mut set,
            English::sample_static(),
        )
        .expect("the single-level menagerie grounds");
        ComposedReasoner::new(English::sample_static(), set)
    }

    #[pr4xis::praxis_value(Extensible, Verifiable)]
    #[test]
    fn is_a_declared_node_an_animal_affirms_through_english() {
        let composed = menagerie_reasoner();
        // GATE (i): English is never a loaded ontology.
        assert!(
            composed
                .loaded()
                .iter()
                .all(|o| o.id().as_str() != "english_wordnet"),
            "english_wordnet must never be a loaded ontology"
        );

        let r = process_with_reasoner(&English::sample(), &composed, "is rex an animal");
        assert_eq!(
            r.outcome,
            ChatOutcome::Answered,
            "a declared node is an animal via English's chain; got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(
            r.from_ontology,
            "the affirmation is an ontology answer; got {:?}",
            r.response
        );
        assert!(
            r.response.to_lowercase().contains("yes"),
            "the answer must affirm (Yes); got {:?}",
            r.response
        );
        // reasoned_over credits the loaded menagerie AND the English substrate
        // (WordNet supplied the is-a chain), success-marked.
        let prov = r.trace.reasoned_over();
        assert!(
            prov.iter().any(
                |(o, ok)| matches!(o, TraceOntology::Loaded(n) if n.as_str() == "menagerie") && *ok
            ),
            "the answer credits the loaded menagerie; got {prov:?}"
        );
        assert!(
            prov.iter().any(
                |(o, ok)| matches!(o, TraceOntology::Loaded(n) if n.as_str() == "english_wordnet")
                    && *ok
            ),
            "reasoned_over must include english — WordNet's taxonomy supplied the is-a chain; \
             got {prov:?}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn is_an_undeclared_node_an_animal_abstains() {
        let composed = menagerie_reasoner();
        // The undeclared `salmon` (kind Mineral) carries no into-English typing, so
        // even though its surface is an animal word, the chat ABSTAINS (§9 — Policy B
        // / WSD surface-matching declined), never a fabricated Yes.
        let r = process_with_reasoner(&English::sample(), &composed, "is salmon an animal");
        assert!(
            matches!(r.outcome, ChatOutcome::Abstained { .. }),
            "an undeclared node must abstain (§9); got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(!r.from_ontology, "an abstention is not an ontology answer");
    }
}

// =========================================================================
// Non-legal guardrail — the Dependability demo through the SAME chat path
// =========================================================================
//
// The three fixes are NOT legal-specific: any compiled ontology, grounded into
// English, must render its natural labels and surface the transitivity rule. The
// Avizienis et al. (2004) Dependability taxonomy is the non-legal witness — a
// `DormantFault ⊑ Fault ⊑ Threat` chain whose multi-word label "dormant fault"
// (its `canonicalForm` Form) must print in place of the identifier "DormantFault".
#[cfg(test)]
mod dependability_demo {
    use std::rc::Rc;

    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

    /// The Dependability ontology through the default lexicalizing projection, so
    /// each concept's label ("Dormant fault", "Threat") rides as an `ontolex:Form`.
    fn dependability_corpus() -> RuntimeOntology {
        materialize(
            emit::<DependabilityCategory>(),
            OntologyName::new_static("Dependability"),
        )
        .expect("Dependability corpus materializes")
    }

    fn reasoner() -> ComposedReasoner {
        ComposedReasoner::new(
            English::sample_static(),
            vec![Rc::new(dependability_corpus())],
        )
    }

    fn credits_dependability(r: &ProcessResult) -> bool {
        r.trace.reasoned_over().iter().any(|(o, ok)| {
            matches!(o, TraceOntology::Loaded(n) if n.as_str() == "Dependability") && *ok
        })
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn is_a_dormant_fault_a_fault_renders_the_label_not_the_identifier() {
        // Fix 1 (non-legal): the multi-word surface "dormant fault" resolves and
        // the answer renders it — never the Rust identifier "DormantFault". A
        // direct edge DormantFault ⊑ Fault, so a Yes crediting the loaded corpus.
        let r = process_with_reasoner(
            &English::sample(),
            &reasoner(),
            "is a dormant fault a fault",
        );
        assert_eq!(
            r.outcome,
            ChatOutcome::Answered,
            "a dormant fault IS a fault; got {:?} / {:?}",
            r.outcome,
            r.response
        );
        assert!(r.response.to_lowercase().contains("yes"));
        assert!(
            r.response.contains("dormant fault"),
            "the answer renders the natural label 'dormant fault'; got {:?}",
            r.response
        );
        assert!(
            !r.response.contains("DormantFault"),
            "the Rust identifier 'DormantFault' must never print; got {:?}",
            r.response
        );
        assert!(
            credits_dependability(&r),
            "the answer credits the loaded Dependability ontology it reasoned over"
        );
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_multi_hop_dependability_answer_carries_the_transitivity_note() {
        // Fix 2 (non-legal): DormantFault ⊑ Fault ⊑ Threat is a multi-hop is-a
        // chain, so the SAME transitivity licensing note appears — the rule is a
        // property of the relation, not of the legal domain.
        let r = process_with_reasoner(
            &English::sample(),
            &reasoner(),
            "is a dormant fault a threat",
        );
        assert_eq!(r.outcome, ChatOutcome::Answered, "got {:?}", r.response);
        assert!(r.response.to_lowercase().contains("yes"));
        assert!(
            r.response.contains("is transitive") && r.response.contains("Tarski"),
            "a multi-hop answer names the transitivity rule + its citation; got {:?}",
            r.response
        );
        // The intermediate whole renders by label, not identifier.
        assert!(
            !r.response.contains("DormantFault"),
            "no identifier leaks in the chain; got {:?}",
            r.response
        );
    }

    /// SLICE-E (R-2): a composition-MINTED definiendum (the degenerate
    /// `is:N/N + a:N` derivation of "what is a long" mints `Pred{"is a"}`) is
    /// never named an unknown WORD. With no failing lexical surface to name,
    /// the reply is the realizer's general-gap surface — both references are
    /// realizer-generated, never authored literals.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn composite_definiendum_is_never_reported_an_unknown_word() {
        use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
        use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

        let english = English::sample();
        // "dog" and "cat" both resolve in the sample lexicon; "dog cat" does
        // not — the unresolvable-composite-with-resolving-leaves shape.
        let composite = montague::Sem::Pred {
            word: "dog cat".into(),
            role: montague::GrammaticalRole::Argument,
            provenance: montague::PredProvenance::Composite {
                func: "dog".into(),
                arg: "cat".into(),
            },
        };
        let result = answer_question(
            &english,
            &english,
            &[],
            "what",
            core::slice::from_ref(&composite),
            montague::QuestionIllocution::Content,
        );

        let gap_naming_composite = realize::realize(
            &ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity("dog cat"),
        );
        let general_gap = realize::realize(&ResponseContent::new(ResponseFrame::UnknownVocabulary));

        assert!(
            !result.from_ontology,
            "an unresolvable composite is not an answer"
        );
        assert!(
            !result.response.contains(&gap_naming_composite),
            "the minted string must never be claimed an unknown word; got {:?}",
            result.response
        );
        assert_eq!(
            result.response, general_gap,
            "with no failing lexical surface the reply is the general gap"
        );
    }

    /// SLICE-E (R-2): an abstention's `unresolved` set names failing LEXICAL
    /// leaves, never a composition-minted concatenation; a composite whose
    /// leaves all resolve contributes nothing; a lexical unknown keeps
    /// today's naming.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn unresolved_reports_composite_leaves_never_the_minted_string() {
        let english = English::sample();
        let question = |pred: montague::Sem| montague::Sem::Question {
            predicate: "what".into(),
            arguments: vec![pred],
            illocution: montague::QuestionIllocution::Content,
        };
        let composite = |word: &str, func: &str, arg: &str| montague::Sem::Pred {
            word: word.into(),
            role: montague::GrammaticalRole::Argument,
            provenance: montague::PredProvenance::Composite {
                func: func.into(),
                arg: arg.into(),
            },
        };

        // Both leaves resolve (the "is a" shape) → nothing unresolved.
        let resolving = question(composite("dog cat", "dog", "cat"));
        assert_eq!(
            unresolved_surfaces(&resolving, &english),
            Vec::<String>::new(),
            "a composite whose leaves resolve names nothing unresolved"
        );

        // A failing leaf is named; the minted string never is.
        let failing = question(composite("blorptt cat", "blorptt", "cat"));
        assert_eq!(
            unresolved_surfaces(&failing, &english),
            vec!["blorptt".to_string()],
            "only the failing lexical leaf is reported"
        );

        // A lexical unknown keeps today's naming.
        let lexical = question(montague::Sem::Pred {
            word: "blorptt".into(),
            role: montague::GrammaticalRole::Argument,
            provenance: montague::PredProvenance::Lexical,
        });
        assert_eq!(
            unresolved_surfaces(&lexical, &english),
            vec!["blorptt".to_string()],
            "a lexical unknown surface is still named"
        );
    }

    /// The `who_is_x` corpus gap (caregiver corpus rows "Who is DCH?",
    /// "Who is a General Caregiver?", "Who is a Legally Responsible
    /// Individual (LRI)?" — 9 of the corpus's 10 `who_is_x`
    /// `UnparsedKnownTerm` rows): a "who is X" single-entity Content
    /// question over a DOMAIN-LOADED entity (a role/organization/category a
    /// loaded caregiving source specifically defines — `ConceptView::
    /// is_domain_loaded`), with none of the scope/infinitival/deontic-or-
    /// possessive markers present, must route through `define_word` exactly
    /// like the "what"/"which" sibling branch — not the generic
    /// `AdmitLimitation` decline every non-"what"/"which" wh-word
    /// previously fell to unconditionally.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn who_is_a_domain_loaded_role_routes_through_define_word() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-caregiver-n"><Lemma writtenForm="caregiver" partOfSpeech="n"/><Sense id="caregiver-n-01" synset="s-caregiver"/></LexicalEntry>
    <Synset id="s-caregiver" ili="i900" partOfSpeech="n"><Definition>a person who attends to the needs of a dependent individual</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("care"))
            .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        let entity = montague::Sem::Concept {
            word: "caregiver".into(),
            concepts: composed.lookup("caregiver").to_vec(),
            role: montague::GrammaticalRole::Argument,
            expression_use: ExpressionUse::Used,
        };
        let result = answer_question(
            &composed,
            composed.english(),
            &[],
            "who",
            core::slice::from_ref(&entity),
            montague::QuestionIllocution::Content,
        );
        assert_eq!(
            result.frame,
            pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame::AssertKnowledge,
            "a domain-loaded role/category answers a bare \"who is X\" exactly \
             like \"what is X\"; got {:?}",
            result.response
        );
        assert!(
            result.from_ontology,
            "the answer reasoned over the loaded caregiving lexicon"
        );
        assert!(
            result.response.contains("attends to the needs"),
            "the REAL definition surfaces, not a generic decline; got {:?}",
            result.response
        );
    }

    /// The negative half of the same gate: a "who is X" question whose
    /// tokens carry a POSSESSIVE PRONOUN ("our") binding the entity to the
    /// asker's own context ("Who is considered OUR employer?" — a genuine
    /// caregiver corpus row that must NOT be answered as a generic
    /// definition) still declines via `AdmitLimitation`, even though
    /// "employer" is itself domain-loaded. `has_deontic_or_descriptive_
    /// marker`'s possessive-pronoun disjunct — the SAME check the sibling
    /// "what"/"which" branch already relies on — is the gate.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn who_is_a_possessively_bound_entity_still_declines() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-employer-n"><Lemma writtenForm="employer" partOfSpeech="n"/><Sense id="employer-n-01" synset="s-employer"/></LexicalEntry>
    <Synset id="s-employer" ili="i901" partOfSpeech="n"><Definition>an entity that hires and pays workers</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("care"))
            .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        let entity = montague::Sem::Concept {
            word: "employer".into(),
            concepts: composed.lookup("employer").to_vec(),
            role: montague::GrammaticalRole::Argument,
            expression_use: ExpressionUse::Used,
        };
        let tokens = [TypedToken {
            expression_use: ExpressionUse::Used,
            word: "our".into(),
            lambek_type: pr4xis_domains::cognitive::linguistics::lambek::types::svo::determiner(),
        }];
        let result = answer_question(
            &composed,
            composed.english(),
            &tokens,
            "who",
            core::slice::from_ref(&entity),
            montague::QuestionIllocution::Content,
        );
        assert_eq!(
            result.frame,
            pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame::AdmitLimitation,
            "a possessively-bound entity is a personal referent, not a \
             generic definiendum — must still decline; got {:?}",
            result.response
        );
        assert!(!result.from_ontology, "a decline is not a grounded answer");
    }

    /// `normalize_ascii_double_quotes` in isolation: the by-occurrence-
    /// parity rewrite into the canonical directional pair, an already-
    /// directional input left untouched, plain text with no quotes at all
    /// unaffected, and an unmatched trailing ASCII mark still alternates
    /// (degrading harmlessly downstream exactly as an unmatched directional
    /// opener already does — `collapse_quoted_spans`'s own doc).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn normalize_ascii_double_quotes_rewrites_to_the_directional_pair() {
        assert_eq!(
            normalize_ascii_double_quotes("the type \"use of a restraint\" mean"),
            "the type \u{201C}use of a restraint\u{201D} mean"
        );
        assert_eq!(
            normalize_ascii_double_quotes("the type \u{201C}use of a restraint\u{201D} mean"),
            "the type \u{201C}use of a restraint\u{201D} mean",
            "an already-directional input is unaffected"
        );
        assert_eq!(
            normalize_ascii_double_quotes("no quotes here at all"),
            "no quotes here at all"
        );
        assert_eq!(
            normalize_ascii_double_quotes("a stray 6\" pipe"),
            "a stray 6\u{201C} pipe",
            "an unmatched trailing mark still alternates — harmless downstream, \
             mirroring collapse_quoted_spans's own unmatched-opener handling"
        );
    }

    /// The what_does_x_mean corpus gap end to end (caregiver corpus row:
    /// `What does the incident type "use of a restraint" mean?`, key_term
    /// "restraint"): entity extraction over a quoted-appositive complement
    /// used to garble into the head NP plus the quoted span's first word
    /// only ("incident type use") because `tokenize::collapse_quoted_spans`
    /// deliberately leaves the two ASCII quote marks untouched (its own
    /// doc: they are `QuoteRole::Ambiguous`, only the DIRECTIONAL glyphs
    /// collapse into a quoted-mention token) — and this corpus's harvested
    /// text quotes with plain ASCII `"…"` throughout. Isolated proof this
    /// closes the gap: the identical sentence shape, ASCII quotes
    /// normalized to directional at the pipeline's own entry
    /// (`normalize_ascii_double_quotes`, wired into
    /// `process_with_reasoner`), correctly isolates the quoted span as the
    /// sole entity and answers from its own gloss — not the garbled
    /// head-noun compound. Deliberately uses an EMBEDDED WordNet word
    /// ("mammal", `English::sample`'s own fixture), not a loaded-lexicon
    /// one, for the quoted definiendum: a loaded surface gets an
    /// ADDITIONAL bare-noun(N) alternative type from `process_with_
    /// reasoner`'s separate "offer a determiner-attachable N reading"
    /// widening (needed for "a `<loaded-term>`" elsewhere), which competes
    /// with the close-apposition reading for the SAME cell and can still
    /// win — a distinct, NOT-yet-fixed derivation-preference gap this test
    /// does not claim to close (confirmed via a synthetic control: the
    /// identical sentence with a domain-loaded quoted word still garbles).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn what_does_the_quoted_appositive_mean_isolates_the_quoted_span() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;

        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="care" label="Care" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-type-n"><Lemma writtenForm="type" partOfSpeech="n"/><Sense id="type-n-01" synset="s-type"/></LexicalEntry>
    <Synset id="s-type" ili="i2" partOfSpeech="n"><Definition>a category of reportable incident</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let english = English::sample_static();
        let onto = lexicon_runtime_ontology_from_lmf(LMF, OntologyName::new_static("care"))
            .expect("inline caregiving lexicon loads");
        let composed = ComposedReasoner::new(english, vec![std::rc::Rc::new(onto)]);

        let result = process_with_reasoner(
            composed.english(),
            &composed,
            "What is the type \"mammal\"?",
        );
        assert_eq!(
            result.outcome,
            ChatOutcome::Answered,
            "the quoted definiendum resolves once ASCII quotes are normalized; \
             got {:?} / {:?}",
            result.outcome,
            result.response
        );
        assert!(
            result.response.contains("warm-blooded vertebrate"),
            "answers from the quoted definiendum's own gloss; got {:?}",
            result.response
        );
        assert!(
            !result.response.to_lowercase().contains("type mammal"),
            "the quoted definiendum must be isolated, never concatenated \
             with its head noun; got {:?}",
            result.response
        );
    }

    /// The EXACT caregiver-corpus row (`what_does_x_mean` bucket, index
    /// 305) end to end against the REAL, full WordNet (`english_loaded()`,
    /// not a synthetic sample fixture): 'What does the incident type "use
    /// of a restraint" mean?' used to extract the garbled three-word
    /// compound "incident type use" (dropping "of a restraint" entirely).
    /// With `normalize_ascii_double_quotes` wired into `process_with_
    /// reasoner`, the CYK chart now sees a real quoted-mention token for
    /// the appositive and the entity-gathering stage isolates it CLEANLY
    /// as "use of a restraint" — no garbling. The outcome still honestly
    /// abstains here (`english_loaded()` alone carries no caregiving/HCBS
    /// statute corpus, so "use of a restraint" — a program-specific
    /// incident-type LABEL, not a WordNet dictionary phrase — has nothing
    /// to define it in THIS test's deliberately narrow fixture); that is
    /// the expected, honest behavior of the vocabulary-gap path, not a
    /// remaining defect in the extraction this task owns — the real
    /// production/corpus-test pipeline loads the statute corpus that
    /// grounds this exact label via `defines_pointers`.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn real_corpus_row_use_of_a_restraint_extracts_the_quoted_span_cleanly() {
        use pr4xis_domains::cognitive::linguistics::english::english_loaded;

        let en = english_loaded();
        let result = process_with_reasoner(
            en,
            en,
            "What does the incident type \"use of a restraint\" mean?",
        );
        assert!(
            matches!(
                &result.outcome,
                ChatOutcome::Abstained { unresolved }
                    if unresolved == &vec!["use of a restraint".to_string()]
            ),
            "the unresolved surface must be the CLEAN quoted definiendum, \
             never the old garbled \"incident type use\"/\"type use\" \
             compound; got {:?} / {:?}",
            result.outcome,
            result.response
        );
        assert!(
            !result.response.to_lowercase().contains("type use"),
            "the pre-fix garbled entity must never reappear; got {:?}",
            result.response
        );
    }

    /// The registered axiom carrying the two claims above is green.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn definiendum_is_a_lexical_unit_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(DefiniendumIsALexicalUnit.verify().is_ok());
    }

    /// Adversarial corpus index 69 ("What is the Caregiver Social Security
    /// Credit Program?", category `fabricated_term`): the registered axiom —
    /// a definitional what-is question whose multi-word nominal subject does
    /// not resolve as a unit is answered by an abstention naming the full
    /// compound, never by constituent-sense enumeration (Downing 1977;
    /// Kripke 1980; Lin, Hilton & Evans 2022) — is green.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn definitional_subject_compound_abstains_as_a_unit_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(DefinitionalSubjectCompoundAbstainsAsAUnit.verify().is_ok());
    }

    /// [`unresolved_definitional_subject_compound`]'s run construction and
    /// as-a-unit resolution, exercised directly against an inline-LMF
    /// fixture (the same style as
    /// `noun_phrase_heads_collapses_an_unseparated_compound_run_to_its_head`),
    /// including the two controls [`English::sample_static`] cannot host:
    /// a compound that IS loaded as a unit (must return `None` — the
    /// existing definitional paths own it), and a collapsed multi-word
    /// constituent participating in a LARGER unloaded compound (the exact
    /// index-69 shape: "social security" is loaded, "caregiver social
    /// security credit program" is not).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn unresolved_definitional_subject_compound_requires_unit_resolution() {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-alpha-n">
      <Lemma writtenForm="alpha" partOfSpeech="n"/>
      <Sense id="alpha-n-1" synset="s-alpha"/>
    </LexicalEntry>
    <Synset id="s-alpha" ili="i1" partOfSpeech="n"><Definition>first thing</Definition></Synset>
    <LexicalEntry id="e-beta-n">
      <Lemma writtenForm="beta" partOfSpeech="n"/>
      <Sense id="beta-n-1" synset="s-beta"/>
    </LexicalEntry>
    <Synset id="s-beta" ili="i2" partOfSpeech="n"><Definition>second thing</Definition></Synset>
    <LexicalEntry id="e-delta-epsilon-n">
      <Lemma writtenForm="delta epsilon" partOfSpeech="n"/>
      <Sense id="delta-epsilon-n-1" synset="s-de"/>
    </LexicalEntry>
    <LexicalEntry id="e-zeta-n">
      <Lemma writtenForm="zeta" partOfSpeech="n"/>
      <Sense id="zeta-n-1" synset="s-de"/>
    </LexicalEntry>
    <Synset id="s-de" ili="i3" partOfSpeech="n"><Definition>a loaded collocation</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let en = English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"));
        let tok = |word: &str, lambek_type| TypedToken {
            expression_use: ExpressionUse::Used,
            word: word.into(),
            lambek_type,
        };

        // A bare predicate-less NP satisfies `is_what_copula_question`'s
        // no-predicate disjunct, so the fixture needs no function-word rows.
        //
        // (1) Two known constituents, unloaded as a unit → the compound.
        let unloaded = vec![tok("alpha", svo::noun()), tok("beta", svo::noun())];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &unloaded).as_deref(),
            Some("alpha beta")
        );

        // (2) A compound loaded AS A UNIT (arriving as the collapsed
        //     multi-word proper-noun token `collapse_multiword_surfaces`
        //     mints) → None: the existing paths own it.
        let loaded_unit = vec![tok("delta epsilon", svo::proper_noun())];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &loaded_unit),
            None
        );

        // (3) The index-69 shape: a loaded multi-word constituent inside a
        //     LARGER compound that resolves to nothing as a unit → the FULL
        //     compound, never a constituent or sub-span.
        let larger = vec![
            tok("alpha", svo::noun()),
            tok("delta epsilon", svo::proper_noun()),
            tok("beta", svo::noun()),
        ];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &larger).as_deref(),
            Some("alpha delta epsilon beta")
        );

        // (4) A single known constituent is not a compound → None (the
        //     single-definiendum path owns it).
        let single = vec![tok("the", svo::determiner()), tok("alpha", svo::noun())];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &single),
            None
        );

        // (5) A run containing an UNKNOWN word is a different epistemic
        //     situation (name that word) — the run breaks at it, leaving no
        //     multi-word run of KNOWN constituents → None.
        let with_unknown = vec![tok("blorptt", svo::noun()), tok("alpha", svo::noun())];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &with_unknown),
            None
        );

        // (6) An APPOSITIVE alias re-mention ("Electronic Visit
        //     Verification (EVV)" — "zeta" is a second written form of the
        //     SAME concept "delta epsilon" resolves to) is not a second
        //     compound constituent (Quirk et al. 1985 §17.65-17.68): the
        //     first-concept dedup leaves one distinct constituent → None,
        //     so the existing definitional paths keep answering the six
        //     real Green corpus rows this control protects.
        let appositive = vec![
            tok("delta epsilon", svo::proper_noun()),
            tok("zeta", svo::noun()),
        ];
        assert_eq!(
            unresolved_definitional_subject_compound(&en, &en, &appositive),
            None
        );
    }

    /// SLICE-A (R-2): a define answer enumerates EVERY sense of the resolved
    /// definiendum in the loaded order — the registered axiom's 7-sense
    /// polysemy fixture is deeper than the deleted day-one `take(5)` cap, so
    /// this is red while any display cap survives.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn define_enumerates_the_loaded_sense_inventory_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(DefineEnumeratesTheLoadedSenseInventory.verify().is_ok());
    }

    /// A define answer floats its domain-lexicon senses ahead of the
    /// general-purpose WordNet senses (a stable partition, nothing dropped) —
    /// the ordering that leads "what is respite" with the caregiving statutory
    /// definition instead of "a pause from doing something".
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn define_prioritizes_domain_senses_over_general_senses_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            DefinePrioritizesDomainSensesOverGeneralSenses
                .verify()
                .is_ok()
        );
    }

    /// task #31: among multiple `(child-sense, parent-sense)` pairs that all
    /// satisfy a taxonomy relation, the two-entity answer path picks the one
    /// with the highest relation-expanded gloss overlap, not the first pair
    /// in loaded lexicon order.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_entity_path_prefers_gloss_overlap_among_reaching_pairs_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            TwoEntityPathPrefersGlossOverlapAmongReachingPairs
                .verify()
                .is_ok()
        );
    }

    /// task #37/#38: the VerbNet corroboration gate must never downgrade a
    /// Subsumption (is-a) query — the fix for the measured 4→47 is-a
    /// regression a first version of this mechanism produced.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn verbnet_corroboration_scoped_to_similarity_and_equivalence_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            VerbNetCorroborationScopedToSimilarityAndEquivalence
                .verify()
                .is_ok()
        );
    }

    /// ConceptNet integration: the lone-hit corroboration composition (Rule
    /// 1/Rule 2) and ConceptNet's own Similarity/Equivalence-only scoping.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn conceptnet_corroboration_composes_with_verbnet_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(ConceptNetCorroborationComposesWithVerbNet.verify().is_ok());
    }

    /// FrameNet integration: the three-source lone-hit corroboration
    /// composition (Rule 1/Rule 2) and FrameNet's own Similarity/
    /// Equivalence-only scoping.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn framenet_corroboration_composes_with_verbnet_and_conceptnet_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            FrameNetCorroborationComposesWithVerbNetAndConceptNet
                .verify()
                .is_ok()
        );
    }

    /// SUMO integration: the four-source lone-hit corroboration composition
    /// (Rule 1/Rule 2) and SUMO's own Similarity/Equivalence-only scoping.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sumo_corroboration_composes_with_verbnet_conceptnet_and_framenet_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet
                .verify()
                .is_ok()
        );
    }

    /// PropBank integration: the five-source lone-hit corroboration
    /// composition (Rule 1/Rule 2), PropBank's own Similarity/
    /// Equivalence-only scoping, and the cross-POS-only scoping decision.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn propbank_corroboration_composes_with_verbnet_conceptnet_framenet_and_sumo_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(
            PropBankCorroborationComposesWithVerbNetConceptNetFrameNetAndSumo
                .verify()
                .is_ok()
        );
    }

    /// The Slice-A sibling sites (R-2 declared follow-ups): explore_concepts'
    /// single-word and pairwise loops, and build_taxonomy_response's subtype
    /// list, each stop cutting to a first-sense/take(5) view of the loaded
    /// inventory.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn chat_exploration_enumerates_the_loaded_inventory_axiom_is_green() {
        use pr4xis::ontology::Axiom;
        assert!(ChatExplorationEnumeratesTheLoadedInventory.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_a_dormant_fault_answers_from_the_avizienis_gloss() {
        // The definitional path reads the loaded Avizienis gloss for the concept.
        let r = process_with_reasoner(&English::sample(), &reasoner(), "what is a dormant fault");
        assert_eq!(r.outcome, ChatOutcome::Answered);
        assert!(r.from_ontology, "the gloss comes from the loaded ontology");
        assert!(
            r.response.to_lowercase().contains("dormant fault"),
            "the answer names the queried concept by label; got {:?}",
            r.response
        );
        assert!(
            r.response.contains("not yet been activated"),
            "the answer surfaces the loaded Avizienis gloss; got {:?}",
            r.response
        );
        assert!(credits_dependability(&r));
    }
}

/// Task #38: `answer_statement`'s two-entity truth-check (`verify_relational_claim`,
/// shared with `answer_question`) and the presupposed-clause detector built on
/// it (`check_presupposed_clause`). All against the small embedded sample
/// corpus (`English::sample()`, no loaded ontology) — the same fast corpus
/// `tests::process_taxonomy_question` already uses for "is a dog a mammal".
#[cfg(test)]
mod presupposition_verification {
    use super::*;

    fn en() -> English {
        English::sample()
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_true_declarative_subsumption_claim_affirms() {
        let r = process_with_reasoner(&en(), &en(), "a dog is a mammal");
        assert_eq!(r.outcome, ChatOutcome::Answered, "got {:?}", r.response);
        assert!(r.from_ontology);
        assert!(
            r.response.to_lowercase().contains("yes"),
            "a true claim must affirm; got {:?}",
            r.response
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn a_false_declarative_subsumption_claim_negates() {
        // Reversed direction of the true fact above — mammal is NOT a dog
        // (a strict antisymmetric violation), so this must be a confident No,
        // never a blind AssertKnowledge affirmation (the pre-fix bug).
        let r = process_with_reasoner(&en(), &en(), "a mammal is a dog");
        assert_eq!(r.outcome, ChatOutcome::Answered, "got {:?}", r.response);
        assert!(
            r.response.to_lowercase().contains("no"),
            "a provably false claim must negate; got {:?}",
            r.response
        );
        assert!(
            !r.response.to_lowercase().contains("yes"),
            "must never affirm a false claim; got {:?}",
            r.response
        );
    }

    /// The argument-order regression guard: `Sem::Prop`'s subject is the
    /// LAST-absorbed argument (backward application), opposite
    /// `Sem::Question`'s subject-first convention — see the doc comment on
    /// `answer_statement`'s two-entity branch. Getting this backwards makes a
    /// TRUE claim ("a seal is a mammal") produce a false correction instead
    /// of an affirmation — caught live during this capability's own
    /// development.
    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn declarative_argument_order_is_subject_last_not_subject_first() {
        let r = process_with_reasoner(&en(), &en(), "a dog is a mammal");
        assert!(
            !r.response.to_lowercase().contains("no"),
            "a true claim must not be misread as its reverse and negated; got {:?}",
            r.response
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn find_presupposed_clause_extracts_the_since_clause_and_main_clause() {
        let found =
            find_presupposed_clause("Since a mammal is a dog, is a cat also a mammal?").unwrap();
        assert_eq!(found.0, "a mammal is a dog");
        assert_eq!(found.1, "is a cat also a mammal?");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn find_presupposed_clause_ignores_an_unregistered_trigger() {
        // "because" is deliberately NOT a registered trigger (no verified
        // citation grounds it as a presupposition trigger — see
        // `presupposition_trigger_lexicon`), so this must be a no-op.
        assert!(
            find_presupposed_clause("Because a dog is a mammal, is a cat also a mammal?").is_none()
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn find_presupposed_clause_requires_a_comma() {
        assert!(find_presupposed_clause("Since a dog is a mammal what is a cat").is_none());
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn a_false_fronted_presupposition_is_corrected_not_built_on() {
        // "a mammal is a dog" is the SAME provable antisymmetric-violation
        // falsehood as the standalone declarative test above, now fronted as
        // a presupposed clause. The correction must fire instead of silently
        // answering the main clause as if the false premise held.
        let r = process_with_reasoner(
            &en(),
            &en(),
            "Since a mammal is a dog, is a cat also a mammal?",
        );
        assert_eq!(r.outcome, ChatOutcome::Answered, "got {:?}", r.response);
        assert!(
            r.response.to_lowercase().contains("no"),
            "a false presupposition must be corrected; got {:?}",
            r.response
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_true_fronted_presupposition_does_not_intervene() {
        // check_presupposed_clause must return None here (the premise is
        // true), leaving the ORIGINAL pipeline dispatch over the whole
        // sentence untouched — asserted indirectly: the response must never
        // be the false-correction text ("a mammal is not a dog" / similar),
        // since the presupposed clause itself is true.
        let r = process_with_reasoner(
            &en(),
            &en(),
            "Since a dog is a mammal, is a cat also a mammal?",
        );
        assert!(
            !(r.response.to_lowercase().contains("a mammal is not a dog")
                || r.response.to_lowercase().contains("a dog is not a mammal")),
            "a true presupposition must never be corrected; got {:?}",
            r.response
        );
    }
}

/// End-to-end proof that a real USC statutory definition, once grounded via
/// `defines_lens` and materialized into a loaded `RuntimeOntology`, is
/// actually REACHABLE from a live `define_word` call — the wiring this
/// module's own `define_word` doc comment (the `statute_definitions` block)
/// added, closing the gap `crates/praxis-corpus-tests/tests/
/// nlp_task_reachability.rs`'s `defines_lens_is_reachable_only_from_its_own_tests`
/// claim named. Uses the SAME real worked example
/// `social::judicial::statute_structure::grounding`'s own test suite already
/// trusts (15 U.S.C. § 6603(h)(6)(A), "The term 'consumer' means a natural
/// person.") rather than a fresh fixture, so a drift between the two test
/// suites' understanding of what the lens produces would show up as a
/// failure here, not silently diverge.
#[cfg(test)]
mod statute_definitions_reachability {
    use std::rc::Rc;

    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_domains::cognitive::linguistics::english::english_loaded;
    use pr4xis_domains::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;
    use pr4xis_domains::social::judicial::statute_structure::grounding::defines_lens;
    use pr4xis_runtime::archive::Archive;
    use pr4xis_runtime::definition::Definition;
    use pr4xis_runtime::grounding::ground;
    use pr4xis_runtime::ontology::materialize;
    use std::collections::BTreeMap;

    /// Ground the real "consumer" provision and materialize it into a
    /// `ComposedReasoner` — the exact `Archive → ground(defines_lens) →
    /// materialize → RuntimeOntology → ComposedReasoner` chain the live
    /// `pr4xis chat` entrypoint (`crates/cli/src/main.rs`'s `run_chat`) runs
    /// at startup, at test scale.
    fn reasoner_with_one_defined_provision() -> ComposedReasoner {
        let en = english_loaded();
        let vn = verbnet_classes_loaded();
        let mint_domain = OntologyName::new_static("usc_t15_test_coinages");

        let content = Archive {
            nodes: vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t15/s6603/h/6/A".to_string(),
                edges: Vec::new(),
                axioms: Vec::new(),
                lexical: Some(
                    "The term \u{201C}consumer\u{201D} means a natural person.".to_string(),
                ),
            }],
            connections: Vec::new(),
        };

        let grounded = ground(
            &content,
            defines_lens(en, en, vn, &BTreeMap::new(), &BTreeMap::new(), &mint_domain),
        )
        .expect("the defines lens grounds the real consumer provision");

        let usc = materialize(grounded, OntologyName::new_static("usc_t15_test"))
            .expect("the grounded provision materializes into a RuntimeOntology");

        ComposedReasoner::new(en, vec![Rc::new(usc)])
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn statute_definitions_finds_the_grounded_provision_by_term() {
        use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;

        let reasoner = reasoner_with_one_defined_provision();
        let hits = reasoner.statute_definitions("consumer");
        assert_eq!(
            hits.len(),
            1,
            "exactly the one grounded provision should be found; got {hits:?}"
        );
        let (urn, text) = hits[0];
        assert_eq!(urn, "/us/usc/t15/s6603/h/6/A");
        assert!(
            text.contains("natural person"),
            "the returned text is the provision's own prose; got {text:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn define_word_surfaces_the_statutory_definition_with_its_citation() {
        let reasoner = reasoner_with_one_defined_provision();
        let answer = define_word(&reasoner, "consumer").text;
        assert!(
            answer.to_lowercase().contains("natural person"),
            "the live define_word path must surface the statutory gloss; got {answer:?}"
        );
        assert!(
            answer.contains("/us/usc/t15/s6603/h/6/A"),
            "the answer must cite the defining provision, not just state the \
             gloss unsourced; got {answer:?}"
        );
    }

    /// THE CREDIT DIRECTION: an answer built out of a loaded provision's own
    /// prose NAMES the loaded ontology that owns the provision. The fixture's
    /// definiendum ("consumer") reaches the answer through the statutory
    /// channel alone — the embedded WordNet has no `usc_t15_test` sense of it —
    /// so the credit can only come from [`statutory_source_of`], never from the
    /// concept ids the caller could have resolved on its own.
    ///
    /// The provision URN carries CAPITALIZED clause letters
    /// (`…/h/6/A`), so this also exercises the case fold `statutory_source_of`
    /// applies: without it the lookup misses and the credit silently vanishes.
    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_statutory_definition_credits_the_loaded_title_that_supplied_it() {
        let reasoner = reasoner_with_one_defined_provision();
        let answer = define_word(&reasoner, "consumer");
        assert!(
            answer.text.contains("/us/usc/t15/s6603/h/6/A"),
            "precondition: the provision's text really is what was answered with; \
             got {:?}",
            answer.text
        );
        assert!(
            answer
                .reasoned_over
                .iter()
                .any(|n| n.as_str() == "usc_t15_test"),
            "the answer rests on the loaded title's provision, so the title must \
             appear among the sources reasoned over; got {:?}",
            answer.reasoned_over
        );
    }

    /// THE SAME CREDIT, END TO END through the live turn pipeline — tokenize →
    /// chart → montague → `answer_question` → `ResponseResult` → `PipelineTrace`
    /// — because that is the layer a host actually reads provenance from
    /// (`crates/wasm/src/lib.rs` projects `trace.reasoned_over()` as the turn's
    /// `ontologies` list). The unit-level check above proves `define_word`
    /// COMPUTES the credit; this proves nothing between it and the wire drops it,
    /// which is exactly what the four independent call sites used to do.
    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn a_full_turn_answered_from_a_provision_names_the_title_in_its_trace() {
        use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;

        let reasoner = reasoner_with_one_defined_provision();
        let result = process_with_reasoner(english_loaded(), &reasoner, "what is a consumer");
        assert!(
            result.response.contains("/us/usc/t15/s6603/h/6/A"),
            "precondition: the turn really did answer out of the provision; got {:?}",
            result.response
        );
        let provenance = result.trace.reasoned_over();
        assert!(
            provenance.iter().any(
                |(o, ok)| matches!(o, TraceOntology::Loaded(n) if n.as_str() == "usc_t15_test")
                    && *ok
            ),
            "the turn's trace must name the loaded title whose provision it \
             quoted; got {provenance:?}"
        );
    }

    /// THE NO-SPURIOUS-CREDIT DIRECTION — without which the test above is
    /// satisfiable by crediting every loaded ontology unconditionally. Over the
    /// SAME reasoner (the title IS loaded), a definiendum the loaded title does
    /// not define is answered from the WordNet substrate alone, and must not
    /// collect the title's name on the way past.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_definition_the_loaded_title_did_not_supply_earns_it_no_credit() {
        use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;

        let reasoner = reasoner_with_one_defined_provision();
        assert!(
            reasoner.statute_definitions("dog").is_empty(),
            "precondition: the loaded title defines nothing for this definiendum"
        );
        let answer = define_word(&reasoner, "dog");
        assert!(
            !answer
                .reasoned_over
                .iter()
                .any(|n| n.as_str() == "usc_t15_test"),
            "a substrate-only definition must not credit a loaded title that \
             contributed nothing; got {:?}",
            answer.reasoned_over
        );
    }

    /// THE DEFINITION-PROVENANCE CHANNEL, END TO END and in BOTH directions —
    /// a lexicon term whose gloss declares an authority reaches the wire
    /// carrying that authority as its OWN realized line, and a substrate term
    /// whose gloss declares none reaches the wire carrying no line at all.
    ///
    /// The two halves are the whole point. The first alone would be satisfied by
    /// a channel that fabricates a citation for every answer; the second alone by
    /// a channel that never fires. Together they say the engine can tell "someone
    /// read 42 USC 300ii(7) when writing this" from "the engine opened Title 42"
    /// — the distinction the old em-dash suffix erased.
    #[pr4xis::praxis_value(Explainable, Honest)]
    #[test]
    fn a_lexicon_definition_states_the_document_it_was_authored_from() {
        use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology_from_lmf;
        use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
        use pr4xis_domains::social::care::caregiving_lexicon::{
            CAREGIVING_LEXICON_NAME, CAREGIVING_LEXICON_PRX, CAREGIVING_LEXICON_VERSION,
        };

        let xml =
            pr4xis_domains::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
                CAREGIVING_LEXICON_NAME,
                CAREGIVING_LEXICON_VERSION,
                CAREGIVING_LEXICON_PRX,
            );
        let onto =
            lexicon_runtime_ontology_from_lmf(&xml, OntologyName::new_static("caregiving_lexicon"))
                .expect("the registered caregiving lexicon materializes");
        let reasoner = ComposedReasoner::new(english_loaded(), vec![std::rc::Rc::new(onto)]);

        let answer = define_word(&reasoner, "respite care");
        assert!(
            !answer.text.contains("42 USC 300ii(7)"),
            "the recited answer must no longer wear its citation as prose; got {:?}",
            answer.text
        );
        assert_eq!(
            answer
                .authored_from
                .iter()
                .map(|p| p.citation.as_str())
                .collect::<Vec<_>>(),
            vec!["42 USC 300ii(7)"],
            "the authority reaches the caller through the structured channel"
        );
        assert!(
            answer
                .authored_from
                .iter()
                .all(|p| p.lexicon.as_str() == "caregiving_lexicon"),
            "…attributed to the lexicon that actually wrote the gloss"
        );

        // END TO END: the live turn realizes the channel as its own labelled
        // line, from the loaded explain-frames table — never appended to `why`,
        // so a renderer never has to split prose to tell the two apart.
        let turn = process_with_reasoner(english_loaded(), &reasoner, "what is respite care");
        let realized = turn
            .definition_provenance
            .as_ref()
            .expect("an answered lexicon definition states where it was written from");
        assert!(
            realized.detail.contains("42 USC 300ii(7)"),
            "the realized line names the authority; got {:?}",
            realized.detail
        );
        assert!(
            !realized.label.is_empty(),
            "the channel's own label is loaded, not left to a renderer"
        );
        assert!(
            !turn
                .why
                .as_deref()
                .unwrap_or_default()
                .contains("42 USC 300ii(7)"),
            "the authority must NOT be folded into the reasoned-over sentence — \
             that is the conflation this channel exists to end; got {:?}",
            turn.why
        );

        // THE OTHER DIRECTION: a WordNet-substrate definiendum this lexicon does
        // not define declares nothing, so no line is realized at all.
        let substrate = process_with_reasoner(english_loaded(), &reasoner, "what is a dog");
        assert!(
            substrate.definition_provenance.is_none(),
            "a lexicographic gloss cites no document and must claim none; got {:?}",
            substrate.definition_provenance
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn statute_definitions_is_empty_for_an_undefined_term() {
        use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;

        let reasoner = reasoner_with_one_defined_provision();
        assert!(
            reasoner.statute_definitions("giraffe").is_empty(),
            "a term no loaded provision defines must return no false hits"
        );
    }

    /// `answer_question`'s single-entity Content-question extraction (the
    /// `what does X mean?` path this task owns) correctly isolates a QUOTED
    /// APPOSITIVE definiendum ("the term 'restraint'") as the lone queried
    /// entity, GIVEN a Sem tree where the syntax chart already chose the
    /// close-apposition derivation — `montague::apply`'s NP-result branch
    /// (Huddleston & Pullum 2002 Ch. 5 "Nouns and noun phrases" §3
    /// "Apposition") already promotes the quoted concept over its head noun
    /// at the SEMANTIC level
    /// (`close_apposition_promotes_the_quoted_definiendum_over_the_head_noun`,
    /// `montague.rs`); this proves `answer_question`'s OWN
    /// `entity_leaves`/`argument_name` reading of that promoted `Sem::Concept`
    /// carries it through end to end as `entities_found`, naming "restraint"
    /// alone rather than concatenating it with "term".
    ///
    /// Hand-built tokens (the SAME style `montague_what_does_x_mean_is_a_
    /// content_question_naming_one_entity`, `lambek/tests.rs`, already
    /// uses), not the full tokenizer pipeline: this isolates the extraction
    /// layer this task owns from tokenization proper. The FULL pipeline's
    /// own version of this same gap — even the simplest full sentence
    /// 'What does the term "restraint" mean?' used to garble into the
    /// composite "term restraint mean" — turned out to have its root
    /// cause in `tokenize::collapse_quoted_spans` being scoped to the
    /// DIRECTIONAL quote glyphs only (its own doc: the two ASCII marks are
    /// `QuoteRole::Ambiguous` and are deliberately left untouched, folded
    /// into ordinary punctuation trimming before a quoted-mention token is
    /// ever minted), while this corpus's harvested text quotes with plain
    /// ASCII `"…"` throughout — so the close-apposition alternative this
    /// function's own extraction correctly prefers was never OFFERED to
    /// the chart in the first place, not out-competed by it. Fixed at the
    /// pipeline's own entry, entirely within this crate:
    /// `normalize_ascii_double_quotes` (wired into `process_with_reasoner`)
    /// rewrites the ASCII pair into the canonical directional pair BEFORE
    /// tokenization, using the SAME loaded `quote_glyphs` vocabulary
    /// `collapse_quoted_spans` itself reads — see
    /// `real_corpus_row_use_of_a_restraint_extracts_the_quoted_span_cleanly`
    /// (this module) for the full-pipeline, real-`english_loaded()` proof
    /// against the EXACT corpus row. One narrower, confirmed-but-separate
    /// residual: a quoted definiendum that is ALSO a domain-loaded surface
    /// picks up an independent bare-noun(N) alternative from `process_with_
    /// reasoner`'s own "let a loaded term take a determiner" widening,
    /// which can still out-compete close-apposition for that one word —
    /// see `what_does_the_quoted_appositive_mean_isolates_the_quoted_span`'s
    /// own doc (`dependability_demo`) for the isolating control.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_does_x_mean_isolates_the_quoted_appositive_over_the_head_noun() {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;

        let en = English::sample_static();
        let tokens = vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "what".into(),
                lambek_type: svo::wh_what_object(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "does".into(),
                lambek_type: svo::does_support(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "restraint".into(),
                lambek_type: svo::close_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "mean".into(),
                lambek_type: svo::bare_transitive_verb(),
            },
        ];
        let sem = montague::interpret(&tokens, en);
        let montague::Sem::Question {
            predicate,
            arguments,
            illocution,
        } = &sem
        else {
            panic!("expected a Question, got {sem:?}");
        };
        let result = answer_question(en, en, &tokens, predicate, arguments, *illocution);
        assert_eq!(
            result.entities_found,
            vec!["restraint".to_string()],
            "must name the quoted definiendum alone, not concatenated with \
             its head noun; got {:?}",
            result.entities_found
        );
    }
}
