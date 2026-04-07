//! Expansion logic for `#[derive(Relations)]`.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Token, parse::Parse};

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
                } else if meta.path.is_ident("has_many_through") {
                    // #[model(has_many_through(Target, Through))]
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
                    // #[model(has_one_through(Target, Through))]
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
                } else {
                    Err(meta.error("unknown relation type"))
                }
            })?;
        }
    }

    let expanded = quote! {
        impl #impl_generics ::rok_orm::Relations for #struct_name #ty_generics #where_clause {}
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#relations_impls)*
        }
    };

    Ok(expanded.into())
}
