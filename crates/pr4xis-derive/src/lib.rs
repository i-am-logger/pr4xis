mod ontology;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
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

/// Derive the `Concept` trait for an enum with unit variants.
///
/// Generates:
/// - `fn variants() -> Vec<Self>` — all enum variants (closed-world enumeration)
/// - `fn name(&self) -> &'static str` — variant name as string (Lemon canonical form)
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
            fn variants() -> Vec<Self> {
                vec![#(Self::#variant_idents),*]
            }

            fn name(&self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #variant_names),*
                }
            }
        }
    };

    expanded.into()
}

/// Define an ontology with compile-time validation and static code generation.
///
/// Generates: Entity enum, Category impl, Relationship impl, reasoning systems,
/// structural axioms, Vocabulary, and Lemon lexical data — all static.
///
/// Concept names in edges/is_a/has_a/causes/opposes are validated at compile time.
///
/// # Example
///
/// ```ignore
/// pr4xis::ontology! {
///     name: "Biology",
///     source: "Mayr (1982)",
///     being: AbstractObject,
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
