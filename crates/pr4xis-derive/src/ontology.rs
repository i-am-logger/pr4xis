use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitStr, Token, braced, bracketed, parenthesized};

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
/// ```ignore
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
    being: Option<Ident>,
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
        let mut being = None;
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
                "being" => {
                    being = Some(input.parse::<Ident>()?);
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
            being,
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

    let has_custom_edges = !def.edges.is_empty();

    // Collect unique edge kinds
    let edge_kinds: Vec<&Ident> = def.edges.iter().map(|e| &e.kind).collect();
    let unique_kinds: Vec<&Ident> = {
        let mut seen = std::collections::HashSet::new();
        edge_kinds
            .iter()
            .filter(|k| seen.insert(k.to_string()))
            .copied()
            .collect()
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

    // Source and being
    let source_tokens = def
        .source
        .as_ref()
        .map(|s| quote! { #s })
        .unwrap_or(quote! { "" });

    let being_tokens = if let Some(ref b) = def.being {
        quote! { Some(#pr4xis::ontology::upper::being::Being::#b) }
    } else {
        quote! { None }
    };

    let being_classified = def.being.as_ref().map(|b| {
        quote! {
            impl #pr4xis::ontology::upper::classify::Classified for #cat_name {
                fn being() -> #pr4xis::ontology::upper::being::Being {
                    #pr4xis::ontology::upper::being::Being::#b
                }
                fn classification_reason() -> &'static str {
                    concat!("DOLCE D18 ", stringify!(#b), "; ", module_path!())
                }
            }
        }
    });

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

    // Category generation — kinded or dense
    let category_impl = if has_custom_edges {
        let kind_name = format_ident!("{}RelationKind", name_str);
        let edge_from: Vec<&Ident> = def.edges.iter().map(|e| &e.from).collect();
        let edge_to: Vec<&Ident> = def.edges.iter().map(|e| &e.to).collect();
        let edge_kind: Vec<&Ident> = def.edges.iter().map(|e| &e.kind).collect();

        // Compute transitive closure of declared edges (Floyd-Warshall at macro expansion).
        // Ensures morphisms() is closed under compose() — if edges (A,B) and (B,C) exist,
        // then (A,C) is included as Composed, matching what compose() returns at runtime.
        let direct_pairs: std::collections::BTreeSet<(String, String)> = def
            .edges
            .iter()
            .map(|e| (e.from.to_string(), e.to.to_string()))
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
        let comp_from: Vec<Ident> = composed_pairs
            .iter()
            .map(|(f, _)| format_ident!("{}", f))
            .collect();
        let comp_to: Vec<Ident> = composed_pairs
            .iter()
            .map(|(_, t)| format_ident!("{}", t))
            .collect();

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #kind_name {
                Identity,
                #(#unique_kinds,)*
                Composed,
            }

            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #relation_name {
                pub from: #entity_name,
                pub to: #entity_name,
                pub kind: #kind_name,
            }

            impl #pr4xis::category::Relationship for #relation_name {
                type Object = #entity_name;
                type Kind = #kind_name;
                fn source(&self) -> #entity_name { self.from }
                fn target(&self) -> #entity_name { self.to }
                fn kind(&self) -> #kind_name { self.kind }
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
                    Some(#relation_name { from: f.from, to: g.to, kind: #kind_name::Composed })
                }

                fn morphisms() -> Vec<#relation_name> {
                    use #pr4xis::category::Entity;
                    let mut m = Vec::new();
                    for c in #entity_name::variants() {
                        m.push(#relation_name { from: c, to: c, kind: #kind_name::Identity });
                    }
                    #(m.push(#relation_name { from: #entity_name::#edge_from, to: #entity_name::#edge_to, kind: #kind_name::#edge_kind });)*
                    #(m.push(#relation_name { from: #entity_name::#comp_from, to: #entity_name::#comp_to, kind: #kind_name::Composed });)*
                    for c in #entity_name::variants() {
                        m.push(#relation_name { from: c, to: c, kind: #kind_name::Composed });
                    }
                    m
                }
            }
        }
    } else {
        // Dense category
        quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #relation_name {
                pub from: #entity_name,
                pub to: #entity_name,
            }

            impl #pr4xis::category::Relationship for #relation_name {
                type Object = #entity_name;
                type Kind = ();
                fn source(&self) -> #entity_name { self.from }
                fn target(&self) -> #entity_name { self.to }
                fn kind(&self) -> () {}
            }

            pub struct #cat_name;

            impl #pr4xis::category::Category for #cat_name {
                type Object = #entity_name;
                type Morphism = #relation_name;

                fn identity(obj: &#entity_name) -> #relation_name {
                    #relation_name { from: *obj, to: *obj }
                }

                fn compose(f: &#relation_name, g: &#relation_name) -> Option<#relation_name> {
                    if f.to != g.from { return None; }
                    Some(#relation_name { from: f.from, to: g.to })
                }

                fn morphisms() -> Vec<#relation_name> {
                    use #pr4xis::category::Entity;
                    let variants = #entity_name::variants();
                    variants.iter()
                        .flat_map(|&a| variants.iter().map(move |&b| #relation_name { from: a, to: b }))
                        .collect()
                }
            }
        }
    };

    // Reasoning systems
    let taxonomy_impl = if has_taxonomy {
        quote! {
            pub struct #tax_name;
            impl #pr4xis::ontology::reasoning::taxonomy::TaxonomyDef for #tax_name {
                type Entity = #entity_name;
                fn relations() -> Vec<(#entity_name, #entity_name)> {
                    vec![#(#tax_pairs),*]
                }
            }
        }
    } else {
        quote! {}
    };

    let mereology_impl = if has_mereology {
        quote! {
            pub struct #mer_name;
            impl #pr4xis::ontology::reasoning::mereology::MereologyDef for #mer_name {
                type Entity = #entity_name;
                fn relations() -> Vec<(#entity_name, #entity_name)> {
                    vec![#(#mer_pairs),*]
                }
            }
        }
    } else {
        quote! {}
    };

    let causation_impl = if has_causation {
        quote! {
            pub struct #caus_name;
            impl #pr4xis::ontology::reasoning::causation::CausalDef for #caus_name {
                type Entity = #entity_name;
                fn relations() -> Vec<(#entity_name, #entity_name)> {
                    vec![#(#caus_pairs),*]
                }
            }
        }
    } else {
        quote! {}
    };

    let opposition_impl = if has_opposition {
        quote! {
            pub struct #opp_name;
            impl #pr4xis::ontology::reasoning::opposition::OppositionDef for #opp_name {
                type Entity = #entity_name;
                fn pairs() -> Vec<(#entity_name, #entity_name)> {
                    vec![#(#opp_pairs),*]
                }
            }
        }
    } else {
        quote! {}
    };

    // Structural axioms
    let mut axiom_pushes = Vec::new();
    if has_taxonomy {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::taxonomy::NoCycles::<#tax_name>::new()
            ));
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::taxonomy::Antisymmetric::<#tax_name>::new()
            ));
        });
    }
    if has_mereology {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::mereology::NoCycles::<#mer_name>::new()
            ));
        });
    }
    if has_causation {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::causation::Asymmetric::<#caus_name>::new()
            ));
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::causation::NoSelfCausation::<#caus_name>::new()
            ));
        });
    }
    if has_opposition {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::opposition::Symmetric::<#opp_name>::new()
            ));
            axioms.push(Box::new(
                #pr4xis::ontology::reasoning::opposition::Irreflexive::<#opp_name>::new()
            ));
        });
    }

    // Vocabulary.name matches the generated struct name (e.g. `name: "Foo"`
    // → Vocabulary.name = "FooOntology"). This keeps parity with the older
    // declarative `define_ontology!` macro, which used the struct ident
    // directly. Without this suffix the new proc macro would emit "Foo"
    // while unmigrated ontologies emit "FooOntology", producing an
    // inconsistent knowledge-base registry.
    let full_name = format!("{name_str}Ontology");
    let name_lit = syn::LitStr::new(&full_name, def.name.span());

    // Domain axioms declared in the `axioms:` clause. Each one emits a
    // unit struct + `impl Axiom` + `RelationshipMeta`, and gets pushed into
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
                    fn description(&self) -> &str {
                        #description
                    }

                    fn holds(&self) -> bool {
                        #holds
                    }

                    fn meta(&self) -> #pr4xis::ontology::meta::RelationshipMeta {
                        #pr4xis::ontology::meta::RelationshipMeta {
                            name: #pr4xis::ontology::meta::OntologyName::new_static(#name_str_lit),
                            description: #pr4xis::ontology::meta::Label::new_static(#description),
                            citation: #pr4xis::ontology::meta::Citation::parse_static(#source),
                            module_path: #pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
                        }
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

    // Per-axiom auto-registration into the global AXIOMS distributed slice
    // so the Lemon lexicon includes every declared axiom.
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
                    static [<_REGISTER_AXIOM_ #name_ident:snake:upper>]: fn() -> #pr4xis::ontology::meta::RelationshipMeta =
                        || <#name_ident as #pr4xis::logic::axiom::Axiom>::meta(&#name_ident);
                }
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, #pr4xis::category::Entity)]
        pub enum #entity_name {
            #(#concept_idents,)*
        }

        #category_impl

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
                let mut axioms: Vec<Box<dyn #pr4xis::ontology::Axiom>> = Vec::new();
                #(#axiom_pushes)*
                axioms
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
            pub fn meta() -> #pr4xis::ontology::meta::RelationshipMeta {
                #pr4xis::ontology::meta::RelationshipMeta {
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
                    #being_tokens,
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
