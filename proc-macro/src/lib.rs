//! Procedural macros for rok-orm.
//!
//! | Macro | Kind | Description |
//! |---|---|---|
//! | `#[derive(Model)]` | derive | Implement the `Model` trait for a struct |
//! | `#[derive(Relations)]` | derive | Implement relationship methods |
//! | `#[derive(ModelHooks)]` | derive | Implement model lifecycle hooks |
//! | `query!` | function-like | Shorthand for building a [`QueryBuilder`] |

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod expand_model;
mod expand_relations;
mod query_macro;

// ── derive(Model) ────────────────────────────────────────────────────────────

#[proc_macro_derive(Model, attributes(model, rok_orm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_model::derive_model(input).unwrap_or_else(|e| e.to_compile_error().into())
}

// ── derive(Relations) ────────────────────────────────────────────────────────

#[proc_macro_derive(Relations, attributes(model))]
pub fn derive_relations(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_relations::derive_relations(input).unwrap_or_else(|e| e.to_compile_error().into())
}

// ── derive(ModelHooks) ───────────────────────────────────────────────────────

#[proc_macro_derive(ModelHooks)]
pub fn derive_model_hooks(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::rok_orm::hooks::ModelHooks
            for #struct_name #ty_generics #where_clause {}
    };
    expanded.into()
}

// ── query! ───────────────────────────────────────────────────────────────────

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    use syn::parse_macro_input;
    let query_macro::QueryMacroInput { model, clauses } =
        parse_macro_input!(input as query_macro::QueryMacroInput);
    query_macro::expand_query(model, clauses)
}
