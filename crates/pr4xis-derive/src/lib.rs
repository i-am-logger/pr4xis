use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Derive the `Entity` trait for an enum with unit variants.
///
/// Generates `fn variants() -> Vec<Self>` that returns all enum variants.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
/// pub enum Color { Red, Green, Blue }
///
/// // Generates:
/// // impl Entity for Color {
/// //     fn variants() -> Vec<Self> {
/// //         vec![Self::Red, Self::Green, Self::Blue]
/// //     }
/// // }
/// ```
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

    let expanded = quote! {
        impl #impl_generics Entity for #name #ty_generics #where_clause {
            fn variants() -> Vec<Self> {
                vec![#(Self::#variant_idents),*]
            }
        }
    };

    expanded.into()
}
