//! Cast encoding and `post_process` codegen for Phase 11.1.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── Cast kind ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum CastKind {
    Json,
    Bool,
    DateTime,
    Csv,
    Encrypted,
}

impl CastKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "json"      => Some(Self::Json),
            "bool"      => Some(Self::Bool),
            "datetime"  => Some(Self::DateTime),
            "csv"       => Some(Self::Csv),
            "encrypted" => Some(Self::Encrypted),
            _ => None,
        }
    }
}

// ── Data ─────────────────────────────────────────────────────────────────────

pub struct CastFieldInfo {
    pub ident: proc_macro2::Ident,
    pub col:   String,
    pub cast:  CastKind,
}

// ── Codegen ──────────────────────────────────────────────────────────────────

/// Generate `to_fields()` mixing regular and cast-encoded fields.
pub fn gen_to_fields(
    regular_pairs: &[(proc_macro2::Ident, String)],
    cast_fields:   &[CastFieldInfo],
) -> TokenStream2 {
    let mut entries: Vec<TokenStream2> = Vec::new();

    for (ident, col) in regular_pairs {
        entries.push(quote! {
            (#col, ::rok_orm::SqlValue::from(self.#ident.clone())),
        });
    }

    for cf in cast_fields {
        let ident = &cf.ident;
        let col   = &cf.col;
        let encode = match &cf.cast {
            CastKind::Json => quote! {
                ::rok_orm::SqlValue::Text(
                    ::serde_json::to_string(&self.#ident).unwrap_or_default()
                )
            },
            CastKind::Bool => quote! {
                ::rok_orm::SqlValue::Integer(if self.#ident { 1 } else { 0 })
            },
            CastKind::DateTime => quote! {
                ::rok_orm::SqlValue::Text(self.#ident.to_rfc3339())
            },
            CastKind::Csv => quote! {
                ::rok_orm::SqlValue::Text(self.#ident.join(","))
            },
            CastKind::Encrypted => quote! {
                ::rok_orm::SqlValue::Text(::rok_orm::casting::encrypt(&self.#ident))
            },
        };
        entries.push(quote! {
            (#col, #encode),
        });
    }

    quote! {
        fn to_fields(&self) -> Vec<(&'static str, ::rok_orm::SqlValue)> {
            vec![ #(#entries)* ]
        }
    }
}

/// Generate `post_process()` for `cast = "encrypted"` fields (decrypt in-place).
/// Returns empty token stream if no encrypted fields exist.
pub fn gen_post_process(cast_fields: &[CastFieldInfo]) -> TokenStream2 {
    let encrypted: Vec<_> = cast_fields.iter()
        .filter(|f| matches!(f.cast, CastKind::Encrypted))
        .collect();

    if encrypted.is_empty() {
        return quote! {};
    }

    let stmts: Vec<TokenStream2> = encrypted.iter().map(|cf| {
        let ident = &cf.ident;
        quote! {
            if let Ok(plain) = ::rok_orm::casting::decrypt(&self.#ident) {
                self.#ident = plain;
            }
        }
    }).collect();

    quote! {
        fn post_process(&mut self) {
            #(#stmts)*
        }
    }
}
