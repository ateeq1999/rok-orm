//! Procedural macros for rok-orm.
//!
//! # Macros
//!
//! | Macro | Kind | Description |
//! |---|---|---|
//! | `#[derive(Model)]` | derive | Implement the `Model` trait for a struct |
//! | `#[derive(Relations)]` | derive | Implement relationship methods |
//! | `#[derive(ModelHooks)]` | derive | Implement model lifecycle hooks |
//! | `query!` | function-like | Shorthand for building a [`QueryBuilder`] |
//!
//! [`QueryBuilder`]: rok_orm_core::QueryBuilder

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(Model, attributes(model, rok_orm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_model(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut custom_table: Option<String> = None;
    let mut struct_pk: Option<String> = None;
    let mut soft_delete = false;
    let mut timestamps = false;

    for attr in &input.attrs {
        let is_model_attr = attr.path().is_ident("model") || attr.path().is_ident("rok_orm");
        if !is_model_attr {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                custom_table = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("primary_key") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                struct_pk = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("soft_delete") {
                soft_delete = true;
                Ok(())
            } else if meta.path.is_ident("timestamps") {
                timestamps = true;
                Ok(())
            } else {
                Err(meta.error("unknown model struct attribute"))
            }
        })?;
    }

    let table =
        custom_table.unwrap_or_else(|| format!("{}s", struct_name.to_string().to_snake_case()));

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "#[derive(Model)] only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "#[derive(Model)] only supports structs",
            ))
        }
    };

    let mut column_names: Vec<String> = Vec::new();
    let mut field_pk: Option<String> = None;

    for field in fields.iter() {
        let field_ident = match &field.ident {
            Some(id) => id.to_string(),
            None => continue,
        };

        let mut skip = false;
        let mut col_override: Option<String> = None;
        let mut is_pk = false;

        for attr in &field.attrs {
            let is_model_attr = attr.path().is_ident("model") || attr.path().is_ident("rok_orm");
            if !is_model_attr {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                    Ok(())
                } else if meta.path.is_ident("primary_key") {
                    is_pk = true;
                    Ok(())
                } else if meta.path.is_ident("column") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    col_override = Some(s.value());
                    Ok(())
                } else {
                    Err(meta.error("unknown model field attribute"))
                }
            })?;
        }

        if is_pk {
            field_pk = Some(col_override.clone().unwrap_or(field_ident.clone()));
        }

        if !skip {
            column_names.push(col_override.unwrap_or(field_ident));
        }
    }

    let pk = field_pk.or(struct_pk).unwrap_or_else(|| "id".to_string());

    let columns_len = column_names.len();

    let soft_delete_impl = if soft_delete {
        quote! {
            fn soft_delete_column() -> Option<&'static str> {
                Some("deleted_at")
            }
        }
    } else {
        quote! {
            fn soft_delete_column() -> Option<&'static str> {
                None
            }
        }
    };

    let timestamps_impl = if timestamps {
        quote! {
            fn timestamps_enabled() -> bool {
                true
            }
        }
    } else {
        quote! {
            fn timestamps_enabled() -> bool {
                false
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::rok_orm::Model for #struct_name #ty_generics #where_clause {
            fn table_name() -> &'static str {
                #table
            }

            fn primary_key() -> &'static str {
                #pk
            }

            fn columns() -> &'static [&'static str] {
                static COLS: [&str; #columns_len] = [#(#column_names),*];
                &COLS
            }

            #soft_delete_impl
            #timestamps_impl
        }
    };

    Ok(expanded.into())
}

#[proc_macro_derive(Relations, attributes(model))]
pub fn derive_relations(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_relations(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand_relations(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "#[derive(Relations)] only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "#[derive(Relations)] only supports structs",
            ))
        }
    };

    let mut relations_impls = Vec::new();

    for field in fields.iter() {
        let field_ident = field.ident.as_ref().expect("named field");
        let _field_type = &field.ty;

        for attr in &field.attrs {
            if !attr.path().is_ident("model") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("has_many") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let _target_name = get_type_name(&target);
                    let foreign_key = format!("{}_id", struct_name.to_string().to_snake_case());

                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::HasMany<Self, #target> {
                            ::rok_orm::relations::HasMany::new(
                                Self::table_name(),
                                Self::primary_key(),
                                #target::table_name(),
                                #target::primary_key(),
                                #foreign_key,
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("has_one") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let foreign_key = format!("{}_id", struct_name.to_string().to_snake_case());

                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::HasOne<Self, #target> {
                            ::rok_orm::relations::HasOne::new(
                                Self::table_name(),
                                Self::primary_key(),
                                #target::table_name(),
                                #foreign_key,
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("belongs_to") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let target_name = get_type_name(&target);
                    let foreign_key = format!("{}_id", target_name.to_snake_case());

                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::BelongsTo<Self, #target> {
                            ::rok_orm::relations::BelongsTo::new(
                                Self::table_name(),
                                #foreign_key,
                                #target::table_name(),
                                #target::primary_key(),
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("belongs_to_many") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let target_name = get_type_name(&target);
                    let pivot = format!("{}_{}", struct_name.to_string().to_snake_case(), target_name.to_snake_case());
                    let left_key = format!("{}_id", struct_name.to_string().to_snake_case());
                    let right_key = format!("{}_id", target_name.to_snake_case());

                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::belongs_to_many::BelongsToMany<Self, #target> {
                            ::rok_orm::belongs_to_many::BelongsToMany::new(
                                Self::table_name(),
                                Self::primary_key(),
                                #pivot.to_string(),
                                #left_key.to_string(),
                                #right_key.to_string(),
                                #target::table_name(),
                                #target::primary_key(),
                            )
                        }
                    });
                    Ok(())
                } else {
                    Err(meta.error("unknown relation type"))
                }
            })?;
        }
    }

    let expanded = quote! {
        impl #impl_generics ::rok_orm::Relations for #struct_name #ty_generics #where_clause {
            #(#relations_impls)*
        }
    };

    Ok(expanded.into())
}

#[proc_macro_derive(ModelHooks)]
pub fn derive_model_hooks(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_model_hooks(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand_model_hooks(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::rok_orm::hooks::ModelHooks for #struct_name #ty_generics #where_clause {}
    };

    Ok(expanded.into())
}

fn get_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Token, Type,
};

struct QueryClause {
    name: Ident,
    args: Vec<Expr>,
}

impl Parse for QueryClause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let mut args = Vec::new();
        while !input.is_empty() && !input.peek(Token![,]) {
            args.push(input.parse::<Expr>()?);
            if input.peek(Token![,]) {
                break;
            }
        }
        Ok(QueryClause { name, args })
    }
}

struct QueryMacroInput {
    model: Type,
    clauses: Vec<QueryClause>,
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let model: Type = input.parse()?;
        let mut clauses = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            clauses.push(input.parse::<QueryClause>()?);
        }
        Ok(QueryMacroInput { model, clauses })
    }
}

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let QueryMacroInput { model, clauses } = parse_macro_input!(input as QueryMacroInput);

    let mut chain = quote! { <#model as ::rok_orm::Model>::query() };

    for clause in clauses {
        let name_str = clause.name.to_string();
        let args = &clause.args;

        let call = match (name_str.as_str(), args.len()) {
            ("where", 2) | ("filter", 2) | ("eq", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .filter(#c, #v) }
            }
            ("where_eq", 2) | ("eq", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .eq(#c, #v) }
            }
            ("where_ne", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_ne(#c, #v) }
            }
            ("where_gt", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_gt(#c, #v) }
            }
            ("where_gte", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_gte(#c, #v) }
            }
            ("where_lt", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_lt(#c, #v) }
            }
            ("where_lte", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_lte(#c, #v) }
            }
            ("where_like", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_like(#c, #v) }
            }
            ("where_not_like", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .where_not_like(#c, #v) }
            }
            ("where_null", 1) => {
                let c = &args[0];
                quote! { .where_null(#c) }
            }
            ("where_not_null", 1) => {
                let c = &args[0];
                quote! { .where_not_null(#c) }
            }
            ("or_where_eq", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .or_where_eq(#c, #v) }
            }
            ("or_where_ne", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .or_where_ne(#c, #v) }
            }
            ("order_by", 1) => {
                let c = &args[0];
                quote! { .order_by(#c) }
            }
            ("order_by_desc", 1) => {
                let c = &args[0];
                quote! { .order_by_desc(#c) }
            }
            ("limit", 1) => {
                let n = &args[0];
                quote! { .limit(#n) }
            }
            ("offset", 1) => {
                let n = &args[0];
                quote! { .offset(#n) }
            }
            ("select", _) => quote! { .select(&[#(#args),*]) },
            ("distinct", 0) => quote! { .distinct() },
            (name, _) => {
                return syn::Error::new(
                    clause.name.span(),
                    format!("unknown query! clause `{name}`"),
                )
                .to_compile_error()
                .into();
            }
        };

        chain = quote! { #chain #call };
    }

    chain.into()
}
