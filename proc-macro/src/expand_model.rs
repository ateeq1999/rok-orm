//! Expansion logic for `#[derive(Model)]`.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, punctuated::Punctuated, Token};

use crate::expand_model_cast::{CastFieldInfo, gen_to_fields, gen_post_process};
use crate::expand_model_serial::{
    gen_hidden_visible_appends, gen_serialize_methods, gen_accessors,
};

pub fn derive_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // ── Struct-level attribute parsing ────────────────────────────────────────
    let mut custom_table: Option<String> = None;
    let mut struct_pk: Option<String> = None;
    let mut soft_delete = false;
    let mut timestamps = false;
    let mut created_at_col: Option<String> = None;
    let mut updated_at_col: Option<String> = None;
    let mut soft_delete_col: Option<String> = None;
    let mut fillable_cols: Vec<String> = Vec::new();
    let mut guarded_cols: Vec<String> = Vec::new();
    let mut touches_rels: Vec<String> = Vec::new();
    let mut uuid_pk = false;
    let mut ulid_pk = false;
    let mut custom_id_fn: Option<String> = None;
    let mut connection_name: Option<String> = None;
    // Phase 11 struct attrs
    let mut hidden_cols: Vec<String> = Vec::new();
    let mut visible_cols: Vec<String> = Vec::new();
    let mut appends_fields: Vec<String> = Vec::new();

    for attr in &input.attrs {
        let is_model_attr = attr.path().is_ident("model") || attr.path().is_ident("rok_orm");
        if !is_model_attr { continue; }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                custom_table = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("primary_key") {
                struct_pk = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("soft_delete") {
                soft_delete = true; Ok(())
            } else if meta.path.is_ident("soft_delete_col") {
                soft_delete_col = Some(meta.value()?.parse::<LitStr>()?.value());
                soft_delete = true; Ok(())
            } else if meta.path.is_ident("timestamps") {
                timestamps = true; Ok(())
            } else if meta.path.is_ident("created_at_col") {
                created_at_col = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("updated_at_col") {
                updated_at_col = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("uuid") {
                uuid_pk = true; Ok(())
            } else if meta.path.is_ident("ulid") {
                ulid_pk = true; Ok(())
            } else if meta.path.is_ident("custom_id") {
                custom_id_fn = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("connection") {
                connection_name = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
            } else if meta.path.is_ident("touches") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                touches_rels.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else if meta.path.is_ident("fillable") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                fillable_cols.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else if meta.path.is_ident("guarded") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                guarded_cols.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else if meta.path.is_ident("hidden") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                hidden_cols.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else if meta.path.is_ident("visible") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                visible_cols.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else if meta.path.is_ident("appends") {
                let value = meta.value()?;
                let content; syn::bracketed!(content in value);
                let list = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                appends_fields.extend(list.into_iter().map(|s| s.value())); Ok(())
            } else {
                Err(meta.error("unknown model struct attribute"))
            }
        })?;
    }

    let table =
        custom_table.unwrap_or_else(|| format!("{}s", struct_name.to_string().to_snake_case()));

    // ── Field parsing ─────────────────────────────────────────────────────────
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => return Err(syn::Error::new(Span::call_site(),
                "#[derive(Model)] only supports structs with named fields")),
        },
        _ => return Err(syn::Error::new(Span::call_site(),
            "#[derive(Model)] only supports structs")),
    };

    let mut column_names: Vec<String> = Vec::new();
    let mut field_pk: Option<String> = None;
    let mut pk_field_ident: Option<proc_macro2::Ident> = None;
    let mut regular_pairs: Vec<(proc_macro2::Ident, String)> = Vec::new();
    let mut cast_fields: Vec<CastFieldInfo> = Vec::new();
    let mut accessor_fields: Vec<(proc_macro2::Ident, String)> = Vec::new();

    for field in fields.iter() {
        let raw_ident = match &field.ident { Some(id) => id, None => continue };
        let field_name = raw_ident.to_string();
        let mut skip = false;
        let mut col_override: Option<String> = None;
        let mut is_pk = false;
        let mut cast_kind: Option<crate::expand_model_cast::CastKind> = None;
        let mut is_accessor = false;

        for attr in &field.attrs {
            let is_model_attr = attr.path().is_ident("model") || attr.path().is_ident("rok_orm");
            if !is_model_attr { continue; }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") { skip = true; Ok(()) }
                else if meta.path.is_ident("primary_key") { is_pk = true; Ok(()) }
                else if meta.path.is_ident("accessor") { is_accessor = true; Ok(()) }
                else if meta.path.is_ident("column") {
                    col_override = Some(meta.value()?.parse::<LitStr>()?.value()); Ok(())
                } else if meta.path.is_ident("cast") {
                    let s = meta.value()?.parse::<LitStr>()?.value();
                    cast_kind = crate::expand_model_cast::CastKind::from_str(&s);
                    Ok(())
                } else {
                    let is_relation = ["has_many", "has_one", "belongs_to",
                        "has_many_through", "has_one_through", "belongs_to_many",
                        "many_to_many", "morph_one", "morph_many", "morph_to",
                        "morph_to_many", "morphed_by_many"]
                        .iter().any(|&r| meta.path.is_ident(r));
                    if is_relation { skip = true; }
                    if !meta.input.is_empty() { let _: proc_macro2::TokenStream = meta.input.parse()?; }
                    Ok(())
                }
            })?;
        }

        let col_name = col_override.unwrap_or(field_name);
        if is_pk {
            field_pk = Some(col_name.clone());
            pk_field_ident = Some(raw_ident.clone());
        }
        if !skip {
            column_names.push(col_name.clone());
            match cast_kind {
                Some(ck) => cast_fields.push(CastFieldInfo {
                    ident: raw_ident.clone(), col: col_name.clone(), cast: ck,
                }),
                None => regular_pairs.push((raw_ident.clone(), col_name.clone())),
            }
            if is_accessor {
                accessor_fields.push((raw_ident.clone(), col_name));
            }
        }
    }

    let pk = field_pk.or_else(|| struct_pk.clone()).unwrap_or_else(|| "id".to_string());

    // Remove PK from regular_pairs and cast_fields
    regular_pairs.retain(|(_, c)| *c != pk);
    cast_fields.retain(|cf| cf.col != pk);

    // ── Codegen ───────────────────────────────────────────────────────────────
    let columns_len = column_names.len();

    let sd_col = soft_delete_col.as_deref().unwrap_or("deleted_at");
    let soft_delete_impl = if soft_delete {
        quote! { fn soft_delete_column() -> Option<&'static str> { Some(#sd_col) } }
    } else {
        quote! { fn soft_delete_column() -> Option<&'static str> { None } }
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
                return Some(::rok_orm::SqlValue::Text(::uuid::Uuid::new_v4().to_string()));
                #[allow(unreachable_code)]
                panic!("uuid-pk feature must be enabled to use UUID primary keys")
            }
        }
    } else if ulid_pk {
        quote! {
            fn new_unique_id() -> Option<::rok_orm::SqlValue> {
                #[cfg(feature = "ulid-pk")]
                return Some(::rok_orm::SqlValue::Text(::ulid::Ulid::new().to_string()));
                #[allow(unreachable_code)]
                panic!("ulid-pk feature must be enabled to use ULID primary keys")
            }
        }
    } else if let Some(ref fn_name) = custom_id_fn {
        let fn_path: syn::Path = syn::parse_str(fn_name)?;
        quote! {
            fn new_unique_id() -> Option<::rok_orm::SqlValue> {
                Some(::rok_orm::SqlValue::Text(#fn_path().into()))
            }
        }
    } else { quote! {} };

    let connection_impl = connection_name.as_ref().map_or(quote!{}, |conn| {
        quote! { fn connection() -> &'static str { #conn } }
    });

    let touches_len = touches_rels.len();
    let touches_impl = if touches_len > 0 {
        quote! {
            fn touches() -> &'static [&'static str] {
                static RELS: [&str; #touches_len] = [#(#touches_rels),*];
                &RELS
            }
        }
    } else { quote! {} };

    let fillable_len = fillable_cols.len();
    let guarded_len  = guarded_cols.len();
    let fillable_impl = if fillable_len > 0 {
        quote! {
            fn fillable() -> &'static [&'static str] {
                static COLS: [&str; #fillable_len] = [#(#fillable_cols),*];
                &COLS
            }
        }
    } else { quote! {} };
    let guarded_impl = if guarded_len > 0 {
        quote! {
            fn guarded() -> &'static [&'static str] {
                static COLS: [&str; #guarded_len] = [#(#guarded_cols),*];
                &COLS
            }
        }
    } else { quote! {} };

    // Phase 11 codegen
    let to_fields_impl  = gen_to_fields(&regular_pairs, &cast_fields);
    let post_process_impl = gen_post_process(&cast_fields);
    let serial_trait_impls = gen_hidden_visible_appends(&hidden_cols, &visible_cols, &appends_fields);
    let serialize_impl  = gen_serialize_methods(
        struct_name, &impl_generics, &ty_generics, where_clause, &appends_fields,
    );
    let accessor_impl   = gen_accessors(
        struct_name, &impl_generics, &ty_generics, where_clause, &accessor_fields,
    );

    let replicate_inherent = pk_field_ident.as_ref().map_or(quote!{}, |pk_ident| {
        quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn pk_reset(&mut self) {
                    self.#pk_ident = Default::default();
                }
            }
        }
    });

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
            #connection_impl
            #touches_impl
            #fillable_impl
            #guarded_impl
            #to_fields_impl
            #post_process_impl
            #serial_trait_impls
        }
        #replicate_inherent
        #serialize_impl
        #accessor_impl
    };

    Ok(expanded.into())
}
