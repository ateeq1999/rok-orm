//! Expansion logic for `#[derive(Model)]`.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, punctuated::Punctuated, Token};

pub fn derive_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut custom_table: Option<String> = None;
    let mut struct_pk: Option<String> = None;
    let mut soft_delete = false;
    let mut timestamps = false;
    let mut created_at_col: Option<String> = None;
    let mut updated_at_col: Option<String> = None;
    let mut soft_delete_col: Option<String> = None;
    let mut fillable_cols: Vec<String> = Vec::new();
    let mut guarded_cols: Vec<String> = Vec::new();
    let mut uuid_pk = false;

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
            } else if meta.path.is_ident("soft_delete_col") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                soft_delete_col = Some(s.value());
                soft_delete = true;
                Ok(())
            } else if meta.path.is_ident("timestamps") {
                timestamps = true;
                Ok(())
            } else if meta.path.is_ident("created_at_col") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                created_at_col = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("updated_at_col") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                updated_at_col = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("uuid") {
                uuid_pk = true;
                Ok(())
            } else if meta.path.is_ident("fillable") {
                let value = meta.value()?;
                let content;
                syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                fillable_cols.extend(list.into_iter().map(|s| s.value()));
                Ok(())
            } else if meta.path.is_ident("guarded") {
                let value = meta.value()?;
                let content;
                syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                guarded_cols.extend(list.into_iter().map(|s| s.value()));
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

    let sd_col = soft_delete_col.as_deref().unwrap_or("deleted_at");
    let soft_delete_impl = if soft_delete {
        quote! {
            fn soft_delete_column() -> Option<&'static str> { Some(#sd_col) }
        }
    } else {
        quote! {
            fn soft_delete_column() -> Option<&'static str> { None }
        }
    };

    let ca_col = created_at_col.as_deref().unwrap_or("created_at");
    let ua_col = updated_at_col.as_deref().unwrap_or("updated_at");
    let timestamps_impl = if timestamps {
        quote! {
            fn timestamps_enabled() -> bool { true }
            fn created_at_column() -> Option<&'static str> { Some(#ca_col) }
            fn updated_at_column() -> Option<&'static str> { Some(#ua_col) }
        }
    } else {
        quote! {
            fn timestamps_enabled() -> bool { false }
            fn created_at_column() -> Option<&'static str> { None }
            fn updated_at_column() -> Option<&'static str> { None }
        }
    };

    let uuid_impl = if uuid_pk {
        quote! {
            fn new_unique_id() -> Option<::rok_orm::SqlValue> {
                #[cfg(feature = "uuid-pk")]
                {
                    Some(::rok_orm::SqlValue::Text(::uuid::Uuid::new_v4().to_string()))
                }
                #[cfg(not(feature = "uuid-pk"))]
                {
                    panic!("uuid-pk feature must be enabled to use UUID primary keys")
                }
            }
        }
    } else {
        quote! {}
    };

    let fillable_len = fillable_cols.len();
    let guarded_len = guarded_cols.len();

    let fillable_impl = if fillable_len > 0 {
        quote! {
            fn fillable() -> &'static [&'static str] {
                static COLS: [&str; #fillable_len] = [#(#fillable_cols),*];
                &COLS
            }
        }
    } else {
        quote! {}
    };

    let guarded_impl = if guarded_len > 0 {
        quote! {
            fn guarded() -> &'static [&'static str] {
                static COLS: [&str; #guarded_len] = [#(#guarded_cols),*];
                &COLS
            }
        }
    } else {
        quote! {}
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
            #uuid_impl
            #fillable_impl
            #guarded_impl
        }
    };

    Ok(expanded.into())
}
