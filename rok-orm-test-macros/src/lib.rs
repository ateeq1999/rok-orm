//! Procedural macros for `rok-orm-test`.
//!
//! Provides the `#[rok_orm_test::test]` attribute for writing database tests
//! that automatically wrap in a transaction (Postgres) or use an in-memory DB
//! (SQLite), rolling back changes after each test.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, parse::Parse, parse::ParseStream,
    Ident, ItemFn, Token, LitBool,
};

// ── Attribute argument parsing ────────────────────────────────────────────────

/// Parsed arguments from `#[rok_orm_test::test(dialect, migrate = true)]`.
struct TestArgs {
    dialect: String,
    migrate: bool,
}

impl Parse for TestArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // First token is the dialect identifier (postgres / sqlite)
        let dialect: Ident = input.parse()?;
        let dialect_str = dialect.to_string();
        if !matches!(dialect_str.as_str(), "postgres" | "sqlite") {
            return Err(syn::Error::new(
                dialect.span(),
                "expected `postgres` or `sqlite`",
            ));
        }

        let mut migrate = false;
        // Optional `, migrate` or `, migrate = true`
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            let flag: Ident = input.parse()?;
            if flag != "migrate" {
                return Err(syn::Error::new(flag.span(), "expected `migrate`"));
            }
            if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                let val: LitBool = input.parse()?;
                migrate = val.value();
            } else {
                migrate = true;
            }
        }

        Ok(TestArgs { dialect: dialect_str, migrate })
    }
}

// ── #[rok_orm_test::test] ─────────────────────────────────────────────────────

/// Attribute macro for database-isolated async tests.
///
/// # Usage
///
/// ```ignore
/// #[rok_orm_test::test(postgres)]
/// async fn my_test(db: &TestDb) -> rok_orm::OrmResult<()> {
///     // All DB writes are rolled back when the test ends.
///     Ok(())
/// }
///
/// #[rok_orm_test::test(sqlite)]
/// async fn my_sqlite_test(db: &TestDb) -> rok_orm::OrmResult<()> {
///     Ok(())
/// }
///
/// #[rok_orm_test::test(postgres, migrate)]
/// async fn my_migrated_test(db: &TestDb) -> rok_orm::OrmResult<()> {
///     Ok(())
/// }
/// ```
///
/// The generated function:
/// 1. Annotates with `#[tokio::test]`
/// 2. Creates a `TestDb` appropriate for the dialect
/// 3. Calls your async test body, passing `&TestDb`
/// 4. Rolls back / tears down the `TestDb` regardless of pass/fail
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TestArgs);
    let func = parse_macro_input!(item as ItemFn);

    match expand_test(args, func) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_test(args: TestArgs, func: ItemFn) -> syn::Result<TokenStream2> {
    let fn_name = &func.sig.ident;
    let fn_body = &func.block;
    let migrate = args.migrate;

    // Validate: must be async, must have exactly one arg `db: &TestDb`
    if func.sig.asyncness.is_none() {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[rok_orm_test::test] function must be `async`",
        ));
    }

    let setup = match args.dialect.as_str() {
        "postgres" => quote! {
            let __test_db = ::rok_orm_test::TestDb::postgres(#migrate).await
                .expect("TestDb::postgres setup failed");
        },
        "sqlite" => quote! {
            let __test_db = ::rok_orm_test::TestDb::sqlite(#migrate).await
                .expect("TestDb::sqlite setup failed");
        },
        _ => unreachable!(),
    };

    // Build the inner async closure capturing the test body.
    // The user's function parameter `db: &TestDb` is replaced by `&__test_db`.
    let expanded = quote! {
        #[tokio::test]
        async fn #fn_name() {
            #setup
            let __db: &::rok_orm_test::TestDb = &__test_db;
            let __result: ::rok_orm::OrmResult<()> = (async move {
                let db: &::rok_orm_test::TestDb = __db;
                #fn_body
            }).await;
            __test_db.teardown().await;
            __result.expect("test returned an error");
        }
    };

    Ok(expanded)
}
