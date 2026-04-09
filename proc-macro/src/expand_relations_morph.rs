//! Morph attribute handlers for `#[derive(Relations)]`.
//!
//! Handles: `morph_one`, `morph_many`, `morph_to`, `morph_to_many`, `morphed_by_many`.

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;

use crate::expand_relations::get_type_name;

/// Try to handle a morph-related `#[model(...)]` attribute.
///
/// Returns `Some(tokens)` if the attribute was handled, `None` if not morph-related.
pub fn handle_morph_attr(
    meta: &syn::meta::ParseNestedMeta<'_>,
    struct_name: &syn::Ident,
    field_ident: &syn::Ident,
) -> syn::Result<Option<TokenStream>> {
    if meta.path.is_ident("morph_one") {
        let args;
        syn::parenthesized!(args in meta.input);
        let (target, morph_key, parent_type) = parse_morph_one_args(&args, struct_name)?;
        Ok(Some(quote! {
            fn #field_ident(&self) -> ::rok_orm::relations::MorphOne<Self, #target> {
                ::rok_orm::relations::MorphOne::new(#target::table_name(), #morph_key, #parent_type)
            }
        }))
    } else if meta.path.is_ident("morph_many") {
        let args;
        syn::parenthesized!(args in meta.input);
        let (target, morph_key, parent_type) = parse_morph_one_args(&args, struct_name)?;
        Ok(Some(quote! {
            fn #field_ident(&self) -> ::rok_orm::relations::MorphMany<Self, #target> {
                ::rok_orm::relations::MorphMany::new(#target::table_name(), #morph_key, #parent_type)
            }
        }))
    } else if meta.path.is_ident("morph_to") {
        let args;
        syn::parenthesized!(args in meta.input);
        let morph_key = parse_morph_key_only(&args)?;
        Ok(Some(quote! {
            fn #field_ident(&self) -> ::rok_orm::relations::MorphToRef {
                ::rok_orm::relations::MorphToRef::new(#morph_key)
            }
        }))
    } else if meta.path.is_ident("morph_to_many") {
        let args;
        syn::parenthesized!(args in meta.input);
        let (target, pivot, morph_key, parent_type) = parse_morph_many_args(&args, struct_name)?;
        let rfk = format!("{}_id", get_type_name(&target).to_snake_case());
        Ok(Some(quote! {
            fn #field_ident(&self) -> ::rok_orm::relations::MorphToMany<Self, #target> {
                ::rok_orm::relations::MorphToMany::new(
                    #pivot, #morph_key, #parent_type,
                    #rfk, #target::table_name(), #target::primary_key(),
                )
            }
        }))
    } else if meta.path.is_ident("morphed_by_many") {
        let args;
        syn::parenthesized!(args in meta.input);
        let (target, pivot, morph_key, _) = parse_morph_many_args(&args, struct_name)?;
        let related_type = struct_name.to_string().to_snake_case() + "s";
        let lfk = format!("{}_id", struct_name.to_string().to_snake_case());
        Ok(Some(quote! {
            fn #field_ident(&self) -> ::rok_orm::relations::MorphedByMany<Self, #target> {
                ::rok_orm::relations::MorphedByMany::new(
                    #pivot, #morph_key, #related_type,
                    #lfk, #target::table_name(), #target::primary_key(),
                )
            }
        }))
    } else {
        Ok(None)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_morph_one_args(
    args: &syn::parse::ParseBuffer<'_>,
    struct_name: &syn::Ident,
) -> syn::Result<(syn::Type, String, String)> {
    let mut related_str: Option<String> = None;
    let mut morph_key_str: Option<String> = None;
    let nested = syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated(args)?;
    for nm in nested {
        if let syn::Meta::NameValue(nv) = nm {
            if nv.path.is_ident("related") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    related_str = Some(s.value());
                }
            } else if nv.path.is_ident("morph_key") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    morph_key_str = Some(s.value());
                }
            }
        }
    }
    let rel_name = related_str.ok_or_else(|| syn::Error::new(
        proc_macro2::Span::call_site(), "morph_one/morph_many require `related = \"Type\"`"
    ))?;
    let target: syn::Type = syn::parse_str(&rel_name)?;
    let morph_key = morph_key_str.unwrap_or_else(|| "morphable".to_string());
    let parent_type = struct_name.to_string().to_snake_case() + "s";
    Ok((target, morph_key, parent_type))
}

fn parse_morph_key_only(args: &syn::parse::ParseBuffer<'_>) -> syn::Result<String> {
    let mut morph_key_str: Option<String> = None;
    let nested = syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated(args)?;
    for nm in nested {
        if let syn::Meta::NameValue(nv) = nm {
            if nv.path.is_ident("morph_key") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    morph_key_str = Some(s.value());
                }
            }
        }
    }
    Ok(morph_key_str.unwrap_or_else(|| "morphable".to_string()))
}

fn parse_morph_many_args(
    args: &syn::parse::ParseBuffer<'_>,
    struct_name: &syn::Ident,
) -> syn::Result<(syn::Type, String, String, String)> {
    let mut related_str: Option<String> = None;
    let mut pivot_str: Option<String> = None;
    let mut morph_key_str: Option<String> = None;
    let nested = syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated(args)?;
    for nm in nested {
        if let syn::Meta::NameValue(nv) = nm {
            if nv.path.is_ident("related") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    related_str = Some(s.value());
                }
            } else if nv.path.is_ident("pivot") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    pivot_str = Some(s.value());
                }
            } else if nv.path.is_ident("morph_key") {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                    morph_key_str = Some(s.value());
                }
            }
        }
    }
    let rel_name = related_str.ok_or_else(|| syn::Error::new(
        proc_macro2::Span::call_site(), "morph_to_many/morphed_by_many require `related`"
    ))?;
    let target: syn::Type = syn::parse_str(&rel_name)?;
    let pivot = pivot_str.ok_or_else(|| syn::Error::new(
        proc_macro2::Span::call_site(), "morph_to_many/morphed_by_many require `pivot`"
    ))?;
    let morph_key = morph_key_str.unwrap_or_else(|| "morphable".to_string());
    let parent_type = struct_name.to_string().to_snake_case() + "s";
    Ok((target, pivot, morph_key, parent_type))
}
