use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, LitStr, Token, braced, bracketed, parenthesized};

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
        })
    }
}

pub fn generate(def: OntologyDef) -> TokenStream {
    let name_str = def.name.value();
    let ont_name = format_ident!("{}Ontology", name_str);
    let cat_name = format_ident!("{}Category", name_str);
    let entity_name = format_ident!("{}Concept", name_str);
    let relation_name = format_ident!("{}Relation", name_str);

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
        quote! { Some(pr4xis::ontology::upper::being::Being::#b) }
    } else {
        quote! { None }
    };

    let being_classified = def.being.as_ref().map(|b| {
        quote! {
            impl pr4xis::ontology::upper::classify::Classified for #cat_name {
                fn being() -> pr4xis::ontology::upper::being::Being {
                    pr4xis::ontology::upper::being::Being::#b
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
        let comp_from: Vec<&Ident> = def.composed.iter().map(|c| &c.from).collect();
        let comp_to: Vec<&Ident> = def.composed.iter().map(|c| &c.to).collect();

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

            impl pr4xis::category::Relationship for #relation_name {
                type Object = #entity_name;
                fn source(&self) -> #entity_name { self.from }
                fn target(&self) -> #entity_name { self.to }
            }

            pub struct #cat_name;

            impl pr4xis::category::Category for #cat_name {
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
                    use pr4xis::category::Entity;
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

            impl pr4xis::category::Relationship for #relation_name {
                type Object = #entity_name;
                fn source(&self) -> #entity_name { self.from }
                fn target(&self) -> #entity_name { self.to }
            }

            pub struct #cat_name;

            impl pr4xis::category::Category for #cat_name {
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
                    use pr4xis::category::Entity;
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
            impl pr4xis::ontology::reasoning::taxonomy::TaxonomyDef for #tax_name {
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
            impl pr4xis::ontology::reasoning::mereology::MereologyDef for #mer_name {
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
            impl pr4xis::ontology::reasoning::causation::CausalDef for #caus_name {
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
            impl pr4xis::ontology::reasoning::opposition::OppositionDef for #opp_name {
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
                pr4xis::ontology::reasoning::taxonomy::NoCycles::<#tax_name>::new()
            ));
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::taxonomy::Antisymmetric::<#tax_name>::new()
            ));
        });
    }
    if has_mereology {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::mereology::NoCycles::<#mer_name>::new()
            ));
        });
    }
    if has_causation {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::causation::Asymmetric::<#caus_name>::new()
            ));
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::causation::NoSelfCausation::<#caus_name>::new()
            ));
        });
    }
    if has_opposition {
        axiom_pushes.push(quote! {
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::opposition::Symmetric::<#opp_name>::new()
            ));
            axioms.push(Box::new(
                pr4xis::ontology::reasoning::opposition::Irreflexive::<#opp_name>::new()
            ));
        });
    }

    let name_lit = &def.name;

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, pr4xis::category::Entity)]
        pub enum #entity_name {
            #(#concept_idents,)*
        }

        #category_impl

        #taxonomy_impl
        #mereology_impl
        #causation_impl
        #opposition_impl

        #being_classified

        pub struct #ont_name;

        impl #ont_name {
            pub fn generated_structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
                let mut axioms: Vec<Box<dyn pr4xis::ontology::Axiom>> = Vec::new();
                #(#axiom_pushes)*
                axioms
            }

            pub const fn meta() -> pr4xis::ontology::OntologyMeta {
                pr4xis::ontology::OntologyMeta {
                    name: #name_lit,
                    module_path: module_path!(),
                }
            }

            #[allow(dead_code, unused_assignments)]
            pub fn vocabulary() -> pr4xis::ontology::Vocabulary {
                pr4xis::ontology::Vocabulary::from_static::<#cat_name, #entity_name>(
                    pr4xis::ontology::OntologyName::new_static(#name_lit),
                    pr4xis::ontology::ModulePath::new_static(module_path!()),
                    pr4xis::ontology::Citation::parse_static(#source_tokens),
                    #being_tokens,
                )
            }

            pub fn labels() -> &'static [(#entity_name, &'static str, &'static str, &'static str)] {
                &[#(#label_entries,)*]
            }
        }
    }
}
