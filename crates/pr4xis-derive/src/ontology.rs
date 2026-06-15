use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitStr, Token, braced, bracketed, parenthesized};

/// The transitive relation-kind vocabulary — a DISTILLED, drift-guarded cache of
/// the Relations ontology's `(R, Transitive, HasProperty)` declarations
/// (`crates/domains/src/formal/relations/ontology.rs`), the ONE source of truth.
/// `pr4xis-derive` is the bottom of the dependency graph and CANNOT read Relations
/// as compiled Rust, so the macro reads this projection of it — the build-time
/// half of "one declaration, two readers" (the runtime half is `pr4xis-runtime`'s
/// `declared_transitive_kinds`, which reads its own copy of the same cache). The
/// file is regenerated from `emit::<RelationsCategory>()` and drift-guarded by a
/// `transitive_kinds()` re-derivation in the `domains` test suite; a hand-edit or
/// a stale cache fails that test. This replaces the former hardcoded
/// `Subsumption / Parthood / Causation` allowlist (the last "code is ontological"
/// hardcode in the closure tiers).
const RELATIONS_TRANSITIVE_KINDS_SRC: &str = include_str!("relations_transitive_kinds.txt");

struct LabelEntry {
    concept: Ident,
    lang: LitStr,
    label: LitStr,
    definition: Option<LitStr>,
}

impl Parse for LabelEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let concept: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        parenthesized!(content in input);
        let lang: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let label: LitStr = content.parse()?;
        let definition = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            Some(content.parse::<LitStr>()?)
        } else {
            None
        };
        Ok(LabelEntry {
            concept,
            lang,
            label,
            definition,
        })
    }
}

struct EdgeEntry {
    from: Ident,
    to: Ident,
    kind: Ident,
}

impl Parse for EdgeEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let from: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let to: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let kind: Ident = content.parse()?;
        Ok(EdgeEntry { from, to, kind })
    }
}

struct PairEntry {
    a: Ident,
    b: Ident,
}

impl Parse for PairEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let a: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let b: Ident = content.parse()?;
        Ok(PairEntry { a, b })
    }
}

struct ComposedEntry {
    from: Ident,
    to: Ident,
}

impl Parse for ComposedEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let from: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let to: Ident = content.parse()?;
        Ok(ComposedEntry { from, to })
    }
}

/// A domain axiom declared inside `ontology!`'s `axioms:` clause.
///
/// Shape:
/// ```text
/// axioms: {
///     FourPhaseCycle: {
///         source: "Kephart & Chess (2003) §2",
///         description: "The four operational phases are the children of MapeKPhase.",
///         holds: { /* bool expression */ },
///     },
/// }
/// ```
struct AxiomEntry {
    name: Ident,
    source: LitStr,
    description: LitStr,
    holds: Expr,
}

impl Parse for AxiomEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        braced!(content in input);

        let mut source: Option<LitStr> = None;
        let mut description: Option<LitStr> = None;
        let mut holds: Option<Expr> = None;

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "source" => source = Some(content.parse()?),
                "description" => description = Some(content.parse()?),
                "holds" => holds = Some(content.parse()?),
                other => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!(
                            "unknown axiom field: {other} (expected source / description / holds)"
                        ),
                    ));
                }
            }
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        let missing =
            |f: &str| syn::Error::new_spanned(&name, format!("axiom '{name}' missing field: {f}"));
        Ok(AxiomEntry {
            source: source.ok_or_else(|| missing("source"))?,
            description: description.ok_or_else(|| missing("description"))?,
            holds: holds.ok_or_else(|| missing("holds"))?,
            name,
        })
    }
}

pub struct OntologyDef {
    name: LitStr,
    source: Option<LitStr>,
    concepts: Vec<Ident>,
    labels: Vec<LabelEntry>,
    edges: Vec<EdgeEntry>,
    composed: Vec<ComposedEntry>,
    is_a: Vec<PairEntry>,
    has_a: Vec<PairEntry>,
    causes: Vec<PairEntry>,
    opposes: Vec<PairEntry>,
    axioms: Vec<AxiomEntry>,
}

impl Parse for OntologyDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut source = None;
        let mut concepts = Vec::new();
        let mut labels = Vec::new();
        let mut edges = Vec::new();
        let mut composed = Vec::new();
        let mut is_a = Vec::new();
        let mut has_a = Vec::new();
        let mut causes = Vec::new();
        let mut opposes = Vec::new();
        let mut axioms: Vec<AxiomEntry> = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<LitStr>()?);
                }
                "source" => {
                    source = Some(input.parse::<LitStr>()?);
                }
                "concepts" => {
                    let content;
                    bracketed!(content in input);
                    concepts = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "labels" => {
                    let content;
                    braced!(content in input);
                    labels = Punctuated::<LabelEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "edges" => {
                    let content;
                    bracketed!(content in input);
                    edges = Punctuated::<EdgeEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "composed" => {
                    let content;
                    bracketed!(content in input);
                    composed = Punctuated::<ComposedEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "is_a" => {
                    let content;
                    bracketed!(content in input);
                    is_a = Punctuated::<PairEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "has_a" => {
                    let content;
                    bracketed!(content in input);
                    has_a = Punctuated::<PairEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "causes" => {
                    let content;
                    bracketed!(content in input);
                    causes = Punctuated::<PairEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "opposes" => {
                    let content;
                    bracketed!(content in input);
                    opposes = Punctuated::<PairEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "axioms" => {
                    let content;
                    braced!(content in input);
                    axioms = Punctuated::<AxiomEntry, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("unknown field: {other}"),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let name = name.ok_or_else(|| input.error("missing required field: name"))?;

        Ok(OntologyDef {
            name,
            source,
            concepts,
            labels,
            edges,
            composed,
            is_a,
            has_a,
            causes,
            opposes,
            axioms,
        })
    }
}

pub fn generate(def: OntologyDef) -> TokenStream {
    let pr4xis = crate::pr4xis_crate();
    let name_str = def.name.value();

    // Validate name forms a valid Rust identifier when suffixed.
    let make_ident = |suffix: &str| -> Result<Ident, syn::Error> {
        let candidate = format!("{name_str}{suffix}");
        syn::parse_str::<Ident>(&candidate).map_err(|_| {
            syn::Error::new_spanned(
                &def.name,
                format!(
                    "ontology name '{name_str}' must form valid Rust identifiers; \
                     '{candidate}' is not a valid identifier (only ASCII letters, digits, \
                     and underscores; cannot start with a digit)"
                ),
            )
        })
    };
    let ont_name = match make_ident("Ontology") {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let cat_name = match make_ident("Category") {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let entity_name = match make_ident("Concept") {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let relation_name = match make_ident("Relation") {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    let concept_idents = &def.concepts;
    let concept_names: Vec<String> = concept_idents.iter().map(|c| c.to_string()).collect();

    // Validate that edges/is_a/has_a/causes/opposes reference declared concepts
    let concept_set: std::collections::HashSet<String> = concept_names.iter().cloned().collect();

    for edge in &def.edges {
        if !concept_set.contains(&edge.from.to_string()) {
            return syn::Error::new_spanned(
                &edge.from,
                format!("concept '{}' not declared in concepts list", edge.from),
            )
            .to_compile_error();
        }
        if !concept_set.contains(&edge.to.to_string()) {
            return syn::Error::new_spanned(
                &edge.to,
                format!("concept '{}' not declared in concepts list", edge.to),
            )
            .to_compile_error();
        }
    }

    for pair in def
        .is_a
        .iter()
        .chain(def.has_a.iter())
        .chain(def.causes.iter())
        .chain(def.opposes.iter())
    {
        if !concept_set.contains(&pair.a.to_string()) {
            return syn::Error::new_spanned(
                &pair.a,
                format!("concept '{}' not declared in concepts list", pair.a),
            )
            .to_compile_error();
        }
        if !concept_set.contains(&pair.b.to_string()) {
            return syn::Error::new_spanned(
                &pair.b,
                format!("concept '{}' not declared in concepts list", pair.b),
            )
            .to_compile_error();
        }
    }

    for comp in &def.composed {
        if !concept_set.contains(&comp.from.to_string()) {
            return syn::Error::new_spanned(
                &comp.from,
                format!("concept '{}' not declared in concepts list", comp.from),
            )
            .to_compile_error();
        }
        if !concept_set.contains(&comp.to.to_string()) {
            return syn::Error::new_spanned(
                &comp.to,
                format!("concept '{}' not declared in concepts list", comp.to),
            )
            .to_compile_error();
        }
    }

    for label in &def.labels {
        if !concept_set.contains(&label.concept.to_string()) {
            return syn::Error::new_spanned(
                &label.concept,
                format!("concept '{}' not declared in concepts list", label.concept),
            )
            .to_compile_error();
        }
    }

    // Synthesise edges from sugar clauses (is_a / has_a / causes / opposes)
    // — they desugar to kinded edges whose names match concepts in the
    // Relations umbrella ontology (`formal::relations`):
    //
    //   is_a:    (child, parent)      → (child, parent, Subsumption)
    //   has_a:   (whole, part)        → (PART, WHOLE, Parthood)
    //   causes:  (cause, effect)      → (cause, effect, Causation)
    //   opposes: (a, b)               → (a, b, Opposition)
    //
    // NOTE the has_a EDGE DIRECTION: the clause reads "whole has-a part", but the
    // emitted Parthood edge is PART→WHOLE, because the canonical mereology relation
    // is part_of(x, y) = "x is part of y" (BFO:0000050; OBO-RO `part of`; praxis's
    // own `formal::mereology` `(Part, Whole, ComposesInto)`). So `reaches(part,
    // whole, Parthood)` holds — the same orientation the USC corpus bridge and the
    // "is X part of Y" chat query assume. A whole→part edge would be the inverse
    // (`has_part`) and would answer "is X part of Y" backwards.
    //
    // Per Gruber (1993) / OBO-RO (Smith 2005), every morphism carries a
    // canonical relation-kind tag. Dense (unkinded) categories are
    // ontologically illegitimate and not emitted.
    struct SugarEdge {
        from: Ident,
        to: Ident,
        kind: Ident,
    }
    let mut sugar_edges: Vec<SugarEdge> = Vec::new();
    for p in &def.is_a {
        sugar_edges.push(SugarEdge {
            from: p.a.clone(),
            to: p.b.clone(),
            kind: format_ident!("Subsumption"),
        });
    }
    for p in &def.has_a {
        // `has_a: (whole, part)` → edge PART→WHOLE (part_of, BFO:0000050): the part
        // is the source, the whole the target, so `reaches(part, whole, Parthood)`.
        sugar_edges.push(SugarEdge {
            from: p.b.clone(),
            to: p.a.clone(),
            kind: format_ident!("Parthood"),
        });
    }
    for p in &def.causes {
        sugar_edges.push(SugarEdge {
            from: p.a.clone(),
            to: p.b.clone(),
            kind: format_ident!("Causation"),
        });
    }
    for p in &def.opposes {
        // Opposition is symmetric per Smith (2005) OBO-RO and Tarski (1941)
        // *Calculus of Relations*: `oppositeOf(a, b) ⟺ oppositeOf(b, a)`.
        // The `SymmetricOnKind` structural axiom enforces this — auto-emit
        // both directions from a single `opposes:` entry so authors don't
        // have to spell out reciprocals.
        sugar_edges.push(SugarEdge {
            from: p.a.clone(),
            to: p.b.clone(),
            kind: format_ident!("Opposition"),
        });
        if p.a != p.b {
            sugar_edges.push(SugarEdge {
                from: p.b.clone(),
                to: p.a.clone(),
                kind: format_ident!("Opposition"),
            });
        }
    }

    // Collect all edge-from/to/kind — custom `edges:` clause plus sugar.
    let all_edge_from: Vec<Ident> = def
        .edges
        .iter()
        .map(|e| e.from.clone())
        .chain(sugar_edges.iter().map(|e| e.from.clone()))
        .collect();
    let all_edge_to: Vec<Ident> = def
        .edges
        .iter()
        .map(|e| e.to.clone())
        .chain(sugar_edges.iter().map(|e| e.to.clone()))
        .collect();
    let all_edge_kind: Vec<Ident> = def
        .edges
        .iter()
        .map(|e| e.kind.clone())
        .chain(sugar_edges.iter().map(|e| e.kind.clone()))
        .collect();

    // Unique kinds — preserves declaration order (custom edges first, then
    // the canonical sugar-derived kinds in is_a/has_a/causes/opposes order).
    //
    // The four canonical Relations-ontology kinds (Subsumption / Parthood /
    // Causation / Opposition — per OBO-RO Smith 2005, SKOS, DOLCE) are always
    // emitted, even if the ontology has no edges of that kind. Without this,
    // cross-ontology functor authors would have to write `match` arms whose
    // variant existence depends on the source's sugar clauses, which makes
    // `F(g∘f) = F(g)∘F(f)` (Mac Lane CWM Ch. II §1) impossible to satisfy
    // generically. A kind variant with no edges is harmless: `morphisms()`
    // only emits actual edges, so `ClosureLaw` is unaffected.
    let canonical_kinds = ["Subsumption", "Parthood", "Causation", "Opposition"];
    let unique_kinds: Vec<Ident> = {
        let mut seen = std::collections::HashSet::new();
        let mut acc: Vec<Ident> = all_edge_kind
            .iter()
            .filter(|k| seen.insert(k.to_string()))
            .cloned()
            .collect();
        for k in &canonical_kinds {
            if seen.insert(k.to_string()) {
                acc.push(format_ident!("{}", k));
            }
        }
        acc
    };

    // Generate label static data
    let label_entries: Vec<TokenStream> = def
        .labels
        .iter()
        .map(|l| {
            let concept = &l.concept;
            let lang = &l.lang;
            let label = &l.label;
            let def_str = l
                .definition
                .as_ref()
                .map(|d| quote! { #d })
                .unwrap_or(quote! { "" });
            quote! {
                (#entity_name::#concept, #lang, #label, #def_str)
            }
        })
        .collect();

    // Per-concept `Concept::lexical()` override arms — the same labels table,
    // surfaced through the trait so each concept carries its ONTOLEX-Lemon
    // entry (label + gloss + language) WITH it (e.g. into an emitted `.prx`),
    // not only in the compile-time `labels()` side table. A concept with no
    // label entry falls through to `None` (honest absence). Sourced from the
    // macro's labels — never hardcoded.
    let lexical_arms: Vec<TokenStream> = def
        .labels
        .iter()
        .map(|l| {
            let concept = &l.concept;
            let lang = &l.lang;
            let label = &l.label;
            let def_str = l
                .definition
                .as_ref()
                .map(|d| quote! { #d })
                .unwrap_or(quote! { "" });
            quote! {
                #entity_name::#concept => Some(#pr4xis::ontology::meta::Lexical::new(
                    #pr4xis::ontology::meta::Label::new_static(#label),
                    #pr4xis::ontology::meta::Definition::new_static(#def_str),
                    #pr4xis::ontology::meta::LanguageCode::new_static(#lang),
                )),
            }
        })
        .collect();

    // Source
    let source_tokens = def
        .source
        .as_ref()
        .map(|s| quote! { #s })
        .unwrap_or(quote! { "" });

    // No `being:` clause — #165 removed DOLCE from core entirely. Upper-
    // ontology classification (if any) is a domain concern.
    let being_classified: Option<proc_macro2::TokenStream> = None;

    // Generate taxonomy
    let tax_name = format_ident!("{}Taxonomy", name_str);
    let tax_pairs: Vec<TokenStream> = def
        .is_a
        .iter()
        .map(|p| {
            let a = &p.a;
            let b = &p.b;
            quote! { (#entity_name::#a, #entity_name::#b) }
        })
        .collect();
    let has_taxonomy = !def.is_a.is_empty();

    // Generate mereology
    let mer_name = format_ident!("{}Mereology", name_str);
    let mer_pairs: Vec<TokenStream> = def
        .has_a
        .iter()
        .map(|p| {
            let a = &p.a;
            let b = &p.b;
            quote! { (#entity_name::#a, #entity_name::#b) }
        })
        .collect();
    let has_mereology = !def.has_a.is_empty();

    // Generate causation
    let caus_name = format_ident!("{}Causation", name_str);
    let caus_pairs: Vec<TokenStream> = def
        .causes
        .iter()
        .map(|p| {
            let a = &p.a;
            let b = &p.b;
            quote! { (#entity_name::#a, #entity_name::#b) }
        })
        .collect();
    let has_causation = !def.causes.is_empty();

    // Generate opposition
    let opp_name = format_ident!("{}Opposition", name_str);
    let opp_pairs: Vec<TokenStream> = def
        .opposes
        .iter()
        .map(|p| {
            let a = &p.a;
            let b = &p.b;
            quote! { (#entity_name::#a, #entity_name::#b) }
        })
        .collect();
    let has_opposition = !def.opposes.is_empty();

    // Category generation — always kinded (issue #152: no dense categories).
    // Every morphism carries a relation-kind tag; edges come from the
    // `edges:` clause plus the sugar-clause synthesis above.
    let category_impl = {
        let kind_name = format_ident!("{}RelationKind", name_str);

        // Compute transitive closure of all declared edges (Floyd-Warshall at
        // macro expansion). Ensures morphisms() is closed under compose() —
        // if (A,B) and (B,C) exist, (A,C) is included as Composed.
        let direct_pairs: std::collections::BTreeSet<(String, String)> = all_edge_from
            .iter()
            .zip(all_edge_to.iter())
            .map(|(f, t)| (f.to_string(), t.to_string()))
            .collect();
        let mut closure: std::collections::BTreeSet<(String, String)> = direct_pairs.clone();
        loop {
            let mut added = false;
            let snapshot: Vec<(String, String)> = closure.iter().cloned().collect();
            for (a, b) in &snapshot {
                for (b2, c) in &snapshot {
                    if b == b2 && !closure.contains(&(a.clone(), c.clone())) {
                        closure.insert((a.clone(), c.clone()));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        // Composed pairs = transitive closure minus direct edges + explicit composed: list.
        let mut composed_pairs: std::collections::BTreeSet<(String, String)> =
            closure.difference(&direct_pairs).cloned().collect();
        for c in &def.composed {
            composed_pairs.insert((c.from.to_string(), c.to.to_string()));
        }
        // These pre-computed closure pairs are kind-less; we no longer
        // emit them (path-(a) partial category, #166). Reference them to
        // silence unused-var warnings while keeping the computation in
        // case future rework wants per-kind or heterogeneous closure.
        let _comp_from: Vec<Ident> = composed_pairs
            .iter()
            .map(|(f, _)| format_ident!("{}", f))
            .collect();
        let _comp_to: Vec<Ident> = composed_pairs
            .iter()
            .map(|(_, t)| format_ident!("{}", t))
            .collect();

        let edge_from = &all_edge_from;
        let edge_to = &all_edge_to;
        let edge_kind = &all_edge_kind;

        // Same-kind transitive inheritance per OBO-RO `transitive_over`
        // (Smith 2005). WHICH kinds are transitive is LOADED from the Relations
        // vocabulary cache (`RELATIONS_TRANSITIVE_KINDS_SRC`), not a hardcoded
        // allowlist — the build-time reader of the one `(R, Transitive,
        // HasProperty)` declaration. Same-kind composition inherits; heterogeneous
        // returns None (partial category, #166).
        let transitive_names: std::collections::BTreeSet<&str> = RELATIONS_TRANSITIVE_KINDS_SRC
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let transitive_kind_arms: Vec<proc_macro2::TokenStream> = unique_kinds
            .iter()
            .filter(|k| transitive_names.contains(k.to_string().as_str()))
            .map(|k| {
                quote! {
                    (#kind_name::#k, #kind_name::#k) => Some(#kind_name::#k),
                }
            })
            .collect();

        // Transitive-closure edges for same-kind transitive relations —
        // these MUST appear in morphisms() so the closure law holds when
        // compose emits them. Heterogeneous closure paths are NOT emitted
        // (they aren't typed morphisms absent a declared rule).
        //
        // Same-kind transitive filter: for each (a, b) path in the
        // closure where every edge in the path has the SAME transitive
        // kind, emit (a, b, kind) as a morphism.
        // The pre-computed `composed_pairs` is kind-less; we re-filter by
        // rebuilding per-kind closure from direct edges.
        let transitive_kind_names: Vec<String> = unique_kinds
            .iter()
            .map(|k| k.to_string())
            .filter(|s| transitive_names.contains(s.as_str()))
            .collect();
        let mut per_kind_closure_morphisms: Vec<proc_macro2::TokenStream> = Vec::new();
        for tkind in &transitive_kind_names {
            let kind_ident = format_ident!("{}", tkind);
            let direct_pairs_of_kind: Vec<(String, String)> = all_edge_from
                .iter()
                .zip(all_edge_to.iter())
                .zip(all_edge_kind.iter())
                .filter(|((_, _), k)| k.to_string() == *tkind)
                .map(|((f, t), _)| (f.to_string(), t.to_string()))
                .collect();
            let direct_set: std::collections::BTreeSet<(String, String)> =
                direct_pairs_of_kind.iter().cloned().collect();
            let mut closure_set = direct_set.clone();
            loop {
                let mut added = false;
                let snapshot: Vec<(String, String)> = closure_set.iter().cloned().collect();
                for (a, b) in &snapshot {
                    for (b2, c) in &snapshot {
                        if b == b2 && !closure_set.contains(&(a.clone(), c.clone())) {
                            closure_set.insert((a.clone(), c.clone()));
                            added = true;
                        }
                    }
                }
                if !added {
                    break;
                }
            }
            let transitive_only: std::collections::BTreeSet<(String, String)> =
                closure_set.difference(&direct_set).cloned().collect();
            for (f, t) in &transitive_only {
                let f_id = format_ident!("{}", f);
                let t_id = format_ident!("{}", t);
                per_kind_closure_morphisms.push(quote! {
                    m.push(#relation_name {
                        from: #entity_name::#f_id,
                        to: #entity_name::#t_id,
                        kind: #kind_name::#kind_ident,
                    });
                });
            }
        }

        quote! {
            /// Morphism-kind tag. Kinds follow the Relations umbrella
            /// ontology conventions (OBO-RO / SKOS / Gruber).
            /// No `Composed` variant (#166): composition of typed morphisms
            /// is partial per OBO-RO — same-kind transitive inherits;
            /// heterogeneous returns None absent a declared rule.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #kind_name {
                Identity,
                #(#unique_kinds,)*
            }

            impl #kind_name {
                /// Every relation-kind variant — the closed-world enumeration
                /// of this ontology's morphism kinds (the kind-level parallel
                /// of [`Concept::variants`](#pr4xis::category::Concept::variants)).
                pub fn variants() -> Vec<Self> {
                    vec![#kind_name::Identity, #(#kind_name::#unique_kinds,)*]
                }

                /// The variant's canonical name — the morphism kind's lexical
                /// grounding (the kind-level parallel of
                /// [`Concept::name`](#pr4xis::category::Concept::name)), so a
                /// kind is carried/compared by stable name, never a positional
                /// discriminant.
                pub fn name(&self) -> &'static str {
                    match self {
                        #kind_name::Identity => "Identity",
                        #(#kind_name::#unique_kinds => stringify!(#unique_kinds),)*
                    }
                }

                /// Re-resolve a kind from its canonical [`name`](Self::name) —
                /// the typed inverse, over `variants()`; fail-closed `None` for
                /// an unknown name.
                pub fn from_name(name: &str) -> Option<Self> {
                    #kind_name::variants().into_iter().find(|k| k.name() == name)
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #relation_name {
                pub from: #entity_name,
                pub to: #entity_name,
                pub kind: #kind_name,
            }

            impl #pr4xis::category::Arrow for #relation_name {
                type Object = #entity_name;
                type Kind = #kind_name;
                fn source(&self) -> #entity_name { self.from }
                fn target(&self) -> #entity_name { self.to }
                fn kind(&self) -> #kind_name { self.kind }

                fn meta(&self) -> #pr4xis::ontology::meta::Provenance {
                    #pr4xis::ontology::meta::Provenance {
                        name: #pr4xis::ontology::meta::OntologyName::new(
                            format!("{:?}-[{:?}]-{:?}", self.from, self.kind, self.to)
                        ),
                        description: #pr4xis::ontology::meta::Label::new(
                            format!("{:?} -[{:?}]-> {:?}", self.from, self.kind, self.to)
                        ),
                        citation: #pr4xis::ontology::meta::Citation::parse_static(#source_tokens),
                        module_path: #pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
                    }
                }
            }

            pub struct #cat_name;

            impl #pr4xis::category::Category for #cat_name {
                type Object = #entity_name;
                type Morphism = #relation_name;

                fn identity(obj: &#entity_name) -> #relation_name {
                    #relation_name { from: *obj, to: *obj, kind: #kind_name::Identity }
                }

                fn compose(f: &#relation_name, g: &#relation_name) -> Option<#relation_name> {
                    if f.to != g.from { return None; }
                    if f.kind == #kind_name::Identity { return Some(g.clone()); }
                    if g.kind == #kind_name::Identity { return Some(f.clone()); }
                    // Per OBO-RO (Smith 2005) `transitive_over`: same-kind
                    // transitive composition inherits the kind. All other
                    // compositions return None (partial category — #166).
                    let composed_kind: Option<#kind_name> = match (f.kind, g.kind) {
                        #(#transitive_kind_arms)*
                        _ => None,
                    };
                    composed_kind.map(|k| #relation_name { from: f.from, to: g.to, kind: k })
                }

                fn morphisms() -> Vec<#relation_name> {
                    use #pr4xis::category::FinitelyGenerated;
                    let mut m = Vec::new();
                    // Identity morphisms
                    for c in #entity_name::variants() {
                        m.push(#relation_name { from: c, to: c, kind: #kind_name::Identity });
                    }
                    // Direct edges (declared via sugar + `edges:`)
                    #(m.push(#relation_name { from: #entity_name::#edge_from, to: #entity_name::#edge_to, kind: #kind_name::#edge_kind });)*
                    // Transitive closure edges for same-kind transitive relations
                    // (per OBO-RO). No heterogeneous closure edges are emitted
                    // absent declared composition rules (#166).
                    #(#per_kind_closure_morphisms)*
                    m
                }
            }
        }
    };

    // Per-def reasoning traits (TaxonomyDef/MereologyDef/CausalDef/OppositionDef)
    // deleted per #169 — relations are kinded morphisms in the category,
    // and structural axioms come from `structural_axioms_for<Cat>()` via
    // the catalog. Sugar clauses (is_a/has_a/causes/opposes) are desugared
    // into kinded edges directly in `morphisms()`.
    let _ = (&tax_pairs, &mer_pairs, &caus_pairs, &opp_pairs);
    let _ = (&tax_name, &mer_name, &caus_name, &opp_name);
    let taxonomy_impl: proc_macro2::TokenStream = quote! {};
    let mereology_impl: proc_macro2::TokenStream = quote! {};
    let causation_impl: proc_macro2::TokenStream = quote! {};
    let opposition_impl: proc_macro2::TokenStream = quote! {};

    // Structural axioms now come from the catalog via
    // `structural_axioms_for::<Cat>()` — no per-ontology re-emission of
    // `NoCycles<TaxonomyDef>` / `Antisymmetric<TaxonomyDef>` etc. (#168).
    // The catalog walks the category's morphisms, groups by typed
    // relation kind, and emits OnKind axioms canonical per OBO-RO.
    let _ = (has_taxonomy, has_mereology, has_causation, has_opposition);

    // Vocabulary.name matches the generated struct name (e.g. `name: "Foo"`
    // → Vocabulary.name = "FooOntology"). This keeps parity with the older
    // declarative `define_ontology!` macro, which used the struct ident
    // directly. Without this suffix the new proc macro would emit "Foo"
    // while unmigrated ontologies emit "FooOntology", producing an
    // inconsistent knowledge-base registry.
    let full_name = format!("{name_str}Ontology");
    let name_lit = syn::LitStr::new(&full_name, def.name.span());

    // Domain axioms declared in the `axioms:` clause. Each one emits a
    // unit struct + `impl Axiom` + `Provenance`, and gets pushed into
    // the ontology's `domain_axioms()` via the `axiom_push_calls` list.
    let axiom_structs: Vec<TokenStream> = def
        .axioms
        .iter()
        .map(|a| {
            let name_ident = &a.name;
            let source = &a.source;
            let description = &a.description;
            let holds = &a.holds;
            let name_str_lit = syn::LitStr::new(&a.name.to_string(), a.name.span());
            quote! {
                #[doc = #description]
                pub struct #name_ident;

                impl #pr4xis::logic::axiom::Axiom for #name_ident {
                    fn verify(&self) -> #pr4xis::logic::proof::Verdict {
                        if #holds {
                            Ok(Box::new(#pr4xis::logic::proof::SimpleProof::new(
                                <Self as #pr4xis::logic::axiom::Axiom>::meta(self),
                            )))
                        } else {
                            Err(Box::new(#pr4xis::logic::proof::SimpleCounterexample::new(
                                <Self as #pr4xis::logic::axiom::Axiom>::meta(self),
                            )))
                        }
                    }

                    fn citation(&self) -> #pr4xis::ontology::meta::Citation {
                        #pr4xis::ontology::meta::Citation::parse_static(#source)
                    }

                    fn name(&self) -> #pr4xis::ontology::meta::OntologyName {
                        #pr4xis::ontology::meta::OntologyName::new_static(#name_str_lit)
                    }

                    fn description(&self) -> #pr4xis::ontology::meta::Label {
                        #pr4xis::ontology::meta::Label::new_static(#description)
                    }
                }
            }
        })
        .collect();

    let axiom_domain_pushes: Vec<TokenStream> = def
        .axioms
        .iter()
        .map(|a| {
            let name_ident = &a.name;
            quote! { axioms.push(Box::new(#name_ident)); }
        })
        .collect();

    // Per-axiom auto-registration. Each declared domain axiom is a unit struct
    // (constructible from its identifier), so we register it the same way
    // `register_axiom!(Name, constructor)` does — into BOTH slices:
    //
    //  * AXIOMS (metadata) — so the Lemon lexicon includes every declared axiom.
    //  * AXIOM_CONSTRUCTORS (the re-bind table) — so a serialized domain-axiom
    //    node (emitted by `pr4xis-runtime::emit` keyed by `Axiom::name`)
    //    re-binds to a freshly-built predicate by that same stable name through
    //    `axiom_by_name` on load. The metadata `name` and the reconstructed
    //    axiom's `name()` are the SAME value, so the round-trip closes.
    let axiom_registrations: Vec<TokenStream> = def
        .axioms
        .iter()
        .map(|a| {
            let name_ident = &a.name;
            quote! {
                #[cfg(not(target_arch = "wasm32"))]
                #pr4xis::paste::paste! {
                    #[#pr4xis::linkme::distributed_slice(#pr4xis::ontology::AXIOMS)]
                    #[linkme(crate = #pr4xis::linkme)]
                    static [<_REGISTER_AXIOM_ #name_ident:snake:upper>]: fn() -> #pr4xis::ontology::meta::Provenance =
                        || <#name_ident as #pr4xis::logic::axiom::Axiom>::meta(&#name_ident);
                    #[#pr4xis::linkme::distributed_slice(#pr4xis::ontology::AXIOM_CONSTRUCTORS)]
                    #[linkme(crate = #pr4xis::linkme)]
                    static [<_REGISTER_AXIOM_CTOR_ #name_ident:snake:upper>]: fn() -> #pr4xis::ontology::BoxedAxiom =
                        || #pr4xis::ontology::boxed_axiom(#name_ident);
                }
            }
        })
        .collect();

    // The concept enum + its `Concept`/`FinitelyGenerated` impls. We emit these
    // by hand rather than via `#[derive(Concept)]` because the ontology carries
    // label data the bare derive cannot see: the generated `Concept::lexical()`
    // returns each concept's ONTOLEX-Lemon entry from the labels table above.
    // `name()` (variant identifier) and `variants()` (finite enumeration) match
    // the derive exactly, so a macro-built enum stays interchangeable with a
    // `#[derive(Concept)]` one.
    let concept_enum_and_impls = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #entity_name {
            #(#concept_idents,)*
        }

        impl #pr4xis::category::Concept for #entity_name {
            fn name(&self) -> &'static str {
                match self {
                    #(#entity_name::#concept_idents => #concept_names,)*
                }
            }

            fn lexical(&self) -> ::core::option::Option<#pr4xis::ontology::meta::Lexical> {
                match self {
                    #(#lexical_arms)*
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }
        }

        impl #pr4xis::category::FinitelyGenerated for #entity_name {
            fn variants() -> Vec<Self> {
                vec![#(#entity_name::#concept_idents,)*]
            }
        }
    };

    quote! {
        #concept_enum_and_impls

        #category_impl

        // The category knows its declared ontology name — the same `name_lit`
        // its `Vocabulary` registers under. This is what lets a functor
        // between two `ontology!` categories serialize each endpoint's
        // ontology name into a cross-ontology, content-addressed `Connection`.
        impl #pr4xis::category::NamedCategory for #cat_name {
            fn ontology_name() -> #pr4xis::ontology::meta::OntologyName {
                #pr4xis::ontology::meta::OntologyName::new_static(#name_lit)
            }
        }

        // The category can reach its ontology's DOMAIN axioms — the typed
        // bridge from `Cat` to the per-`axioms:`-clause claims declared above,
        // mirroring how `NamedCategory` bridges `Cat` to its declared name.
        // Delegates to the generated `domain_axioms()` accessor, so a
        // projection (`pr4xis-runtime::emit`) serializes them as
        // content-addressed `Axiom` nodes WITHOUT reaching into the
        // `<Name>Ontology` struct. An ontology with no `axioms:` clause yields
        // an empty Vec (honest absence — never a fabricated axiom).
        impl #pr4xis::category::DomainAxiomatized for #cat_name {
            fn domain_axioms() -> Vec<Box<dyn #pr4xis::ontology::Axiom>> {
                #ont_name::generated_domain_axioms()
            }
        }

        #taxonomy_impl
        #mereology_impl
        #causation_impl
        #opposition_impl

        #being_classified

        // Domain axioms declared via the `axioms:` clause. Each one is a
        // full `impl Axiom` with structured `meta()` — no hand-written blocks.
        #(#axiom_structs)*

        pub struct #ont_name;

        impl #ont_name {
            pub fn generated_structural_axioms() -> Vec<Box<dyn #pr4xis::ontology::Axiom>> {
                #pr4xis::ontology::reasoning::structural_axioms_for::<#cat_name>()
            }

            /// Domain axioms declared in this ontology's `axioms:` clause.
            ///
            /// These are claims *specific to this ontology's subject matter*,
            /// as distinct from `generated_structural_axioms()`, which are
            /// automatic taxonomy/mereology/causation/opposition invariants.
            pub fn generated_domain_axioms() -> Vec<Box<dyn #pr4xis::ontology::Axiom>> {
                let mut axioms: Vec<Box<dyn #pr4xis::ontology::Axiom>> = Vec::new();
                #(#axiom_domain_pushes)*
                axioms
            }

            /// Structured metadata — unified Lemon+PROV-O record.
            /// Same shape as functors/adjunctions/nat-trans/axioms (issue #153).
            pub fn meta() -> #pr4xis::ontology::meta::Provenance {
                #pr4xis::ontology::meta::Provenance {
                    name: #pr4xis::ontology::meta::OntologyName::new_static(#name_lit),
                    description: #pr4xis::ontology::meta::Label::new_static(#name_lit),
                    citation: #pr4xis::ontology::meta::Citation::EMPTY,
                    module_path: #pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }

            #[allow(dead_code, unused_assignments)]
            pub fn vocabulary() -> #pr4xis::ontology::Vocabulary {
                #pr4xis::ontology::Vocabulary::from_static::<#cat_name, #entity_name>(
                    #pr4xis::ontology::OntologyName::new_static(#name_lit),
                    #pr4xis::ontology::ModulePath::new_static(module_path!()),
                    #pr4xis::ontology::Citation::parse_static(#source_tokens),
                )
            }

            pub fn labels() -> &'static [(#entity_name, &'static str, &'static str, &'static str)] {
                &[#(#label_entries,)*]
            }
        }

        // Auto-register this ontology into the global VOCABULARIES slice.
        // On wasm32, linkme is unsupported — registration is skipped.
        #[cfg(not(target_arch = "wasm32"))]
        #pr4xis::paste::paste! {
            #[#pr4xis::linkme::distributed_slice(#pr4xis::ontology::VOCABULARIES)]
            #[linkme(crate = #pr4xis::linkme)]
            static [<_REGISTER_ #ont_name:snake:upper>]: fn() -> #pr4xis::ontology::Vocabulary = #ont_name::vocabulary;
        }

        // Auto-register each declared domain axiom into the AXIOMS slice.
        // The registry then has a complete Lemon-layer lexicon including
        // every axiom's citation (issue #148).
        #(#axiom_registrations)*
    }
}
