mod ontology;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Derive the `Entity` trait for an enum with unit variants.
///
/// Generates:
/// - `fn variants() -> Vec<Self>` — all enum variants
/// - `fn name(&self) -> &'static str` — variant name as string
#[proc_macro_derive(Entity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(&input, "Entity can only be derived for enums")
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
                    "Entity derive only supports unit variants (no fields)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let variant_names: Vec<String> = variant_idents.iter().map(|v| v.to_string()).collect();

    let expanded = quote! {
        impl #impl_generics Entity for #name #ty_generics #where_clause {
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
