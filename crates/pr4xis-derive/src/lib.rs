mod ontology;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{Data, DeriveInput, Fields, Ident};

/// Resolve the `pr4xis` crate path for use in generated code.
///
/// Within the `pr4xis` crate itself, returns `crate`. From downstream crates,
/// returns the name under which `pr4xis` is imported (usually `pr4xis`,
/// possibly renamed via Cargo `package = ...`).
pub(crate) fn pr4xis_crate() -> TokenStream2 {
    match crate_name("pr4xis") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::pr4xis },
    }
}

/// Derive the `Concept` (open-world) and `FinitelyGenerated` (closed-world)
/// traits for an enum with unit variants.
///
/// Generates two impls — a derived unit enum is always closed-world, so it gets
/// both:
/// - `impl Concept` with `fn name(&self) -> &'static str` — variant name as
///   string (Lemon canonical form)
/// - `impl FinitelyGenerated` with `fn variants() -> Vec<Self>` — all enum
///   variants (the finite closed-world enumeration)
#[proc_macro_derive(Concept)]
pub fn derive_concept(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(&input, "Concept can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let mut variant_idents = Vec::new();
    for v in &data_enum.variants {
        match &v.fields {
            Fields::Unit => variant_idents.push(&v.ident),
            _ => {
                return syn::Error::new_spanned(
                    v,
                    "Concept derive only supports unit variants (no fields)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let variant_names: Vec<String> = variant_idents.iter().map(|v| v.to_string()).collect();

    let pr4xis = pr4xis_crate();
    let expanded = quote! {
        impl #impl_generics #pr4xis::category::Concept for #name #ty_generics #where_clause {
            fn name(&self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #variant_names),*
                }
            }
        }

        impl #impl_generics #pr4xis::category::FinitelyGenerated for #name #ty_generics #where_clause {
            fn variants() -> Vec<Self> {
                vec![#(Self::#variant_idents),*]
            }
        }
    };

    expanded.into()
}

/// Define an ontology with compile-time validation and static code generation.
///
/// Generates: `Concept` enum, `Category` impl, `Arrow` impl (with kind
/// tagging), structural axioms inherited from the catalog, `Ontology`
/// impl, and a type-level `fn meta() -> Provenance` for trace
/// attribution.
///
/// Concept names referenced in `edges:` / `is_a:` / `has_a:` /
/// `causes:` / `opposes:` are validated at compile time.
///
/// # Example
///
/// ```text
/// pr4xis::ontology! {
///     name: "Biology",
///     source: "Mayr (1982) The Growth of Biological Thought",
///     concepts: [Cell, Tissue, Organism],
///     labels: {
///         Cell: ("en", "Cell", "The basic structural unit"),
///     },
///     is_a: [(Cell, Tissue), (Tissue, Organism)],
/// }
/// ```
#[proc_macro]
pub fn ontology(input: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(input as ontology::OntologyDef);
    ontology::generate(def).into()
}

/// Declare which constitutional guarantee(s) a test witnesses.
///
/// The attribute leaves the test function untouched and registers a
/// [`GuaranteeTag`](pr4xis::constitution::GuaranteeTag) into the
/// [`CONSTITUTION_TESTS`](pr4xis::constitution::CONSTITUTION_TESTS) distributed
/// slice, so the `constitution_coverage` meta-test can partition the suite by
/// guarantee. The first ident is the *primary* (partition) guarantee; any
/// further idents are *secondary* guarantees the same irreducible test also
/// witnesses.
///
/// Place it **above** `#[test]` so it sees and re-emits the full test:
///
/// ```ignore
/// #[pr4xis::praxis_value(Honest)]
/// #[test]
/// fn an_unknown_word_is_left_ungrounded() { /* ... */ }
///
/// #[pr4xis::praxis_value(Honest, Verifiable, Deterministic)]
/// #[test]
/// fn prop_mutated_prx_always_rejected() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn praxis_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    let parser = syn::punctuated::Punctuated::<Ident, syn::Token![,]>::parse_terminated;
    let guarantees: Vec<Ident> = match parser.parse(attr) {
        Ok(p) => p.into_iter().collect(),
        Err(e) => return e.to_compile_error().into(),
    };

    if guarantees.is_empty() {
        return syn::Error::new(
            Span::call_site(),
            "praxis_value requires at least one guarantee, e.g. #[praxis_value(Honest)]",
        )
        .to_compile_error()
        .into();
    }

    let primary = &guarantees[0];
    let secondary = &guarantees[1..];
    let fn_name = &item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    // Module-scoped: test fn names are unique within their module, so the
    // registration static next to each one is too.
    let static_name = format_ident!("__PRAXIS_VALUE_{}", fn_name);
    let pr4xis = pr4xis_crate();

    let expanded = quote! {
        #item_fn

        #[cfg(not(target_arch = "wasm32"))]
        #[allow(non_upper_case_globals)]
        #[#pr4xis::linkme::distributed_slice(#pr4xis::constitution::CONSTITUTION_TESTS)]
        #[linkme(crate = #pr4xis::linkme)]
        static #static_name: #pr4xis::constitution::GuaranteeTag =
            #pr4xis::constitution::GuaranteeTag {
                primary: #pr4xis::constitution::Guarantee::#primary,
                secondary: &[ #( #pr4xis::constitution::Guarantee::#secondary ),* ],
                // The attribute wraps a plain `#[test]` — a point-claim.
                kind: #pr4xis::constitution::TestKind::Example,
                module: module_path!(),
                name: #fn_name_str,
            };
    };

    expanded.into()
}
