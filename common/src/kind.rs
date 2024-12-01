//! Macros for defining kind enums.

/// Macro for defining a kind enum.
///
/// # Example
///
/// ```rust
/// # use crate::common::define_kind;
///
/// define_kind! {
///     #[doc = "Shape kind."]
///     enum Kind {
///         #[doc = "A cube"]
///         Cube = 1,
///
///         #[doc = "A sphere"]
///         Sphere = 2,
///     }
/// }
#[expect(clippy::module_name_repetitions, reason = "more readable")]
#[macro_export]
macro_rules! define_kind {
    (
        #[doc = $doc:literal]
        enum $name:ident {
            $(
                #[doc = $variant_doc:literal]
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            $crate::private::strum::Display,
            $crate::private::strum::EnumString,
            Eq,
            PartialEq,
        )]
        #[cfg_attr(
            feature = "serde",
            derive(
                $crate::private::serde::Deserialize,
                $crate::private::serde::Serialize,
            ),
            serde(rename_all = "SCREAMING_SNAKE_CASE"),
        )]
        #[doc = $doc]
        #[repr(u8)]
        #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
        pub enum $name {
            $(
                 #[doc = $variant_doc]
                 $variant = $value,
            )*
        }

        impl $name {
            /// Converts this into its [`u8`] representation.
            #[must_use]
            pub const fn u8(self) -> u8 {
                self as u8
            }
        }

        $(
            impl $crate::FromParam<$value> for $name {
                const VALUE: $name = $name::$variant;
            }
        )*

        #[cfg(feature = "postgres")]
        impl<'a> $crate::private::postgres_types::FromSql<'a> for $name {
            $crate::private::postgres_types::accepts!(INT2);

            fn from_sql(
                ty: &$crate::private::postgres_types::Type,
                raw: &[u8],
            ) -> Result<
                $name,
                Box<dyn ::std::error::Error
                    + ::core::marker::Sync
                    + ::core::marker::Send>,
            > {
                match u8::try_from(i16::from_sql(ty, raw)?)? {
                    $(
                        v if Self::$variant.u8() == v => Ok(Self::$variant),
                    )*
                    v => Err(::std::format!(
                        "invalid `{}` value: {v}",
                        ::core::stringify!($name),
                    ).into()),
                }
            }
        }

        #[cfg(feature = "postgres")]
        impl $crate::private::postgres_types::ToSql for $name {
            $crate::private::postgres_types::accepts!(INT2);
            $crate::private::postgres_types::to_sql_checked!();

            fn to_sql(
                &self,
                ty: &$crate::private::postgres_types::Type,
                w: &mut $crate::private::postgres_types::private::BytesMut,
            ) -> Result<
                $crate::private::postgres_types::IsNull,
                ::std::boxed::Box<
                    dyn ::std::error::Error
                        + ::core::marker::Sync
                        + ::core::marker::Send
                >,
            > {
                i16::from(self.u8()).to_sql(ty, w)
            }
        }
    };
}

/// Helper trait converting const parameter to a value.
pub trait FromParam<const PARAM: u8> {
    /// Value of the parameter.
    const VALUE: Self;
}
