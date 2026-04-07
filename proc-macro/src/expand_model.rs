//! Expansion logic for `#[derive(Model)]`.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr};

pub fn derive_model(input: DeriveInput) -> syn::Result<TokenStream> {
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
            _ => return Err(syn::Error::new(
                Span::call_site(),
                "#[derive(Model)] only supports structs with named fields",
            )),
        },
        _ => return Err(syn::Error::new(
            Span::call_site(),
            "#[derive(Model)] only supports structs",
        )),
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
            fn soft_delete_column() -> Option<&'static str> { Some("deleted_at") }
        }
    } else {
        quote! {
            fn soft_delete_column() -> Option<&'static str> { None }
        }
    };

    let timestamps_impl = if timestamps {
        quote! {
            fn timestamps_enabled() -> bool { true }
            fn created_at_column() -> Option<&'static str> { Some("created_at") }
            fn updated_at_column() -> Option<&'static str> { Some("updated_at") }
        }
    } else {
        quote! {
            fn timestamps_enabled() -> bool { false }
            fn created_at_column() -> Option<&'static str> { None }
            fn updated_at_column() -> Option<&'static str> { None }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::rok_orm::Model for #struct_name #ty_generics #where_clause {
            fn table_name() -> &'static str { #table }
            fn primary_key() -> &'static str { #pk }
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
