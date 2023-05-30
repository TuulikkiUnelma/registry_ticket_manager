//! This crate contains the derive macro for the `RegistryManager` trait.

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

#[proc_macro_derive(RegistryTicket)]
pub fn registry_ticket_derive(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    let name = input_ast.ident;

    let generics = input_ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics RegistryTicket for #name #ty_generics #where_clause {
            fn from_index(index: usize) -> Option<Self> {
                index.try_into().ok().map(Self)
            }
            fn to_index(&self) -> usize {
                self.0 as usize
            }
        }
    };

    expanded.into()
}
