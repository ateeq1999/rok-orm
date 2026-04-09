//! `morph_type_map!` — declarative macro for polymorphic parent resolution.
//!
//! Generates a typed enum + `resolve(morph_type, morph_id, pool)` method
//! that dispatches based on the `{morph_key}_type` column value.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::morph_type_map;
//!
//! morph_type_map! {
//!     ImageableMorphParent {
//!         "users" => User,
//!         "posts" => Post,
//!     }
//! }
//!
//! // Generated enum:
//! //   pub enum ImageableMorphParent {
//! //       User(User),
//! //       Post(Post),
//! //       Unknown(String, i64),
//! //   }
//!
//! // Usage:
//! let parent = ImageableMorphParent::resolve(
//!     &image.imageable_type,
//!     image.imageable_id,
//!     &pool,
//! ).await?;
//! match parent {
//!     ImageableMorphParent::User(u) => println!("user: {}", u.name),
//!     ImageableMorphParent::Post(p) => println!("post: {}", p.title),
//!     ImageableMorphParent::Unknown(t, id) => eprintln!("unknown: {} #{}", t, id),
//! }
//! ```

/// Generate a typed enum + async `resolve` method for polymorphic parent dispatch.
///
/// # Syntax
///
/// ```rust,ignore
/// morph_type_map! {
///     EnumName {
///         "type_string" => TypeName,
///         ...
///     }
/// }
/// ```
///
/// Each `"type_string"` must match the value stored in the `{morph_key}_type` column.
/// `TypeName` becomes both the enum variant name and the resolved type.
#[macro_export]
macro_rules! morph_type_map {
    (
        $enum_name:ident {
            $( $type_str:literal => $variant:ident ),+ $(,)?
        }
    ) => {
        #[derive(Debug)]
        pub enum $enum_name {
            $( $variant($variant), )+
            Unknown(String, i64),
        }

        impl $enum_name {
            /// Resolve a polymorphic parent from its type string and ID.
            ///
            /// Returns `Unknown(type_str, id)` for unregistered type strings.
            #[cfg(feature = "postgres")]
            pub async fn resolve(
                morph_type: &str,
                morph_id: i64,
                pool: &::sqlx::PgPool,
            ) -> ::rok_orm::errors::OrmResult<Self> {
                use ::rok_orm::PgModel;
                match morph_type {
                    $(
                        $type_str => {
                            let row = <$variant>::find_by_pk(pool, morph_id)
                                .await
                                .map_err(::rok_orm::errors::OrmError::from)?;
                            Ok(row
                                .map(Self::$variant)
                                .unwrap_or_else(|| Self::Unknown(morph_type.to_string(), morph_id)))
                        },
                    )+
                    _ => Ok(Self::Unknown(morph_type.to_string(), morph_id)),
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    // morph_type_map! produces a valid enum — tested via compilation
    morph_type_map! {
        TestMorphParent {
            "alphas" => Alpha,
        }
    }

    #[derive(Debug)]
    struct Alpha;
    impl crate::model::Model for Alpha {
        fn table_name() -> &'static str { "alphas" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    #[test]
    fn unknown_variant_is_constructable() {
        let p = TestMorphParent::Unknown("betas".to_string(), 99);
        match p {
            TestMorphParent::Unknown(t, id) => {
                assert_eq!(t, "betas");
                assert_eq!(id, 99);
            }
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn alpha_variant_wraps_struct() {
        let a = Alpha;
        let p = TestMorphParent::Alpha(a);
        assert!(matches!(p, TestMorphParent::Alpha(_)));
    }
}
