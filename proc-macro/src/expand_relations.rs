//! Expansion logic for `#[derive(Relations)]`.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Token};

use crate::expand_relations_morph::handle_morph_attr;

pub fn get_type_name(ty: &syn::Type) -> String {
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

pub fn derive_relations(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => return Err(syn::Error::new(
                Span::call_site(),
                "#[derive(Relations)] only supports structs with named fields",
            )),
        },
        _ => return Err(syn::Error::new(
            Span::call_site(),
            "#[derive(Relations)] only supports structs",
        )),
    };

    let mut relations_impls = Vec::new();
    // (field_name_str, child_table_expr, fk_expr) for RelationMeta
    let mut meta_entries: Vec<(String, String, String)> = Vec::new();

    for field in fields.iter() {
        let field_ident = field.ident.as_ref().expect("named field");

        for attr in &field.attrs {
            if !attr.path().is_ident("model") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("has_many") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let foreign_key = format!("{}_id", struct_name.to_string().to_snake_case());
                    let target_table = format!("{}s", get_type_name(&target).to_snake_case());
                    meta_entries.push((field_ident.to_string(), target_table, foreign_key.clone()));
                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::HasMany<Self, #target> {
                            ::rok_orm::relations::HasMany::new(
                                Self::table_name(), Self::primary_key(),
                                #target::table_name(), #target::primary_key(),
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
                                Self::table_name(), Self::primary_key(),
                                #target::table_name(), #foreign_key,
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
                                Self::table_name(), #foreign_key,
                                #target::table_name(), #target::primary_key(),
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("belongs_to_many") {
                    let value = meta.value()?;
                    let target: syn::Type = value.parse()?;
                    let target_name = get_type_name(&target);
                    let pivot = format!(
                        "{}_{}",
                        struct_name.to_string().to_snake_case(),
                        target_name.to_snake_case()
                    );
                    let left_key = format!("{}_id", struct_name.to_string().to_snake_case());
                    let right_key = format!("{}_id", target_name.to_snake_case());
                    relations_impls.push(quote! {
                        fn #field_ident(
                            &self,
                        ) -> ::rok_orm::belongs_to_many::BelongsToMany<Self, #target> {
                            ::rok_orm::belongs_to_many::BelongsToMany::new(
                                Self::table_name(), Self::primary_key(),
                                #pivot.to_string(), #left_key.to_string(),
                                #right_key.to_string(),
                                #target::table_name(), #target::primary_key(),
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("many_to_many") {
                    handle_many_to_many(&meta, struct_name, field_ident, &mut relations_impls)
                } else if meta.path.is_ident("has_many_through") {
                    let args;
                    syn::parenthesized!(args in meta.input);
                    let target: syn::Type = args.parse()?;
                    let _comma: Token![,] = args.parse()?;
                    let through: syn::Type = args.parse()?;
                    let through_name = get_type_name(&through);
                    let first_key = format!("{}_id", struct_name.to_string().to_snake_case());
                    let second_key = format!("{}_id", through_name.to_snake_case());
                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::HasManyThrough<Self, #through, #target> {
                            ::rok_orm::relations::HasManyThrough::new(
                                #through::table_name(), #through::primary_key(),
                                #first_key, #second_key,
                                #target::table_name(),
                            )
                        }
                    });
                    Ok(())
                } else if meta.path.is_ident("has_one_through") {
                    let args;
                    syn::parenthesized!(args in meta.input);
                    let target: syn::Type = args.parse()?;
                    let _comma: Token![,] = args.parse()?;
                    let through: syn::Type = args.parse()?;
                    let through_name = get_type_name(&through);
                    let first_key = format!("{}_id", struct_name.to_string().to_snake_case());
                    let second_key = format!("{}_id", through_name.to_snake_case());
                    relations_impls.push(quote! {
                        fn #field_ident(&self) -> ::rok_orm::relations::HasOneThrough<Self, #through, #target> {
                            ::rok_orm::relations::HasOneThrough::new(
                                #through::table_name(), #through::primary_key(),
                                #first_key, #second_key,
                                #target::table_name(),
                            )
                        }
                    });
                    Ok(())
                } else if let Some(ts) = handle_morph_attr(&meta, struct_name, field_ident)? {
                    relations_impls.push(ts);
                    Ok(())
                } else {
                    Err(meta.error("unknown relation type"))
                }
            })?;
        }
    }

    let meta_match_arms: Vec<proc_macro2::TokenStream> = meta_entries.iter().map(|(name, table, fk)| {
        quote! { #name => Some((#table, #fk)), }
    }).collect();

    let expanded = quote! {
        impl #impl_generics ::rok_orm::Relations for #struct_name #ty_generics #where_clause {}
        impl #impl_generics ::rok_orm::relations::registry::RelationMeta for #struct_name #ty_generics #where_clause {
            fn relation_info(name: &str) -> Option<(&'static str, &'static str)> {
                match name {
                    #(#meta_match_arms)*
                    _ => None,
                }
            }
        }
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#relations_impls)*
        }
    };

    Ok(expanded.into())
}

// ── many_to_many (supports both positional and named forms) ──────────────────

fn handle_many_to_many(
    meta: &syn::meta::ParseNestedMeta<'_>,
    struct_name: &syn::Ident,
    field_ident: &syn::Ident,
    out: &mut Vec<proc_macro2::TokenStream>,
) -> syn::Result<()> {
    let (target, pivot, left_key, right_key, pivot_cols) =
        if meta.input.peek(Token![=]) {
            let value = meta.value()?;
            let target: syn::Type = value.parse()?;
            let target_name = get_type_name(&target);
            let pivot = format!("{}_{}", struct_name.to_string().to_snake_case(), target_name.to_snake_case());
            let lk = format!("{}_id", struct_name.to_string().to_snake_case());
            let rk = format!("{}_id", target_name.to_snake_case());
            (target, pivot, lk, rk, Vec::<String>::new())
        } else {
            parse_many_to_many_named(meta, struct_name)?
        };
    let with_pivot_call = if pivot_cols.is_empty() {
        quote! {}
    } else {
        quote! { .with_pivot(&[#(#pivot_cols),*]) }
    };
    out.push(quote! {
        fn #field_ident(&self) -> ::rok_orm::relations::ManyToMany<Self, #target> {
            ::rok_orm::relations::ManyToMany::new(
                #pivot, #left_key, #right_key,
                #target::table_name(), #target::primary_key(),
            ) #with_pivot_call
        }
    });
    Ok(())
}

fn parse_many_to_many_named(
    meta: &syn::meta::ParseNestedMeta<'_>,
    struct_name: &syn::Ident,
) -> syn::Result<(syn::Type, String, String, String, Vec<String>)> {
    let args;
    syn::parenthesized!(args in meta.input);
    let mut related_str: Option<String> = None;
    let mut pivot_str: Option<String> = None;
    let mut fk_str: Option<String> = None;
    let mut rfk_str: Option<String> = None;
    let mut pivots: Vec<String> = Vec::new();
    let nested = syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated(&args)?;
    for nm in nested {
        match nm {
            syn::Meta::NameValue(nv) if nv.path.is_ident("related") => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    related_str = Some(s.value());
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("pivot") => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    pivot_str = Some(s.value());
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("fk") => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    fk_str = Some(s.value());
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("rfk") => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    rfk_str = Some(s.value());
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("pivots") => {
                if let syn::Expr::Array(arr) = nv.value {
                    for elem in arr.elems {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = elem {
                            pivots.push(s.value());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let rel_name = related_str.ok_or_else(|| syn::Error::new(
        proc_macro2::Span::call_site(), "many_to_many(named) requires `related = \"Type\"`"
    ))?;
    let target: syn::Type = syn::parse_str(&rel_name)?;
    let target_name = rel_name.clone();
    let pv = pivot_str.unwrap_or_else(|| format!("{}_{}", struct_name.to_string().to_snake_case(), target_name.to_snake_case()));
    let lk = fk_str.unwrap_or_else(|| format!("{}_id", struct_name.to_string().to_snake_case()));
    let rk = rfk_str.unwrap_or_else(|| format!("{}_id", target_name.to_snake_case()));
    Ok((target, pv, lk, rk, pivots))
}
