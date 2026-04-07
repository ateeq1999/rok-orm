//! Expansion logic for the `query!` proc-macro.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Token, Type,
};

pub struct QueryClause {
    pub name: Ident,
    pub args: Vec<Expr>,
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

pub struct QueryMacroInput {
    pub model: Type,
    pub clauses: Vec<QueryClause>,
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

pub fn expand_query(model: Type, clauses: Vec<QueryClause>) -> TokenStream {
    let mut chain = quote! { <#model as ::rok_orm::Model>::query() };

    for clause in clauses {
        let name_str = clause.name.to_string();
        let args = &clause.args;

        let call = match (name_str.as_str(), args.len()) {
            ("where", 2) | ("filter", 2) => {
                let (c, v) = (&args[0], &args[1]);
                quote! { .filter(#c, #v) }
            }
            ("eq", 2) | ("where_eq", 2) => {
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
